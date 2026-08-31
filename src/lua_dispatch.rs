//! Dispatching through Hyprland's Lua configuration mode.
//!
//! Since Hyprland 0.55 a `hyprland.lua` config makes the IPC socket evaluate every
//! `dispatch <arg>` request as the Lua expression `hl.dispatch(<arg>)`. The classic
//! `dispatch exec [rules] command` strings emitted by the `hyprland` crate fail to
//! parse there, so this module builds the equivalent `hl.dsp.*` expressions and
//! sends them over the command socket directly.

use hyprland::data::Client;
use hyprland::error::HyprError;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::OnceLock;

const SPECIAL_WORKSPACE_ID: i32 = -99;

/// Whether the running Hyprland expects Lua dispatch expressions.
///
/// Probed once with the side-effect free `hl.dsp.no_op()`, which only parses when
/// the compositor runs a Lua config.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| dispatch("hl.dsp.no_op()").is_ok())
}

/// Runs `hl.dispatch(expr)` and fails unless Hyprland answers `ok`.
pub fn dispatch(expr: &str) -> hyprland::Result<()> {
    let response = send(&format!("dispatch {expr}"))?;
    if response.trim() == "ok" {
        Ok(())
    } else {
        Err(HyprError::NotOkDispatch(response))
    }
}

/// Translates one `exec.conf` line (`[rule;rule] command`) into `hl.dsp.exec_cmd(...)`.
pub fn exec_cmd(line: &str) -> String {
    let line = line.trim();
    let (rules, command) = match line.strip_prefix('[').and_then(|rest| rest.split_once(']')) {
        Some((rules, command)) => (rules, command.trim()),
        None => ("", line),
    };
    let rules: Vec<String> = rules
        .split(';')
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .map(exec_rule)
        .collect();

    if rules.is_empty() {
        format!("hl.dsp.exec_cmd({})", quote(command))
    } else {
        format!("hl.dsp.exec_cmd({}, {{{}}})", quote(command), rules.join(", "))
    }
}

/// Moves a window to a workspace without following it.
pub fn move_to_workspace(client: &Client, workspace_id: i32) -> String {
    format!(
        "hl.dsp.window.move({{workspace = {}, follow = false, window = {}}})",
        quote(&workspace_selector(workspace_id)),
        window(client)
    )
}

pub fn move_workspace_to_monitor(workspace_id: i32, monitor_id: impl std::fmt::Display) -> String {
    format!(
        "hl.dsp.workspace.move({{monitor = {monitor_id}, workspace = {}}})",
        quote(&workspace_selector(workspace_id))
    )
}

pub fn set_floating(client: &Client, floating: bool) -> String {
    format!("hl.dsp.window.float({{action = {}, window = {}}})", toggle(floating), window(client))
}

pub fn set_pinned(client: &Client, pinned: bool) -> String {
    format!("hl.dsp.window.pin({{action = {}, window = {}}})", toggle(pinned), window(client))
}

pub fn set_fullscreen(client: &Client, maximized: bool) -> String {
    let mode = if maximized { "maximized" } else { "fullscreen" };
    format!(
        "hl.dsp.window.fullscreen({{mode = \"{mode}\", action = \"set\", window = {}}})",
        window(client)
    )
}

/// Moves a window to an exact position.
pub fn move_to(client: &Client, (x, y): (i16, i16)) -> String {
    format!("hl.dsp.window.move({{x = {x}, y = {y}, window = {}}})", window(client))
}

fn exec_rule(rule: &str) -> String {
    let (name, value) = rule
        .split_once(' ')
        .map_or((rule, ""), |(name, value)| (name, value.trim()));
    // The rule effects kept their names in the Lua API except for this one.
    let name = match name {
        "fullscreenstate" => "fullscreen_state",
        other => other,
    };
    let value = if value.is_empty() {
        "true".to_string()
    } else if value.parse::<i64>().is_ok() {
        value.to_string()
    } else {
        quote(value)
    };
    format!("{name} = {value}")
}

fn window(client: &Client) -> String {
    quote(&format!("address:{}", client.address))
}

fn workspace_selector(id: i32) -> String {
    if id == SPECIAL_WORKSPACE_ID {
        "special".to_string()
    } else {
        id.to_string()
    }
}

fn toggle(on: bool) -> &'static str {
    if on { "\"on\"" } else { "\"off\"" }
}

fn quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn send(command: &str) -> hyprland::Result<String> {
    let mut stream = UnixStream::connect(socket_path()?)?;
    stream.write_all(command.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

fn socket_path() -> hyprland::Result<PathBuf> {
    let instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| HyprError::Other("HYPRLAND_INSTANCE_SIGNATURE is not set".to_string()))?;
    let base = std::env::var("XDG_RUNTIME_DIR")
        .map(|dir| PathBuf::from(dir).join("hypr"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/hypr"));
    Ok(base.join(instance).join(".socket.sock"))
}

#[cfg(test)]
mod tests {
    use super::exec_cmd;

    #[test]
    fn exec_line_with_rules_becomes_exec_cmd_table() {
        let line = "[monitor 0;workspace 2 silent;float;move 12 38;size 1512 910;pin;fullscreenstate 0] firefox";
        assert_eq!(
            exec_cmd(line),
            "hl.dsp.exec_cmd(\"firefox\", {monitor = 0, workspace = \"2 silent\", float = true, \
             move = \"12 38\", size = \"1512 910\", pin = true, fullscreen_state = 0})"
        );
    }

    #[test]
    fn exec_line_without_rules_has_no_table() {
        assert_eq!(exec_cmd("foot --working-directory=/home/me\n"), "hl.dsp.exec_cmd(\"foot --working-directory=/home/me\")");
    }

    #[test]
    fn special_workspace_and_quotes_survive() {
        let line = "[workspace special silent] sh -c \"echo \\\"hi\\\"\"";
        assert_eq!(
            exec_cmd(line),
            "hl.dsp.exec_cmd(\"sh -c \\\"echo \\\\\\\"hi\\\\\\\"\\\"\", {workspace = \"special silent\"})"
        );
    }
}
