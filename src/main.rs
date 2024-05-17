use std::{env, fs, thread, time};
use std::process::{Command, exit};
use std::io::prelude::*;
use serde::Deserialize;
use serde_json;
use std::path::PathBuf;
use clap::Parser;

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short='o', long)]
    save_once: bool,

    #[arg(short='i', long)]
    save_immediately: bool,

    #[arg(short, long)]
    save_interval: Option<u64>,

    #[arg(short='p', long)]
    session_path: Option<PathBuf>,
}   

fn fetch_command(info: &ClientInfo) -> String {
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
        |info: &ClientInfo| format!("monitor {}", info.monitor),
        |info: &ClientInfo| format!("workspace {} silent", info.workspace.id), 
        |info: &ClientInfo| format!("{}", run_if(info.floating, "float")),
        |info: &ClientInfo| format!("move {} {}", info.at.0, info.at.1),  
        |info: &ClientInfo| format!("size {} {}", info.size.0, info.size.1), 
        |info: &ClientInfo| format!("{}", run_if(info.pinned, "pin")), 
        |info: &ClientInfo| format!("{}", run_if(info.fullscreen, "fullscreen")), 
        |info: &ClientInfo| format!("{}", run_if(info.fake_fullscreen, "fakefullscreen")) 
    ];

    let output = Command::new("hyprctl")
                    .arg("-j")
                    .arg("clients")
                    .output()
                    .expect("Failed to call hyprctl");

    let client_info: Vec<ClientInfo> = serde_json::from_str(&String::from_utf8_lossy(&output.stdout).to_string()).unwrap();

    let mut session_config = fs::File::create(session_path)
                                .expect("Failed to create session file");
    
    for info in &client_info {
        let exec_opts: Vec<String> = 
            props
                .iter()
                .map(|opt| opt(info))
                .filter(|opt| !opt.is_empty())
                .collect();
        session_config.write(format!("exec-once = [{}] {}", exec_opts.join(";"), fetch_command(info)).as_bytes());
    }
    println!("Session saved");
}

fn main() {
    let args = Args::parse();
    let save_interval = args.save_interval.unwrap_or(60);
    if save_interval < 1 {
        panic!("Save interval needs to be a positive integer");
    }

    let share_dir = env::var("HOME").unwrap() + "/.local/share/hyprsession";
    fs::create_dir_all(&share_dir).expect("Failed to create share dir");
    let default_path = PathBuf::from(share_dir + "/session.conf");
    let session_path = args.session_path.unwrap_or(default_path);

    fs::File::create(&session_path)
        .expect("Failed to create session file");

    if args.save_once || args.save_immediately || cfg!(debug_assertions) {
        save_session(&session_path);
        if args.save_once { 
            exit(0);
        }
    }

    loop {
        thread::sleep(time::Duration::from_secs(save_interval));
        save_session(&session_path);
    }
}
    