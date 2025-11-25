use hyprland::data::{Client, Clients};
use hyprland::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{read_to_string, Write};
use std::process::Command;

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
    let status = output.status;

    if !status.success() {
        eprintln!(
            "Failed to fetch command for PID {}: {}",
            info.pid,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if output.stdout.is_empty() {
        return String::from("unknown");
    }
    let command = String::from_utf8_lossy(&output.stdout).to_string();

    let re = Regex::new(r"(?<path>.*)\.(?<command>.+)-wrapped").unwrap();
    if let Some(captures) = re.captures(&command) {
        captures.name("path").unwrap().as_str().to_string()
            + captures.name("command").unwrap().as_str()
            + "\n"
    } else {
        command.trim().to_string() + "\n"
    }
}

fn iif<'a>(prop: bool, val: &'a str, alt: &'a str) -> &'a str {
    if prop {
        val
    } else {
        alt
    }
}

pub fn save_session(base_path: &str, save_duplicate_pids: bool) {
    let base_dir = base_path.to_owned();
    let props = [
        |info: &Client| format!("monitor {:?}", info.monitor),
        |info: &Client| format!("workspace {} silent",
            iif(
                info.workspace.id == -99,
                "special",
                stringify!(info.workspace.id)
            )
        ),
        |info: &Client| format!("{}", iif(info.floating, "float", "")),
        |info: &Client| format!("move {} {}", info.at.0, info.at.1),
        |info: &Client| format!("size {} {}", info.size.0, info.size.1),
        |info: &Client| format!("{}", iif(info.pinned, "pin", "")),
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
