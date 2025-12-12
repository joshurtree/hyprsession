use std::fs::create_dir_all;
use std::process::exit;
use std::{env, thread, time};
//use serde::Deserialize;
use clap::{Parser, ValueEnum};

use crate::session::*;

#[derive(Copy, Clone, PartialEq, ValueEnum)]
enum Mode {
    /// Load session then periodicly save session (default)
    Default,

    /// Periodicly save the session
    SaveOnly,

    /// Save the session once then exit
    SaveAndExit,

    /// Load the session then exit
    LoadAndExit,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Which mode to run the program in (default: default)
    #[arg(long, value_enum, default_value_t = Mode::Default)]
    mode: Mode,

    /// Whether to store multiple clients owned by the same application
    #[arg(long, default_value_t = false)]
    save_duplicate_pids: bool,

    /// Interval between saving sessions (default: 60)
    #[arg(short = 'i', long, default_value_t = 60)]
    save_interval: u64,

    /// The path where the session is saved (default: ~/.local/share)
    #[arg(short = 's', long)]
    session_path: Option<String>,

    #[arg(short = 'l', long, default_value_t = 30)]
    load_time: u64,

    /// Only simulate calls to Hyprland (supresses loading of session)
    #[arg(long, default_value_t = false)]
    simulate: bool,
}

pub fn main() -> hyprland::Result<()> {
    let args = Args::parse();
    let mode = args.mode;
    let save_interval = args.save_interval;
    let simulate = args.simulate;
    let default_path = env::var("HOME").unwrap() + "/.local/share/hyprsession";
    let session_path = args.session_path.unwrap_or(default_path);

    if args.save_interval == 0 {
        eprintln!("Error: save-interval of 0 is invalid");
        exit(1);
    }

    create_dir_all(&session_path).expect(&format!("Failed to create session dir: {session_path}"));

    match mode {
        Mode::Default | Mode::LoadAndExit => load_session(&session_path, args.load_time, false, simulate),
        Mode::SaveAndExit | Mode::SaveOnly => save_session(&session_path, args.save_duplicate_pids),
    }?;

    if mode == Mode::LoadAndExit  {
        std::thread::sleep(time::Duration::from_secs(args.load_time + 5));
    }

    if mode == Mode::LoadAndExit || mode == Mode::SaveAndExit {
        exit(0);
    }

    loop {
        save_session(&session_path, args.save_duplicate_pids)?;
        thread::sleep(time::Duration::from_secs(save_interval));
    }
}