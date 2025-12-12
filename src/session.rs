use hyprland::data::{Client, Clients, FullscreenMode};
use hyprland::{dispatch, dispatch::* };
use hyprland::event_listener::EventListener;
use hyprland::prelude::*;
use hyprland::shared::Address;
use std::fs::File;
use std::io::{read_to_string, Write};
use crate::command_detection::fetch_command;

const EXEC_NAME: &str = "/exec.conf";
const CLIENTS_PATH_NAME: &str = "/clients.json";

macro_rules! iif {
    ($prop:expr, $val:expr) => {
        if $prop { $val } else { "" }
    };
    ($prop:expr, $val:expr, $alt:expr) => {
        if $prop { $val } else { $alt }
    };
}

fn adjust_client(client: &Client, matching_client: &Client) {
    dispatch!(MoveToWorkspaceSilent, 
                        WorkspaceIdentifierWithSpecial::Id(matching_client.workspace.id), 
                        Some(WindowIdentifier::Address(client.address.clone())))
        .expect(&format!("Failed to move client window: {:?}", client.title));
    dispatch!(MoveWorkspaceToMonitor, 
                        WorkspaceIdentifier::Id(matching_client.workspace.id), 
                        MonitorIdentifier::Id(matching_client.monitor.unwrap_or(0)))
        .expect(&format!("Failed to move workspace to monitor: {:?}", matching_client.monitor.unwrap_or(0)));
    if matching_client.floating != client.floating {
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

fn process_window_event(address: Address, clients_data: &'static str, start_time: std::time::Instant, load_time: u64, simulate: bool) {
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
            println!("Adjusting client: {:?}", matching_client.title);
            if !simulate { adjust_client(client, matching_client); }
        }
    }
}

pub fn save_session(base_path: &str, save_duplicate_pids: bool) -> hyprland::Result<()> {
    let base_dir = base_path.to_owned();
    let props = [
        |info: &Client| format!("monitor {:?}", info.monitor.unwrap_or(0)),
        |info: &Client| iif!(info.workspace.id == -99,
                             format!("workspace special silent"),
                             format!("workspace {} silent", info.workspace.id)),
        |info: &Client| format!("{}", iif!(info.floating, "float")),
        |info: &Client| format!("move {} {}", info.at.0, info.at.1),
        |info: &Client| format!("size {} {}", info.size.0, info.size.1),
        |info: &Client| format!("{}", iif!(info.pinned, "pin")),
        |info: &Client| format!("fullscreenstate {}", info.fullscreen as i32),
    ];

    let client_info = Clients::get().expect("Unable to fetch clients");

    let mut exec_file = File::create(base_dir.clone() + EXEC_NAME)
        .expect("Failed to create session file");
    let clients_file = File::create(base_dir.clone() + CLIENTS_PATH_NAME)
        .expect("Failed to create clients file");
    let mut pids: Vec<i32> = vec![];
    let mut saved_clients: Vec<Client> = vec![];

    for info in client_info.iter().rev() {
        saved_clients.push(info.clone());
        if !save_duplicate_pids && pids.contains(&info.pid) {
            continue;
        }
        pids.push(info.pid);

        let exec_opts: Vec<String> = props
            .iter()
            .map(|opt| opt(info))
            .filter(|opt| !opt.is_empty())
            .collect();
        exec_file
            .write_all(format!("[{}] {}\n", exec_opts.join(";"), fetch_command(info)).as_bytes())?;
    }

    serde_json::to_writer(&clients_file, &saved_clients)
        .expect("Failed to write to clients file");
    println!("Session saved");
    Ok(())
}

fn load_programs(base_path: &String, simulate: bool) -> hyprland::Result<()> {
    let base_dir = base_path.to_owned();

    let session_file = File::open(base_dir.clone() + EXEC_NAME);

    if session_file.is_ok() {
        for line in read_to_string(session_file.unwrap()).unwrap().lines() {
            if !simulate {
                hyprland::dispatch!(Exec, line)?;
            }
            println!("Sending: dispatch exec {line}");
        }
    }

    Ok(())
}

pub fn load_session(base_path: &String, load_time:  u64, adjust_clients_only: bool, simulate: bool) -> hyprland::Result<()> {
    let base_dir = base_path.to_owned();
    let start_time = std::time::Instant::now();

    if adjust_clients_only {
        load_programs(&base_dir, simulate)?;
    }

    std::thread::spawn(move || {
        let clients_file_path = base_dir + CLIENTS_PATH_NAME;
        let clients_data = std::fs::read_to_string(&clients_file_path)
            .unwrap_or_else(|_| "[]".to_string());
        let clients_data: &'static str = Box::leak(clients_data.into_boxed_str());

        let mut event_listener = EventListener::new();
        event_listener.add_window_title_changed_handler({move |event| {
            process_window_event(event.address, clients_data, start_time, load_time, simulate);
        }});
        event_listener.add_window_opened_handler({move |event| {
            process_window_event(event.window_address, clients_data, start_time, load_time, simulate);
        }});
        let _ = event_listener.start_listener();
    });
        
    Ok(())
}

pub fn clear_session(simulate: bool) -> hyprland::Result<()> {
    if !simulate {
        for info in Clients::get()?.iter() {
            let _ = std::process::Command::new("kill")
                .arg(info.pid.to_string())
                .output()
                .unwrap();
        }
    }
    println!("Cleared existing session");
    Ok(())
}