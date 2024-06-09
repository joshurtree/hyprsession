use std::fs::File;
use std::io::{read_to_string, Write};
use std::path::Path;
use std::process::Command;

use hyprland::data::{Client, Clients};
use hyprland::dispatch;
use hyprland::dispatch::*;
use hyprland::dispatch::DispatchType::*;
use hyprland::keyword::Keyword;
use hyprland::prelude::*;
use hyprland::event_listener::EventListener;

use crate::{AppConfig, AppConfigs};
//use hyprland::shared::WorkspaceType;

const EXEC_NAME: &str = "exec.conf";
const RULES_NAME: &str ="windowrules.conf";
const CLIENTS_NAME: &str = "windowslist.json";

fn fetch_command(info: &Client, conf: &AppConfig) -> String {
    let output = Command::new("ps")
                    .arg("--no-headers")
                    .arg("-o")
                    .arg("cmd")
                    .arg("-p")
                    .arg(format!("{}", info.pid))
                    .output()
                    .expect("Failed to call ps");

    let mut command_and_args = String::from_utf8_lossy(&output.stdout).to_string();
    command_and_args.pop();
    let command: Vec<&str> = command_and_args
            .splitn(2, " ")
            .collect();
    //log::debug!("{}", command.join(", "));
    return command[0].to_owned() + if command.len() == 1 || conf.ignore_arguments { "" } else { command[1] } + &conf.extra_args;
}

fn run_if(prop: bool, val: &str) -> &str {
    if prop { val } else { "" }
}

pub fn save_session(base_path: &Path, apps: &AppConfigs) -> Result<(), std::io::Error> {
    let base_dir = base_path.to_owned();
    let props = [ 
        |info: &Client| format!("monitor {}", info.monitor),
        |info: &Client| format!("workspace {} silent", info.workspace.id), 
        |info: &Client| format!("{}", run_if(info.floating, "float")),
        |info: &Client| format!("move {} {}", info.at.0, info.at.1),  
        |info: &Client| format!("size {} {}", info.size.0, info.size.1), 
        |info: &Client| format!("{}", run_if(info.pinned, "pin")), 
        |info: &Client| format!("{}", run_if(info.fullscreen, "fullscreen")), 
        //|info: &Client| format!("{}", run_if(info.fake_fullscreen, "fakefullscreen")) 
    ];

    let client_info = Clients::get()
        .map_err(|err| log::error!("Unable to fetch clients: {}", err))
        .unwrap();

    let mut exec_file = File::create(base_dir.join(EXEC_NAME))
        .map_err(|err| log::error!("Failed to create exec file: {}", err))
        .unwrap();
    
    let clients_file = File::create(base_dir.join(CLIENTS_NAME))
        .map_err(|err| log::error!("Failed to create windows file {}", err))
        .unwrap();

    serde_json::to_writer(clients_file, &client_info.iter().collect::<Vec<_>>())?;

    for info in client_info.iter() {
        let defconfig = Default::default();
        let appconfig: &AppConfig = apps.get(&info.initial_class).unwrap_or(&defconfig);

        if !appconfig.ignore {
            let exec_opts: Vec<String> = 
                props
                    .iter()
                    .map(|opt| opt(info))
                    .filter(|opt| !opt.is_empty())
                    .collect();

            let exec_line = format!("[{}] {}", exec_opts.join(";"), fetch_command(info, appconfig));
            log::debug!("Adding line to execution file: {}", exec_line);
            writeln!(exec_file, "{}", exec_line)?;
        } else {
            log::info!("Ignoring app with initialClass: {}", info.initial_class);
        }
    }
    log::info!("Session saved");
    return Ok(());
}

fn dispatch_client_props(client: &Client) -> hyprland::Result<()> {
    let wid = WindowIdentifier::Address(client.address.clone());

    if client.pinned { Dispatch::call(TogglePin)?; }
    if client.floating { dispatch!(ToggleFloating, None)?; }
    if client.fullscreen { dispatch!(Custom, "fullscreen", &client.fullscreen_mode.to_string())?; }
    dispatch!(MoveWindowPixel, Position::Exact(client.at.0, client.at.1), wid.clone())?;
    dispatch!(ResizeWindowPixel, Position::Exact(client.size.0, client.size.1), wid.clone())?;
    dispatch!(MoveToWorkspaceSilent, WorkspaceIdentifierWithSpecial::Id(client.workspace.id), None)?;
    Ok(())
}
pub fn load_session(base_path: &Path, simulate: bool) -> Result<(), std::io::Error> {
    let base_dir = base_path.to_owned();
    let session_file = File::open(base_dir.join(EXEC_NAME));
    let clients: Vec<Client> = serde_json::from_reader(File::open(base_dir.join(CLIENTS_NAME))?)
        .map_err(|err| log::error!("Error parsing clients: {:#?}", err))
        .unwrap();

    if session_file.is_ok() {
        let mut temp_rules_path = std::env::temp_dir();
        temp_rules_path.push(RULES_NAME);
        std::fs::copy(base_dir.join(RULES_NAME), &temp_rules_path).unwrap();
        
        if !simulate {
            Keyword::set("source", temp_rules_path.as_os_str().to_str().unwrap()).unwrap();
            let mut event_listener = EventListener::new();
            event_listener.add_window_open_handler( move |data| {
                clients.iter().find_map(|client| { 
                    if client.initial_class == data.window_class {
                        dispatch_client_props(client)
                            .map_err(|err| log::error!("Error dispatching client props: {:?}", err))
                            .unwrap();
                        Some(())
                    } else {
                        None
                    }
                }); 
            });
        }

        for line in read_to_string(session_file.unwrap())?.lines() {
            if !simulate {
                hyprland::dispatch!(Exec, line).unwrap();
            } 
            println!("Sending: dispach exec {}", line);
        }
    }

    Ok(())
}