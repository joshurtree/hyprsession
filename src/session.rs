use std::fs::File;
use std::io::{read_to_string, Write};
use std::process::Command;

use hyprland::data::{Client, Clients};
use hyprland::dispatch::*;
//use hyprland::keyword;
//use hyprland::event_listener::EventListener;
use hyprland::prelude::*;
//use hyprland::shared::WorkspaceType;

const EXEC_NAME: &str = "/exec.conf";

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

pub fn save_session(base_path: &str) {
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
    
    let mut exec_file = File::create(base_dir + EXEC_NAME)
        .expect("Failed to create session file");
    for info in client_info.iter() {
        let exec_opts: Vec<String> = 
            props
                .iter()
                .map(|opt| opt(info))
                .filter(|opt| !opt.is_empty())
                .collect();
        exec_file.write(format!("[{}] {}", exec_opts.join(";"), fetch_command(info)).as_bytes());
    }
    println!("Session saved");
}

pub fn load_session(base_path: &String, simulate: bool) {
    let base_dir = base_path.to_owned();
    let session_file = File::open(base_dir + EXEC_NAME);
    
    if session_file.is_ok() {
        for line in read_to_string(session_file.unwrap()).unwrap().lines() {
            if !simulate {
                hyprland::dispatch!(Exec, line);
            } 
            println!("Sending: dispach exec {}", line);
        }
    }
}