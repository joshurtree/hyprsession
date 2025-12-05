use hyprland::data::{Client, Clients};
use hyprland::prelude::*;
use std::fs::File;
use std::io::{read_to_string, Write};
use crate::command_detection::fetch_command;

const EXEC_NAME: &str = "/exec.conf";



macro_rules! iif {
    ($prop:expr, $val:expr) => {
        if $prop { $val } else { "" }
    };
    ($prop:expr, $val:expr, $alt:expr) => {
        if $prop { $val } else { $alt }
    };
}

pub fn save_session(base_path: &str, save_duplicate_pids: bool) {
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
        //|info: &Client| iif!(info.fake_fullscreen, "fakefullscreen", "")
    ];

    let client_info = Clients::get().expect("Unable to fetch clients");

    let mut exec_file = File::create(base_dir + EXEC_NAME).expect("Failed to create session file");
    let mut pids: Vec<i32> = vec![];

    for info in client_info.iter().rev() {
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
            .write_all(format!("[{}] {}", exec_opts.join(";"), fetch_command(info)).as_bytes());
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
            println!("Sending: dispach exec {line}");
        }
    }
}



