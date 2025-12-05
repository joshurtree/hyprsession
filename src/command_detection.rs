use regex::Regex;
use std::fs;
use std::process::Command;
use hyprland::data::Client;

/// Extract the binary name from a command string
pub fn extract_binary_name(command: &str) -> &str {
    command
        .split_whitespace()
        .next()
        .unwrap_or("")
        .split('/')
        .last()
        .unwrap_or("")
}

/// Handle NixOS wrapped commands
pub fn handle_nix_wrapped(command: &str) -> Option<String> {
    // Handle both formats: /nix/store/hash-name-wrapped and .name-wrapped
    let re = Regex::new(r"(?:/nix/store/[^/]*-|\.?)([^/\s]+)-wrapped(.*)").ok()?;
    let caps = re.captures(command)?;
    let unwrapped = caps.get(1)?.as_str();
    let args = caps.get(2).map_or("", |m| m.as_str());
    Some(if args.is_empty() {
        unwrapped.to_string()
    } else {
        format!("{}{}", unwrapped, args)
    })
}

/// Handle Flatpak applications
pub fn handle_flatpak(command: &str) -> Option<String> {
    if !command.contains("flatpak") {
        return None;
    }
    
    // Look for --app-id= first, then fall back to positional argument
    if let Ok(re) = Regex::new(r"--app-id=([^\s]+)") {
        if let Some(caps) = re.captures(command) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    
    // Fall back to first non-option argument after 'flatpak run'
    let re = Regex::new(r"flatpak\s+run\s+(?:--[^\s]+=?[^\s]*\s+)*([a-zA-Z0-9._-]+(?:\.[a-zA-Z0-9._-]+)+)").ok()?;
    re.captures(command)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Handle AppImage applications
pub fn handle_appimage(command: &str) -> Option<String> {
    let re = Regex::new(r"([^/\s]+\.AppImage)").ok()?;
    re.captures(command)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Handle Snap applications
pub fn handle_snap(command: &str) -> Option<String> {
    if !command.contains("/snap/") {
        return None;
    }
    
    let re = Regex::new(r"/snap/([^/]+)").ok()?;
    re.captures(command)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Check if a command exists in PATH using 'which'
pub fn command_exists_in_path(command: &str) -> bool {
    if command.is_empty() {
        return false;
    }
    
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Handle special command cases (NixOS wrapped, Flatpak, AppImage, etc.)
pub fn handle_special_commands(command: &str) -> String {
    // Handle NixOS wrapped commands
    if let Some(unwrapped) = handle_nix_wrapped(command) {
        return unwrapped;
    }
    
    // Handle Flatpak applications
    if let Some(flatpak_app) = handle_flatpak(command) {
        return flatpak_app;
    }
    
    // Handle AppImage applications  
    if let Some(appimage_name) = handle_appimage(command) {
        return appimage_name;
    }
    
    // Handle Snap applications
    if let Some(snap_name) = handle_snap(command) {
        return snap_name;
    }
    
    // Return original command if no special handling needed
    command.to_string()
}

/// Fetch command for a Hyprland client using multiple detection methods
pub fn fetch_command(client: &Client) -> String {
    // Method 1: Try /proc/PID/cmdline (most reliable)
    let cmdline_path = format!("/proc/{}/cmdline", client.pid);
    if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
        let cleaned: String = cmdline
            .replace('\0', " ")
            .trim()
            .to_string();
            
        if !cleaned.is_empty() {
            let processed = handle_special_commands(&cleaned);
            let binary_name = extract_binary_name(&processed);
            
            // Validate with 'which' if it looks like a simple command
            if !binary_name.contains('/') && command_exists_in_path(binary_name) {
                return processed;
            } else if !processed.is_empty() {
                return processed;
            }
        }
    }
    
    // Method 2: Try /proc/PID/exe (executable path fallback)
    let exe_path = format!("/proc/{}/exe", client.pid);
    if let Ok(exe_target) = fs::read_link(&exe_path) {
        if let Some(exe_name) = exe_target.file_name() {
            let exe_string = exe_name.to_string_lossy().to_string();
            let processed = handle_special_commands(&exe_string);
            
            // Validate with 'which'
            if command_exists_in_path(&processed) {
                return processed;
            } else if !processed.is_empty() {
                return processed;
            }
        }
    }
    
    // Method 3: Fall back to client.class (final fallback)
    if !client.class.is_empty() {
        return client.class.to_lowercase();
    }
    
    // Last resort: use client title
    if !client.title.is_empty() {
        return client.title.to_lowercase();
    }
    
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_binary_name() {
        assert_eq!(extract_binary_name("firefox --new-window"), "firefox");
        assert_eq!(extract_binary_name("/usr/bin/firefox"), "firefox");
        assert_eq!(extract_binary_name("/nix/store/.firefox-wrapped"), ".firefox-wrapped");
        assert_eq!(extract_binary_name("code"), "code");
    }
    
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
        let cmd_simple = ".chromium-wrapped";
        let result_simple = handle_nix_wrapped(cmd_simple).unwrap();
        assert_eq!(result_simple, "chromium");
    }
    
    #[test]
    fn test_handle_flatpak() {
        let cmd = "flatpak run org.mozilla.firefox --new-window";
        let result = handle_flatpak(cmd).unwrap();
        assert_eq!(result, "org.mozilla.firefox");
        
        let cmd_with_flags = "flatpak run --app-id=org.gimp.GIMP --file=test.jpg";
        let result_with_flags = handle_flatpak(cmd_with_flags).unwrap();
        assert_eq!(result_with_flags, "org.gimp.GIMP");
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
    
    #[test]
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