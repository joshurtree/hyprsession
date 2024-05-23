use std::{env, thread, time};
use std::fs::create_dir_all;
use std::process::exit;
//use serde::Deserialize;
use clap::{Parser, ValueEnum};

mod session;
use crate::session::*;

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
    
    /// The path where the session is saved (default: ~/.local/share)
    #[arg(short='s', long)]
    session_path: Option<String>,

    /// Only simulate calls to Hyprland (supresses loading of session)
    #[arg(long)]
    simulate: bool
}   

fn main() {
    let args = Args::parse();
    let mode = args.mode.unwrap_or(Mode::Default);
    let save_interval = args.save_interval.unwrap_or(60);
    let simulate = args.simulate;
    let default_path = env::var("HOME").unwrap() + "/.local/share/hyprsession";
    let session_path = args.session_path.unwrap_or(default_path);

    if save_interval < 1 {
        panic!("Save interval needs to be a positive integer");
    }

    create_dir_all(&session_path)
        .expect(&format!("Failed to create session dir: {}", session_path));

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
    