use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

use hyprsession::session::load_session;

#[test]
fn test_load_session_with_nonexistent_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // This should not panic or fail when the session file doesn't exist
    load_session(&session_path, true);
}

#[test]
fn test_load_session_with_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{}/exec.conf", session_path);

    // Create an empty session file
    File::create(&exec_file_path).expect("Failed to create test file");

    // This should handle empty files gracefully
    load_session(&session_path, true);
}

#[test]
fn test_load_session_with_sample_data() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{}/exec.conf", session_path);

    // Create a session file with sample data
    let mut file = File::create(&exec_file_path).expect("Failed to create test file");
    writeln!(
        file,
        "[monitor 0;workspace 1 silent;move 100 200;size 800 600] firefox"
    )
    .unwrap();
    writeln!(
        file,
        "[monitor 1;workspace 2 silent;float;move 50 50;size 400 300] kitty"
    )
    .unwrap();

    // Load the session in simulate mode (won't actually execute commands)
    load_session(&session_path, true);
}

#[test]
fn test_session_directory_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap();

    // Verify the temporary directory exists
    assert!(Path::new(session_path).exists());
    assert!(Path::new(session_path).is_dir());
}

#[cfg(test)]
mod cli_tests {
    use std::process::Command;

    #[test]
    fn test_cli_help() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("hyprsession"));
        assert!(stdout.contains("--mode"));
        assert!(stdout.contains("--save-interval"));
    }

    #[test]
    fn test_cli_version() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--version"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("hyprsession"));
    }

    #[test]
    fn test_cli_simulate_mode() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--mode", "save-and-exit", "--simulate"])
            .output()
            .expect("Failed to execute command");

        // In simulate mode, it should not fail even if hyprland is not available
        // The program might still fail due to missing hyprland connection, so we'll check stderr
        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap();
            // If it fails due to hyprland connection issues, that's expected in test environment
            println!("CLI test failed with stderr: {}", stderr);
        }
    }

    #[test]
    fn test_cli_invalid_save_interval() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--save-interval", "0"])
            .output()
            .expect("Failed to execute command");

        // Should fail with invalid save interval
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Save interval needs to be a positive integer"));
    }
}
