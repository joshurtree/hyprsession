use std::{env, thread, time};
use std::fs::{create_dir_all, File};
use std::io::{read_to_string, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
//use serde::Deserialize;
use clap::{Parser, ValueEnum};

use hyprland::data::{Client, Clients};
use hyprland::dispatch::*;
//use hyprland::event_listener::EventListener;
//use hyprland::keyword::*;
use hyprland::prelude::*;
//use hyprland::shared::WorkspaceType;

/*
#[derive(Debug, Deserialize)]
struct WorkspaceInfo {
    id: i32,
    name: String
}


#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
struct ClientInfo {
    address: String,
    mapped: bool,
    hidden: bool,
    at: (i32, i32),
    size: (i32, i32),
    workspace: WorkspaceInfo,
    floating: bool,
    monitor: i32,
    class: String,
    title: String,
    initial_class: String,
    initial_title: String,
    pid: i32,
    xwayland: bool,
    pinned: bool,
    fullscreen: bool,
    fullscreen_mode: i32,
    fake_fullscreen: bool,
    grouped: Vec<String>,
    swallowing: String,
}
*/

#[derive(Copy, Clone, Parser, PartialEq, ValueEnum)]
enum Mode {
    /// Load session then periodicly save session (default)
    Default,
    
    /// Periodicly save the session 
    SaveOnly,
    /// Save the session once then exit
    SaveAndExit,

    /// Load the session then exit
    LoadAndExit
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Which mode to run the program in
    #[arg(short, long)]
    mode: Option<Mode>,

    /// Interval between saving sessions (default: 60)
    #[arg(short='i', long)]
    save_interval: Option<u64>,
    
    /// The path where the session is saved (default: ~/.local/share/session.conf)
    #[arg(short='s', long)]
    session_path: Option<PathBuf>,

    /// Only simulate calls to Hyprland (supresses loading of session)
    #[arg(long)]
    simulate: bool
}   


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

fn save_session(session_path: &PathBuf) {
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
    
    let mut session_config = File::create(session_path)
                                .expect("Failed to create session file");
    for info in client_info.iter() {
        let exec_opts: Vec<String> = 
            props
                .iter()
                .map(|opt| opt(info))
                .filter(|opt| !opt.is_empty())
                .collect();
        session_config.write(format!("[{}] {}", exec_opts.join(";"), fetch_command(info)).as_bytes());
    }
    println!("Session saved");
}


fn load_session(session_path: &PathBuf, simulate: bool) {
    let session_file = File::open(session_path).expect("Failed to open session file.");
    
    for line in read_to_string(session_file).unwrap().lines() {
        if !simulate {
            hyprland::dispatch!(Exec, line);
        } 
        println!("Sending: dispach exec {}", line);
    }
}

fn main() {
    let args = Args::parse();
    let mode = args.mode.unwrap_or(Mode::Default);
    let save_interval = args.save_interval.unwrap_or(60);
    let simulate = args.simulate;
    let share_dir = env::var("HOME").unwrap() + "/.local/share/hyprsession";

    if save_interval < 1 {
        panic!("Save interval needs to be a positive integer");
    }

    create_dir_all(&share_dir).expect("Failed to create share dir");
    let default_path = PathBuf::from(share_dir + "/session.conf");
    let session_path = args.session_path.unwrap_or(default_path);

    match mode {
        Mode::Default | Mode::LoadAndExit =>
            load_session(&session_path, simulate),
        Mode::SaveAndExit | Mode::SaveOnly => save_session(&session_path)
    } 

    if mode == Mode::LoadAndExit || mode == Mode::SaveAndExit {
        exit(0);
    }

    loop {
        save_session(&session_path);
        thread::sleep(time::Duration::from_secs(save_interval));
    }
}
    