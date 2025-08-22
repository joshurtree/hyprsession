
use std::fs::{File, read_to_string};
use std::io::Write;
use std::process::Command;
//use std::thread;
//use std::time;

use hyprland::data::{Client, Clients, FullscreenMode};
use hyprland::{dispatch, dispatch::* };
//use hyprland::keyword;
use hyprland::event_listener::{AsyncEventListener};
use hyprland::prelude::*;
//use hyprland::shared::WorkspaceType;

const EXEC_PATH_NAME: &str = "/exec.conf";
const CLIENTS_PATH_NAME: &str = "/clients.json";

fn fetch_command(info: &Client) -> String {
    let output = Command::new("ps")
                    .arg("--no-headers")
                    .arg("-o")
                    .arg("cmd")
                    .arg("-p")
                    .arg(format!("{}", info.pid))
                    .output()
                    .expect("Failed to call ps");
    return String::from_utf8_lossy(&output.stdout).to_string();
}

fn run_if(prop: bool, val: &str) -> &str {
    if prop { val } else { "" }
}

fn adjust_client(client: &Client, matching_client: &Client) {
    dispatch!(MoveToWorkspaceSilent, 
                        WorkspaceIdentifierWithSpecial::Id(matching_client.workspace.id), 
                        Some(WindowIdentifier::Address(client.address.clone())))
        .expect(&format!("Failed to move client window: {:?}", client.title));
    dispatch!(MoveWorkspaceToMonitor, 
                        WorkspaceIdentifier::Id(matching_client.workspace.id), 
                        MonitorIdentifier::Id(matching_client.monitor))
        .expect(&format!("Failed to move workspace to monitor: {}", matching_client.monitor));
    if matching_client.floating {
        dispatch!(ToggleFloating, Some(WindowIdentifier::Address(client.address.clone())))
            .expect(&format!("Failed to toggle floating for client: {:?}", client.title));
    }

    if matching_client.pinned != client.pinned {
        dispatch!(FocusWindow, 
                            WindowIdentifier::Address(client.address.clone()))
            .expect(&format!("Failed to focus client window: {:?}", client.title));
    
        Dispatch::call(DispatchType::TogglePin)
            .expect(&format!("Failed to toggle pin for client: {:?}", client.title));
    }

    if matching_client.fullscreen != client.fullscreen {
        hyprland::dispatch!(FocusWindow, 
                            WindowIdentifier::Address(client.address.clone()))
            .expect(&format!("Failed to focus client window: {:?}", client.title));
        

        hyprland::dispatch!(ToggleFullscreen, 
            if matching_client.fullscreen == FullscreenMode::Maximized {FullscreenType::Maximize} else {FullscreenType::Real})
            .expect(&format!("Failed to toggle fullscreen for client: {:?}", client.title));
    }

    if matching_client.fullscreen != FullscreenMode::None {
        hyprland::dispatch!(MoveWindowPixel, 
                            Position::Exact(matching_client.at.0, matching_client.at.1), 
                            WindowIdentifier::Address(client.address.clone()))
            .expect(&format!("Failed to move client window: {:?}", client.title));
        hyprland::dispatch!(ResizeWindowPixel, 
                            Position::Exact(matching_client.size.0, matching_client.size.1), 
                            WindowIdentifier::Address(client.address.clone()))
            .expect(&format!("Failed to resize client window: {:?}", client.title));
    }
}

pub fn save_session(base_path: &str) {
    let base_dir = base_path.to_owned();
    let props = [ 
        |info: &Client| format!("monitor {}", info.monitor),
        |info: &Client| format!("workspace {} silent", info.workspace.id), 
        |info: &Client| format!("{}", run_if(info.floating, "float")),
        |info: &Client| format!("move {} {}", info.at.0, info.at.1),  
        |info: &Client| format!("size {} {}", info.size.0, info.size.1), 
        |info: &Client| format!("{}", run_if(info.pinned, "pin")), 
        |info: &Client| format!("fullscreenstate {}", info.fullscreen as i32), 
        //|info: &Client| format!("{}", run_if(info.fake_fullscreen, "fakefullscreen")) 
    ];

    let client_info = Clients::get().expect("Unable to fetch clients");
    let mut exec_file = File::create(base_dir.to_owned() + EXEC_PATH_NAME)
        .expect("Failed to create session file");
    let mut clients_file = File::create(base_dir + CLIENTS_PATH_NAME)
            .expect("Failed to create clients file");
    let mut pids: Vec<i32> = vec![];

    for info in client_info.iter() {
        if !pids.contains(&info.pid) {
            let exec_opts: Vec<String> = 
                props
                    .iter()
                    .map(|opt| opt(info))
                    .filter(|opt| !opt.is_empty())
                    .collect();
            let _ = exec_file.write(format!("[{}] {}", 
                            exec_opts.join(";"), 
                            fetch_command(info)).as_bytes());
            pids.push(info.pid);
        } 

        serde_json::to_writer(&clients_file, &info)
            .expect("Failed to write to clients file");
    }
    clients_file.flush().expect("Failed to flush clients file");
    println!("Session saved");
}

pub fn load_session(base_path: &String, load_time: u64, simulate: bool) {
    let base_dir = base_path.to_owned();
    let start_time = std::time::Instant::now();

    for line in read_to_string(base_dir.to_owned() + EXEC_PATH_NAME).unwrap().lines() {
        if !simulate {
            hyprland::dispatch!(Exec, line)
                .expect(&format!("Failed to dispatch exec: {}", line));
        } 
        println!("Sending: dispach exec {}", line);
    }

    let clients_data: &'static str = Box::leak(read_to_string(base_dir + CLIENTS_PATH_NAME).unwrap_or_else(|_| {
        println!("No clients data found, skipping client adjustments");
        String::new()
    }).into_boxed_str());

    let mut event_listener = AsyncEventListener::new();
    if !simulate {
        event_listener.add_window_title_change_handler(async_closure! {move |address| {
            let clients: Vec<Client> = serde_json::from_str(clients_data)
                .expect("Failed to parse clients data");

            if start_time.elapsed().as_secs() > load_time { 
                println!("Load time exceeded, skipping client adjustments");
                return;
            }
            
            for client in clients.iter() {
                if let Some(matching_client) = Clients::get().expect("Unable to fetch clients")
                                                             .iter()
                                                             .find(|c| c.address == address) { 
                    adjust_client(client, matching_client);
                }
            }
        }});
    }
    let _ = event_listener.start_listener_async();
}
