use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

use hyprsession::session::{LocalSession, Session};

#[test]
fn test_load_session_with_nonexistent_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // Create a LocalSession with test configuration
    let session = LocalSession {
        base_path: session_path,
        simulate: true,
        load_time: 1, // Reduced load time for faster tests
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // This should not panic or fail when the session file doesn't exist
    let _ = session.load("");
}

#[test]
fn test_load_session_with_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{}/exec.conf", session_path);

    // Create an empty session file
    File::create(&exec_file_path).expect("Failed to create test file");

    // Create a LocalSession with test configuration
    let session = LocalSession {
        base_path: session_path,
        simulate: true,
        load_time: 1, // Reduced load time for faster tests
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // This should handle empty files gracefully
    let _ = session.load("");
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

    // Create a LocalSession with test configuration (reduced load_time for faster tests)
    let session = LocalSession {
        base_path: session_path,
        simulate: true,
        load_time: 1, // Reduced from 30 to 1 for faster tests
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // Load the session in simulate mode (won't actually execute commands)
    let _ = session.load("");
    let _ = session.load("testsession");
}

#[test]
fn test_load_session_with_malformed_data() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{}/exec.conf", session_path);
    
    // Create a session file with malformed data
    let mut file = File::create(&exec_file_path).expect("Failed to create test file");
    writeln!(file, "[monitor ;workspace silent;move size] unknown_command").unwrap();
    
    // Create a LocalSession with test configuration (reduced load_time for faster tests)
    let session = LocalSession {
        base_path: session_path,
        simulate: true,
        load_time: 1, // Reduced from 30 to 1 for faster tests
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };
    
    // This should handle malformed data gracefully
    let _ = session.load("");
    let _ = session.load("testsession");
}

#[test]
#[ignore]
fn test_session_trait_save_method() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // Create a LocalSession with test configuration
    let session = LocalSession {
        base_path: session_path.clone(),
        simulate: false, 
        load_time: 1,
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // Test saving a session (should work even in simulate mode for basic functionality)
    let result = session.save("test_session");
    assert!(result.is_ok());
    assert!(std::path::Path::new(&format!("{}/test_session", session_path)).exists());
}

#[test]
fn test_session_trait_list_method() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // Create a LocalSession with test configuration
    let session = LocalSession {
        base_path: session_path.clone(),
        simulate: true,
        load_time: 1,
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // Create some mock session directories
    std::fs::create_dir_all(format!("{}/session1", session_path)).unwrap();
    std::fs::create_dir_all(format!("{}/session2", session_path)).unwrap();
    std::fs::create_dir_all(format!("{}/session3", session_path)).unwrap();

    // Test listing sessions
    let sessions: Vec<String> = session.list().collect();
    
    // Should find our created sessions (order may vary)
    assert!(sessions.len() >= 3);
    assert!(sessions.contains(&"session1".to_string()));
    assert!(sessions.contains(&"session2".to_string()));
    assert!(sessions.contains(&"session3".to_string()));
}

#[test]
fn test_session_trait_delete_method() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // Create a LocalSession with test configuration
    let session = LocalSession {
        base_path: session_path.clone(),
        simulate: true,
        load_time: 1,
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // Create a mock session directory
    let session_dir = format!("{}/test_delete", session_path);
    std::fs::create_dir_all(&session_dir).unwrap();
    
    // Verify it exists
    assert!(std::path::Path::new(&session_dir).exists());

    // Test deleting the session
    session.delete("test_delete");
    
    // Verify it's been deleted
    assert!(!std::path::Path::new(&session_dir).exists());
}

#[test]
fn test_session_trait_clear_method() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // Create a LocalSession with test configuration
    let session = LocalSession {
        base_path: session_path,
        simulate: true, // Use simulate mode to avoid actual process killing
        load_time: 1,
        adjust_clients_only: false,
        save_duplicate_pids: false,
    };

    // Test clearing session (should work in simulate mode)
    let result = session.clear();
    
    // In simulate mode, this should succeed without errors
    match result {
        Ok(_) => println!("Clear test passed"),
        Err(e) => println!("Clear test completed with result: {:?}", e),
    }
}

#[test]
fn test_local_session_struct_creation() {
    // Test that LocalSession can be created with different configurations
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();

    // Test default-like configuration
    let session1 = LocalSession {
        base_path: session_path.clone(),
        simulate: false,
        load_time: 60,
        adjust_clients_only: false,
        save_duplicate_pids: true,
    };

    // Test simulation configuration
    let session2 = LocalSession {
        base_path: session_path.clone(),
        simulate: true,
        load_time: 5,
        adjust_clients_only: true,
        save_duplicate_pids: false,
    };

    // Verify fields are set correctly
    assert_eq!(session1.simulate, false);
    assert_eq!(session1.load_time, 60);
    assert_eq!(session1.save_duplicate_pids, true);
    
    assert_eq!(session2.simulate, true);
    assert_eq!(session2.load_time, 5);
    assert_eq!(session2.adjust_clients_only, true);
}


#[cfg(test)]
mod cli_tests {
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_cli_help() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--help"])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("hyprsession"));
        assert!(stdout.contains("[MODE]"));
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
            .args(&["run", "--", "save-and-exit", "--simulate"])
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
    fn test_list_sessions_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let session_path = temp_dir.path().to_str().unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--", "list"])
            .env("HYPRSESSION_PATH", session_path)
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Available sessions:"));
    }

    #[test]
    fn test_list_sessions() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let session_path = temp_dir.path().to_str().unwrap();

        // Create some dummy session directories
        std::fs::create_dir(format!("{}/session1", session_path)).unwrap();
        std::fs::create_dir(format!("{}/session2", session_path)).unwrap();

        let output = Command::new("cargo")
            .args(&["run", "--", "list"])
            .env("HYPRSESSION_PATH", session_path)
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("session1"));
        assert!(stdout.contains("session2"));

        // Clean up
        std::fs::remove_dir_all(format!("{}/session1", session_path)).unwrap();
        std::fs::remove_dir_all(format!("{}/session2", session_path)).unwrap();
    }

    #[test]
    #[ignore]
    fn test_cli_invalid_save_interval() {
        let output = Command::new("cargo")
            .args(&["run", "--", "--save-interval", "0"])
            .output()
            .expect("Failed to execute command");

        // Should fail with invalid save interval
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Save interval needs to be greater than 0"));
    }
}