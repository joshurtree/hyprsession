use std::fs::create_dir_all;
use std::process::exit;
use std::{env, thread, time};
//use serde::Deserialize;
use clap::{Parser, ValueEnum};

pub mod command_detection;
pub mod command_faker;
pub mod legacy;
pub mod session;

use crate::session::*;
use crate::command_faker::fake_command;
use crate::command_detection::command_exists_in_path;

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

    /// Create a command to fake applications
    Command,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Which mode to run the program in (default: default)
    #[arg(value_enum, default_value_t = Mode::Default)]
    mode: Mode,

    /// Name of the session to switch to or delete
    #[arg(default_value_t = String::from("default"))]
    name: String,

    /// Command to run (for Command mode)
    #[arg(default_value_t = String::from(""))]
    command: String,

    /// Interval between saving sessions (default: 60)
    #[arg(short = 'i', long, default_value_t = 60)]
    save_interval: u64,

    /// Time to wait before considering the session loaded (default: 60)
    #[arg(short = 'l', long, default_value_t = 60)]
    load_time: u64,

    /// Only simulate calls to Hyprland (supresses loading of session)
    #[arg(long, default_value_t = false)]
    simulate: bool,

    /// Only adjust exiting clients without launching new programs
    #[arg(long, default_value_t = false)]
    adjust_clients_only: bool,
}

fn migration_check(session_path: &str) {
    let old_session = std::path::Path::new(session_path).join("exec.conf");
    let new_session = std::path::Path::new(session_path).join("default");
    if old_session.exists() {
        if std::path::Path::new(session_path).join("exec.conf").exists() && !new_session.join("exec.conf").exists() {
            println!("Migrating session data from {} to {}", old_session.display(), new_session.display());
            std::fs::create_dir_all(&new_session).unwrap();
            std::fs::rename(old_session, new_session.join("exec.conf")).unwrap();
            std::fs::write(new_session.join("clients.json"), "[]").unwrap();
        }
    }
}

fn main() -> hyprland::Result<()> {
    if std::env::args().any(|arg| arg == "--mode") {
        eprintln!("Warning: '--mode' argument is deprecated. Please consult documentation for updated usage.");
        crate::legacy::main()?;
        exit(0);
    }

    let args = Args::parse();
    let session_path = if env::var("HYPRSESSION_PATH").is_ok() {
        env::var("HYPRSESSION_PATH").unwrap()
    } else {
        env::var("HOME").unwrap() + "/.local/share/hyprsession"
    };

    println!("Using session path: {}", session_path);
    create_dir_all(&session_path).expect(&format!("Failed to create session dir: {session_path}"));
    migration_check(&session_path);

    if args.save_interval == 0 {
        eprintln!("Save interval needs to be greater than 0");
        exit(1);
    }

    let session = LocalSession {
        base_path: session_path.clone(),
        load_time: args.load_time,
        adjust_clients_only: args.adjust_clients_only,
        simulate: args.simulate,
        save_duplicate_pids: false,
    };

    match args.mode {
        Mode::Clear => {
            session.clear()?;
        }
        Mode::Default | Mode::Load => {
            session.load(&args.name)?;
        }
        Mode::List => {
            println!("Available sessions:");
            let mut has_sessions = false;
            for session_name in session.list() {
                println!(" - {}", session_name);
                has_sessions = true;
            }
            if !has_sessions {
                println!("(No sessions found)");
            }
        }
        Mode::Delete => {
            session.delete(&args.name);
        }
        Mode::Save => {
            session.save(&args.name)?;
        }
        Mode::Command => {
            let command_name = args.name.clone();
            if command_name.is_empty() {
                eprintln!("Error: Command name cannot be empty");
                exit(1);
            }

            if command_exists_in_path(&command_name) {
                eprintln!("Command '{}' already exists in PATH.", command_name);
                exit(1);
            }

            match fake_command(&command_name, &args.command.clone()) {
                Ok(_) => {
                    println!("Fake command '{}' created successfully.", command_name);
                }
                Err(e) => {
                    eprintln!("Failed to create command '{}': {}", command_name, e);
                    exit(1);
                }
            }
        }
    }

    if args.mode != Mode::Default {
        exit(0);
    }

    loop {
        thread::sleep(time::Duration::from_secs(args.save_interval));
        session.save(&args.name)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_check() {
        let temp_dir = tempfile::tempdir().unwrap();
        let session_path = temp_dir.path().to_str().unwrap();

        // Create the old session file
        std::fs::write(temp_dir.path().join("exec.conf"), "test").unwrap();

        migration_check(session_path);

        // Check that the old session file was moved
        assert!(!temp_dir.path().join("exec.conf").exists());
        assert!(temp_dir.path().join("default").join("exec.conf").exists());
    }
}