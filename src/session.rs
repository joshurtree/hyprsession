use std::fs::File;
use std::io::{read_to_string, Write};
use std::process::Command;

use hyprland::data::{Client, Clients};
use hyprland::dispatch::*;
use hyprland::keyword::Keyword;
//use hyprland::event_listener::EventListener;
use hyprland::prelude::*;

use crate::{AppConfig, AppConfigs};
//use hyprland::shared::WorkspaceType;

const EXEC_NAME: &str = "/exec.conf";
const RULES_NAME: &str = "/windowrules.conf";

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

pub fn save_session(base_path: &str, apps: &AppConfigs) -> Result<(), std::io::Error> {
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

    let client_info = Clients::get().expect("Unable to fetch clients");

    let mut exec_file = File::create(base_dir.clone() + EXEC_NAME)
        .map_err(|err| log::error!("Failed to create exec file: {}", err))
        .expect("");

    let mut rules_file = File::create(base_dir.clone() + RULES_NAME)
        .map_err(|err| log::error!("Failed to create rules file {}", err))
        .expect("");

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
            writeln!(exec_file, "{}\n", exec_line)?;

            if appconfig.apply_windowrules {
                for exec_opt in exec_opts {
                    writeln!(rules_file, "windowrule={}, {}\n", exec_opt, info.initial_class)?;
                }
            }
        } else {
            log::info!("Ignoring app with initialClass: {}", info.initial_class);
        }
    }
    log::info!("Session saved");
    return Ok(());
}

pub fn load_session(base_path: &String, simulate: bool) {
    let base_dir = base_path.to_owned();
    let session_file = File::open(base_dir + EXEC_NAME);
    
    if session_file.is_ok() {
        let mut temp_rules_path = std::env::temp_dir();
        temp_rules_path.push(RULES_NAME);
        std::fs::copy(base_dir.clone() + RULES_NAME, &temp_rules_path).unwrap();

        if !simulate {
            Keyword::set("source", temp_rules_path.as_os_str().to_str().unwrap()).unwrap();
        }

        for line in read_to_string(session_file.unwrap()).unwrap().lines() {
            if !simulate {
                hyprland::dispatch!(Exec, line).unwrap();
            } 
            println!("Sending: dispach exec {}", line);
        }
    }
}