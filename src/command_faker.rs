use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::collections::HashMap;

pub fn fake_command(name: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME")?;
    let bin_path = PathBuf::from(home_dir).join(".local/bin");
    
    // Create directory if it doesn't exist
    fs::create_dir_all(&bin_path)?;
        
    let file_path = bin_path.join(name);
    if !command.is_empty() {
        // Write the command to the file with shebang
        let content = format!("#!/bin/sh\n{}", command);
        fs::write(&file_path, content)?;

        // Make the file executable
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms)?;
    } else {
        // Remove the file if command is empty
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
    }
    Ok(())
}

/// Parse a .desktop file and extract the application name and command
fn parse_desktop_file(path: &PathBuf) -> Option<(String, String)> {
    let content = fs::read_to_string(path).ok()?;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut in_desktop_entry = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        } else if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }
        
        if !in_desktop_entry {
            continue;
        }
        
        if let Some(value) = line.strip_prefix("Name=") {
            name = Some(value.to_lowercase().to_string());
        } else if let Some(value) = line.strip_prefix("Exec=") {
            // Remove field codes like %f, %F, %u, %U, etc.
            let cleaned = value
                .split_whitespace()
                .filter(|s| !s.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ");
            exec = Some(cleaned);
        }
        
        // Stop early if we have both
        if name.is_some() && exec.is_some() {
            break;
        }
    }
    
    match (name, exec) {
        (Some(n), Some(e)) if !e.is_empty() => Some((n, e)),
        _ => None,
    }
}

/// Get XDG desktop file directories in priority order
fn get_desktop_file_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    
    // User-specific directory (highest priority)
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data_home).join("applications"));
    } else if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }
    
    // System directories
    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            if !dir.is_empty() {
                dirs.push(PathBuf::from(dir).join("applications"));
            }
        }
    } else {
        // Default system directories
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }
    
    dirs
}

/// Create a map between application names and their commands from XDG .desktop files
/// Returns a HashMap where keys are application names and values are the commands to run them
pub fn build_xdg_command_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    for dir in get_desktop_file_dirs() {
        if !dir.exists() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Only process .desktop files
                if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                    continue;
                }
                
                if let Some((name, command)) = parse_desktop_file(&path) {
                    // Only add if not already present (respects priority order)
                    map.entry(name).or_insert(command);
                }
            }
        }
    }
    
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_fake_command() {
        let result = fake_command("firefox", "flatpak run org.mozilla.firefox");
        assert!(result.is_ok());
        assert!(std::path::PathBuf::from(std::env::var("HOME").unwrap())
            .join(".local/bin/firefox")
            .exists());
        let cleanup = fake_command("firefox", "");
        assert!(cleanup.is_ok());
        assert!(!std::path::PathBuf::from(std::env::var("HOME").unwrap())
            .join(".local/bin/firefox")
            .exists());
    }

    #[test]
    fn test_build_xdg_command_map() {
        let commands = build_xdg_command_map();
        
        // The map should not be empty on most systems
        // We can't test specific applications since they vary by system
        // But we can test the structure
        for (name, command) in commands.iter().take(5) {
            println!("Found: {} -> {}", name, command);
            assert!(!name.is_empty());
            assert!(!command.is_empty());
        }
    }

    #[test]
    fn test_parse_desktop_file() {
        // Create a temporary .desktop file for testing
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test.desktop");
        
        let content = r#"[Desktop Entry]
Name=Test Application
Exec=test-app --flag %f
Type=Application
Icon=test-icon
"#;
        fs::write(&test_file, content).unwrap();
        
        let result = parse_desktop_file(&test_file);
        assert!(result.is_some());
        
        let (name, command) = result.unwrap();
        assert_eq!(name, "test application");
        assert_eq!(command, "test-app --flag");
        
        // Cleanup
        fs::remove_file(&test_file).unwrap();
    }
}