use hyprland::data::{Client, Clients, FullscreenMode};
use hyprland::{dispatch, dispatch::* };
use hyprland::event_listener::EventListener;
use hyprland::prelude::*;
use hyprland::shared::Address;
use std::fs::File;
use std::io::{read_to_string, Write};
use crate::command_detection::fetch_command;
use std::path::PathBuf;

const EXEC_NAME: &str = "exec.conf";
const CLIENTS_PATH_NAME: &str = "clients.json";

pub trait Session {
    fn save(&self, name: &str) -> hyprland::Result<()>;
    fn load(&self, name: &str) -> hyprland::Result<()>;
    fn clear(&self) -> hyprland::Result<()>;
    fn list(&self) -> impl Iterator<Item=String>;
    fn delete(&self, name: &str);
}

macro_rules! iif {
    ($prop:expr, $val:expr) => {
        if $prop { $val } else { "" }
    };
    ($prop:expr, $val:expr, $alt:expr) => {
        if $prop { $val } else { $alt }
    };
}

macro_rules! to_base_dir {
    ($base_path:expr, $name:expr) => {
        if $name.is_empty() {
            PathBuf::from($base_path)
        } else {
            [$base_path, $name].iter().collect::<PathBuf>()
        }
    };
}

pub struct LocalSession {
    pub base_path: String,
    pub simulate: bool,
    pub load_time:  u64, 
    pub adjust_clients_only: bool,
    pub save_duplicate_pids: bool
}

struct ClientAdjustmentContext<'a> {
    real_client: &'a Client,
    session_client: &'a Client,
    simulate: bool,
}

trait CheckAndAdjust {
    fn check<T: PartialEq>(&self, param: fn(&Client) -> T) -> bool;
    fn check_and_adjust<T: PartialEq>(&self, param: fn(&Client) -> T,  action: impl Fn() -> hyprland::Result<()>, msg: &str, fail_msg: &str);
    fn warning(&self, msg: &str);
}


impl CheckAndAdjust for ClientAdjustmentContext<'_> {
    fn check<T: PartialEq>(&self, param: fn(&Client) -> T) -> bool {
        param(self.real_client) != param(self.session_client)
    }

    fn check_and_adjust<T: PartialEq>(&self, param: fn(&Client) -> T,  action: impl Fn() -> hyprland::Result<()>, msg: &str, fail_msg: &str) {
        if self.check(param) {
            println!("{}", msg);
            if !self.simulate {
                action().unwrap_or_else(|_| {
                    self.warning(fail_msg);
                });
            }
        }
    }

    fn warning(&self, msg: &str) {
        println!("Warning: {}: {}", msg, self.real_client.title);
    }
}

fn adjust_client(real_client: &Client, session_client: &Client, simulate: bool) {
    let context = ClientAdjustmentContext {
        real_client: real_client,
        session_client: session_client,
        simulate,
    };

    let client_addr = WindowIdentifier::Address(context.real_client.address.clone());
    context.check_and_adjust(
        |c| c.workspace.id,
        || dispatch!(
            MoveToWorkspaceSilent, 
            WorkspaceIdentifierWithSpecial::Id(real_client.workspace.id),
            Some(client_addr.clone())
        ), 
        format!("Moving {} to workspace {}", real_client.title, session_client.workspace.id).as_str(),
        format!("Failed to move client to workspace {}", real_client.title).as_str()
    );
    context.check_and_adjust(
        |c| c.monitor.unwrap_or(0),
        || dispatch!(
            MoveWorkspaceToMonitor, 
            WorkspaceIdentifier::Id(session_client.workspace.id), 
            MonitorIdentifier::Id(session_client.monitor.unwrap_or(0))
        ), 
        format!("Moving {} to monitor {}", real_client.title, session_client.monitor.unwrap_or(0)).as_str(),
        format!("Failed to move client to monitor {}", real_client.title).as_str()
    );
    context.check_and_adjust(
        |c| c.floating,
        || dispatch!(ToggleFloating, Some(client_addr.clone())),
        format!("Toggling floating for client {}", real_client.title).as_str(),
        format!("Failed to toggle floating for client {}", real_client.title).as_str()
    );
    context.check_and_adjust(
        |c| c.pinned,
        || dispatch!(TogglePinWindow, client_addr.clone()),
        format!("Pinning client window {}", real_client.title).as_str(),
        format!("Failed to pin client window {}", real_client.title).as_str()
    );
    context.check_and_adjust(
        |c| c.fullscreen,
        || {        
            dispatch!(FocusWindow, client_addr.clone()).unwrap_or_else(|_| {
                println!("Warning: Failed to focus client window: {}", real_client.title);
            });
            dispatch!(
                ToggleFullscreen,
                if session_client.fullscreen == FullscreenMode::Maximized { FullscreenType::Maximize } else { FullscreenType::Real }
            )
        },
        format!("Toggling fullscreen for client {}", real_client.title).as_str(),
        format!("Failed to toggle fullscreen for client {}", real_client.title).as_str()
    );

    if session_client.fullscreen == FullscreenMode::None {
        println!("Moving client: {}", real_client.title);
        if !context.simulate {
            hyprland::dispatch!(
                MoveWindowPixel,
                Position::Exact(session_client.at.0, session_client.at.1),
                WindowIdentifier::Address(real_client.address.clone())
            ).unwrap_or_else(|_| {
                println!("Warning: Failed to move client window: {:?}", real_client.title);
            });
        }
            // hyprland::dispatch!(
        //     ResizeWindowPixel,
        //     Position::Exact(session_client.size.0, session_client.size.1),
        //     WindowIdentifier::Address(real_client.address.clone())
        // ).unwrap_or_else(|_| {
        //     println!("Warning: Failed to resize client window: {:?}", real_client.title);
        // });
    }
}

fn process_window_event(address: Address, clients_data: &'static str, start_time: std::time::Instant, load_time: u64, simulate: bool) {
    let clients: Vec<Client> = serde_json::from_str(clients_data)
        .expect("Failed to parse clients data");

    if start_time.elapsed().as_secs() > load_time { 
        println!("Load time exceeded, skipping client adjustments");
        return;
    }

    for session_client in clients.iter() {
        if let Some(real_client) = Clients::get().expect("Unable to fetch clients")
                                                        .iter()
                                                        .find(|c| c.address == address) { 
            println!("Adjusting client: {:?}", real_client.title);
            adjust_client(real_client, session_client, simulate);
        } else {
            println!("Client '{:?}' not found - skipping", address);
        }
    } 
}

fn load_programs(base_path: &PathBuf, simulate: bool) -> hyprland::Result<()> {
    let session_file = File::open(base_path.join(EXEC_NAME));

    if session_file.is_ok() {
        for line in read_to_string(session_file.unwrap()).unwrap().lines() {
            if !simulate {
                hyprland::dispatch!(Exec, line)?;
            }
            println!("Sending: dispatch exec {line}");
        }
    }

    Ok(())
}

impl Session for LocalSession {
    fn save(&self, name: &str) -> hyprland::Result<()> {
        println!("Saving session: {}", name);
        let base_dir: PathBuf = to_base_dir!(self.base_path.clone(), name.to_string());
        std::fs::create_dir_all(&base_dir).expect("Failed to create session directory");

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
        ];

        let client_info = Clients::get().expect("Unable to fetch clients");

        let mut exec_file = File::create(base_dir.join(EXEC_NAME))
            .expect("Failed to create session file");
        let clients_file = File::create(base_dir.join(CLIENTS_PATH_NAME))
            .expect("Failed to create clients file");
        let mut pids: Vec<i32> = vec![];
        let mut saved_clients: Vec<Client> = vec![];

        for info in client_info.iter().rev() {
            saved_clients.push(info.clone());
            if !self.save_duplicate_pids && pids.contains(&info.pid) {
                continue;
            }
            let cmd = fetch_command(info);
            if !cmd.is_ok() {
                continue;
            }
            pids.push(info.pid);

            let exec_opts: Vec<String> = props
                .iter()
                .map(|opt| opt(info))
                .filter(|opt| !opt.is_empty())
                .collect();
            exec_file
                .write_all(format!("[{}] {}\n", exec_opts.join(";"), cmd.unwrap()).as_bytes())?;
        }

        serde_json::to_writer(&clients_file, &saved_clients)
            .expect("Failed to write to clients file");
        println!("Session saved");
        Ok(())
    }

    fn load(&self, name: &str) -> hyprland::Result<()> {
        println!("Loading session: {}", name);
        let base_dir: PathBuf = to_base_dir!(self.base_path.clone(), name.to_string());
        let start_time = std::time::Instant::now();

        if !self.adjust_clients_only {
            self.clear()?;
            load_programs(&base_dir.clone(), self.simulate)?;
        }

        // Clone the values we need inside the thread before moving into the closure
        let load_time = self.load_time;
        let simulate = self.simulate;
        
        std::thread::spawn(move || {
            let clients_file_path = base_dir.join(CLIENTS_PATH_NAME);
            let clients_data = std::fs::read_to_string(&clients_file_path)
                .unwrap_or_else(|_| "[]".to_string());
            let clients_data: &'static str = Box::leak(clients_data.into_boxed_str());

            let mut event_listener = EventListener::new();
            event_listener.add_window_title_changed_handler({move |event| {
                process_window_event(event.address, clients_data, start_time, load_time, simulate);
            }});
            event_listener.add_window_opened_handler({move |event| {
                process_window_event(event.window_address, clients_data, start_time, load_time, simulate);
            }});
            let _ = event_listener.start_listener();
        });

        std::thread::sleep(std::time::Duration::from_secs(self.load_time + 1));
        println!("Finished loading session");
        Ok(())
    }

    fn clear(&self) -> hyprland::Result<()> {
        if !self.simulate {
            let pids: Vec<i32> = Clients::get()?
                .iter()
                .map(|c| c.pid)
                .collect();
            for pid in pids {
                let _ = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .output()
                    .unwrap();
            }

            loop {
                if Clients::get()?.iter().collect::<Vec<&Client>>().is_empty() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        println!("Cleared existing session");
        Ok(())
    }

    fn list(&self) -> impl Iterator<Item=String> {
        let paths = std::fs::read_dir(self.base_path.clone()).unwrap();

        paths.filter_map(|path| {
            let entry = path.unwrap();
            if entry.path().is_dir() {
                Some(entry.file_name())
            } else {
                None
            }
        }).map(|os_str| os_str.to_string_lossy().into_owned())
    }

    fn delete(&self, name: &str) {
        let full_path = format!("{}/{}", self.base_path, name);
        if std::fs::remove_dir_all(&full_path).is_ok() {
            println!("Deleted session: {}", name);
        } else {
            println!("Failed to delete session: {}", name);
        }
    }
}