use std::{thread, time};
use std::fs::create_dir_all;
use std::path::Path;
use std::process::exit;
//use serde::Deserialize;
use clap::{Parser, ValueEnum, Subcommand};
use log::LevelFilter;
use simplelog::{ColorChoice, TermLogger, TerminalMode };

mod session;
mod config;

use crate::session::*;
use crate::config::*;

#[derive(Copy, Clone, PartialEq, ValueEnum, Subcommand)]
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
    #[command(subcommand)]
    command: Option<Mode>,

    /// Depreciated - Use command instead
    #[arg(short, long)]
    mode: Option<Mode>,

    /// Interval between saving sessions (default: 60)
    #[arg(short='i', long)]
    save_interval: Option<u64>,
    
    /// The path where the session is saved (default: ~/.local/share/hyprsession)
    #[arg(short, long)]
    session_path: Option<String>,

    /// The path where the config file is held (default: ~/.config/hypr/hyprsession.yaml)
    #[arg(short, long)]
    config_path: Option<String>,

    /// Only simulate calls to Hyprland (supresses loading of session)
    #[arg(long)]
    simulate: bool,

    #[arg(short, long)]
    verbose: bool
}   

fn main() {
    let args = Args::parse();
    let term_log_level = if args.verbose { LevelFilter::Trace } else { LevelFilter::Error };

    TermLogger::init(term_log_level, 
                    simplelog::Config::default(), 
                    TerminalMode::Mixed, 
                    ColorChoice::Auto);

    let mode = args.command.unwrap_or(args.mode.unwrap_or(Mode::Default)) ;
    let simulate = args.simulate;
    let config_path = 
        args.config_path.unwrap_or(std::env::var("HOME").unwrap() + "/.config/hypr/hyprsession.yaml");
    let conf = load_config(&config_path);
    let default_path = args.session_path.unwrap_or(conf.session_path);
    let session_path = Path::new(&default_path);
    let save_interval = args.save_interval.unwrap_or(conf.save_interval);


    if save_interval < 1 {
        log::error!("Save interval needs to be a positive integer");
        exit(-1);
    }

    if !create_dir_all(&session_path).is_ok() {
        log::error!("Failed to create session directory: {:?}", session_path);
        exit(-1);
    }

    let do_save_session = 
        || save_session(&session_path, &conf.apps)
            .map_err(|err| log::error!("Session write error: {}", err))
            .unwrap();
    match mode {
        Mode::Default | Mode::LoadAndExit =>
            load_session(&session_path, simulate)
                .map_err(|err| log::error!("Session load error: {}", err))
                .unwrap(),
        Mode::SaveAndExit | Mode::SaveOnly => do_save_session()
    } 

    if mode == Mode::LoadAndExit || mode == Mode::SaveAndExit {
        exit(0);
    }

    loop {
        do_save_session();
        thread::sleep(time::Duration::from_secs(save_interval));
    }
}
    