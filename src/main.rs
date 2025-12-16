use std::fs::create_dir_all;
use std::process::exit;
use std::{env, thread, time};
//use serde::Deserialize;
use clap::{Parser, ValueEnum};

pub mod session;
pub mod command_detection;
pub mod legacy;

use crate::session::*;

#[derive(Copy, Clone, PartialEq, ValueEnum)]
enum Mode {
    /// Default mode: Load session and then save the session at intervals
    Default,

    /// Save the session
    Save,

    /// List available sessions
    List,

    /// Load a session then exit
    Load,

    /// Clear the current session
    Clear,

    /// Delete a session
    Delete,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Which mode to run the program in (default: default)
    #[arg(value_enum, default_value_t = Mode::Default)]
    mode: Mode,

    /// Name of the session to switch to or delete
    #[arg(default_value_t = String::from(""))]
    name: String,

    /// Whether to store multiple clients owned by the same application
    #[arg(long, default_value_t = false)]
    save_duplicate_pids: bool,

    /// Interval between saving sessions (default: 60)
    #[arg(short = 'i', long, default_value_t = 60)]
    save_interval: u64,

    /// Time to wait before considering the session loaded (default: 30)
    #[arg(short = 'l', long, default_value_t = 30)]
    load_time: u64,

    /// Only simulate calls to Hyprland (supresses loading of session)
    #[arg(long, default_value_t = false)]
    simulate: bool,

    /// Only adjust exiting clients without launching new programs
    #[arg(long, default_value_t = false)]
    adjust_clients_only: bool,

    /// Number of times to save the session before exiting
    #[arg(long, default_value_t = 0)]
    save_count: u32,
}


fn main() {
    if std::env::args().any(|arg| arg == "--mode") {
        eprintln!("Warning: '--mode' argument is deprecated. Please consult documentation for updated usage.");
        crate::legacy::main();
        return;
    }

    let args = Args::parse();
    let mode = args.mode;
    let session_path = if env::var("HYPRSESSION_PATH").is_ok() {
        env::var("HYPRSESSION_PATH").unwrap()
    } else {
        env::var("HOME").unwrap() + "/.local/share/hyprsession"
    };

    create_dir_all(&session_path).expect(&format!("Failed to create session dir: {session_path}"));

    if args.save_interval == 0 {
        eprintln!("Save interval needs to be greater than 0");
        exit(1);
    }

    match mode {
        Mode::Default | Mode::Load | Mode::Clear => {
            clear_session(args.simulate);
            if mode != Mode::Clear {
                load_session(&session_path, &args.name, args.load_time, args.adjust_clients_only, args.simulate).expect("Failed to load session");
            }
        }
        Mode::List => {
            list_sessions(&session_path);
        }
        Mode::Delete => {
            delete_session(&session_path, &args.name);
        }
        Mode::Save => {
            save_session(&session_path, &args.name, args.save_duplicate_pids);
        }
    }

    if mode != Mode::Default {
        exit(0);
    }

    for _ in 0..(if args.save_count == 0 { 99999 } else { args.save_count }) {
        thread::sleep(time::Duration::from_secs(args.save_interval));
        save_session(&session_path, &args.name, args.save_duplicate_pids);
    }
}
