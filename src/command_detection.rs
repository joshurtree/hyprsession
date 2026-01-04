//use regex::Regex;
use std::fs;
use std::process::Command;
use hyprland::data::{Client, Clients};
use hyprland::shared::HyprData;
use std::collections::HashMap;

/// Check if a command exists in PATH using 'which'
pub fn command_exists_in_path(command: &str) -> bool {
    if command.is_empty() {
        return false;
    }
    
    Command::new("which")
        .arg(command.split_whitespace().next().unwrap())
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Extract the binary name from a command string
fn extract_binary_name(command: &str) -> String {
    let parts = command.split_whitespace();
    let binary = parts
        .clone()
        .next()
        .unwrap_or("")
        .split('/')
        .last()
        .unwrap_or("");

    format!("{} {}", binary, parts.skip(1).collect::<Vec<&str>>().join(" ")).trim().to_string()
}

fn handle_proc_cmdline(client: &Client) -> Result<String, std::io::Error> {
    let cmdline_path = format!("/proc/{}/cmdline", client.pid);
    if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
        let cleaned: String = cmdline
            .replace('\0', " ")
            .trim()
            .to_string();

        Ok(extract_binary_name(&cleaned))
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Cmdline not found"))
    }
}

fn handle_proc_exe(client: &Client) -> Result<String, std::io::Error> {
    let exe_path = format!("/proc/{}/exe", client.pid);
    if let Ok(exe_target) = fs::read_link(&exe_path) {
        if let Some(exe_name) = exe_target.file_name() {
            return Ok(exe_name.to_string_lossy().to_string());
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Exe not found"))
}

fn handle_initial_class(client: &Client) -> Result<String, std::io::Error> {
    Ok(client.initial_class.to_lowercase())
}

fn handle_initial_title(client: &Client) -> Result<String, std::io::Error> {
    Ok(client.initial_title.to_lowercase())
}

/// Fetch command for a Hyprland client using multiple detection methods
pub fn fetch_command(client: &Client, xdg_map: &HashMap<String, String>) -> Result<String, std::io::Error> {
    let handlers = vec![
        handle_proc_cmdline,
        handle_proc_exe,
        handle_initial_class,
        handle_initial_title,
    ];

    for handler in handlers {
        if let Ok(command) = handler(client) {
            if command_exists_in_path(&command) {
                return Ok(command);
            } else {
                // Fallback: check in xdg_map
                if let Some(xdg_command) = xdg_map.get(&command) {
                    return Ok(xdg_command.clone());
                }
            }
        }
    }

    // Fallback to cmdline even if not in PATH
    handle_proc_cmdline(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_binary_name() {
        assert_eq!(extract_binary_name("/usr/bin/firefox"), "firefox");
        assert_eq!(extract_binary_name("code"), "code");
        assert_eq!(extract_binary_name("/usr/bin/firefox --new-window"), "firefox --new-window");
        assert_eq!(extract_binary_name("/nix/store/.firefox-wrapped"), ".firefox-wrapped");
    }
    
    /*
    #[test]
    fn test_handle_nix_wrapped() {
        // Test .name-wrapped format
        let cmd = "/nix/store/.firefox-wrapped --new-instance";
        let result = handle_nix_wrapped(cmd).unwrap();
        assert_eq!(result, "firefox --new-instance");
        
        // Test hash-name-wrapped format  
        let cmd_no_args = "/nix/store/xyz789-code-wrapped";
        let result_no_args = handle_nix_wrapped(cmd_no_args).unwrap();
        assert_eq!(result_no_args, "code");
        
        // Test full hash-name-wrapped format with args
        let cmd_full = "/nix/store/abc123-firefox-wrapped --new-window --private";
        let result_full = handle_nix_wrapped(cmd_full).unwrap();
        assert_eq!(result_full, "firefox --new-window --private");
        
        // Test simple .name-wrapped without args
        let cmd_simple = "/nix/store/randomhash/.chromium-wrapped";
        let result_simple = handle_nix_wrapped(cmd_simple).unwrap();
        assert_eq!(result_simple, "chromium");
    }
    
    #[test]
    fn test_handle_flatpak() {
        let cmd = "flatpak run org.mozilla.firefox --new-window";
        let result = handle_flatpak(cmd).unwrap();
        assert_eq!(result, "flatpak run org.mozilla.firefox --new-window");
        
        let cmd_with_flags = "flatpak run --app-id=org.gimp.GIMP --file=test.jpg";
        let result_with_flags = handle_flatpak(cmd_with_flags).unwrap();
        assert_eq!(result_with_flags, "flatpak run --app-id=org.gimp.GIMP --file=test.jpg");
    }
    
    #[test]
    fn test_handle_appimage() {
        let cmd = "/home/user/Applications/Firefox.AppImage --profile test";
        let result = handle_appimage(cmd).unwrap();
        assert_eq!(result, "Firefox.AppImage");
    }
    
    #[test]
    fn test_handle_snap() {
        let cmd = "/snap/firefox/current/usr/lib/firefox/firefox --new-window";
        let result = handle_snap(cmd).unwrap();
        assert_eq!(result, "firefox");
    }
    */
    #[test]
    #[ignore] // Weird bug when running as a nix flake
    fn test_command_exists_in_path() {
        // These commands should typically exist on most Linux systems
        assert!(command_exists_in_path("ls"));
        assert!(command_exists_in_path("cat"));
        assert!(command_exists_in_path("which"));
        
        // This command should not exist
        assert!(!command_exists_in_path("definitely_not_a_real_command_123456"));
        assert!(!command_exists_in_path(""));
    }
}