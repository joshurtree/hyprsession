use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

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
}