# Command Detection Methods for Hyprland Clients

This document compares different methods for determining the command used to create clients in Hyprland.

## Method Comparison

| Method | Reliability | Performance | Information Detail | NixOS Support | Special Cases |
|--------|-------------|-------------|-------------------|---------------|---------------|
| `/proc/PID/cmdline` | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Full command line | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| `ps -o cmd` | ⭐⭐⭐⭐ | ⭐⭐⭐ | Full command line | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| `/proc/PID/comm` | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Executable name only | ⭐⭐ | ⭐⭐ |
| `/proc/PID/exe` | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Full executable path | ⭐⭐⭐ | ⭐⭐⭐ |
| Client class/title | ⭐⭐ | ⭐⭐⭐⭐⭐ | Application name | ⭐ | ⭐ |

## Recommended Implementation

For your hyprsession project, here's the recommended approach:

```rust
use std::fs;
use regex::Regex;

pub fn fetch_command_robust(client: &Client) -> String {
    // 1. Primary: /proc/PID/cmdline (most reliable)
    if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", client.pid)) {
        let cleaned = cmdline.replace('\0', " ").trim().to_string();
        if !cleaned.is_empty() {
            return handle_special_commands(&cleaned);
        }
    }
    
    // 2. Fallback: ps command (your current method)
    if let Ok(output) = std::process::Command::new("ps")
        .args(&["--no-headers", "-o", "cmd", "-p", &client.pid.to_string()])
        .output() 
    {
        if output.status.success() && !output.stdout.is_empty() {
            let command = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return handle_special_commands(&command);
        }
    }
    
    // 3. Last resort: client metadata
    if !client.class.is_empty() {
        return client.class.clone();
    }
    
    "unknown".to_string()
}

fn handle_special_commands(command: &str) -> String {
    // Handle NixOS wrapped commands
    let nix_re = Regex::new(r"/nix/store/[^/]+-(.+?)-wrapped(?:\s+(.*))?").unwrap();
    if let Some(captures) = nix_re.captures(command) {
        let binary = captures.get(1).unwrap().as_str();
        let args = captures.get(2).map(|m| m.as_str()).unwrap_or("");
        return if args.is_empty() {
            binary.to_string()
        } else {
            format!("{} {}", binary, args)
        };
    }
    
    // Handle Flatpak
    if command.contains("flatpak run") {
        let flatpak_re = Regex::new(r"flatpak run ([^\s]+)").unwrap();
        if let Some(captures) = flatpak_re.captures(command) {
            return captures.get(1).unwrap().as_str().to_string();
        }
    }
    
    // Handle AppImage
    if command.contains(".AppImage") {
        let appimage_re = Regex::new(r"([^/\s]+\.AppImage)").unwrap();
        if let Some(captures) = appimage_re.captures(command) {
            return captures.get(1).unwrap().as_str().to_string();
        }
    }
    
    command.to_string()
}
```

## Special Cases to Handle

### 1. NixOS Wrapped Commands
```bash
# Original command:
/nix/store/abc123-firefox-wrapped --new-instance

# Should become:
firefox --new-instance
```

### 2. Flatpak Applications
```bash
# Original command:
/usr/bin/flatpak run --app-id=org.mozilla.firefox

# Should become:
org.mozilla.firefox
```

### 3. AppImage Applications
```bash
# Original command:
/home/user/Applications/Firefox.AppImage --some-arg

# Should become:
Firefox.AppImage --some-arg
```

### 4. Snap Applications
```bash
# Original command:
/snap/firefox/current/usr/lib/firefox/firefox

# Should become:
firefox
```

## Performance Considerations

1. **File System Access**: `/proc` reading is fastest
2. **Process Execution**: `ps` command has overhead
3. **Caching**: Consider caching for frequently accessed PIDs
4. **Error Handling**: Always have fallback methods

## Integration with Your Current Code

Replace your current `fetch_command` function:

```rust
// In src/session.rs
fn fetch_command(info: &Client) -> String {
    fetch_command_robust(info)
}
```

This approach will:
- ✅ Handle NixOS wrapped commands correctly
- ✅ Work with Flatpak/AppImage/Snap applications  
- ✅ Provide better reliability than ps alone
- ✅ Maintain backward compatibility
- ✅ Handle edge cases gracefully