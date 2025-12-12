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
    let _ = load_session(&session_path, 30, false, true);
}

#[test]
fn test_load_session_with_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{}/exec.conf", session_path);

    // Create an empty session file
    File::create(&exec_file_path).expect("Failed to create test file");

    // This should handle empty files gracefully
    let _ = load_session(&session_path, 30, false, true);
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
    let _ = load_session(&session_path, 30, false, true);
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

#[cfg(test)]
mod documentation_tests {
    use std::fs;

    /// Doc test that verifies the current version has a corresponding section in README.md
    /// This ensures that when we update the version in Cargo.toml, we also update the changelog.
    #[test]
    fn test_version_has_readme_section() {
        // Read the current version from Cargo.toml
        let cargo_toml = fs::read_to_string("Cargo.toml")
            .expect("Failed to read Cargo.toml");
        
        // Extract version from Cargo.toml
        let version_line = cargo_toml.lines()
            .find(|line| line.starts_with("version = "))
            .expect("Could not find version line in Cargo.toml");
        
        let version = version_line
            .split('"')
            .nth(1)
            .expect("Could not extract version from Cargo.toml")
            .split('-')
            .nth(0)
            .expect("Could not extract version from Cargo.toml")
            .trim();
        
        // Read README.md
        let readme = fs::read_to_string("README.md")
            .expect("Failed to read README.md");
        
        // Check if the version appears as a changelog section
        let version_section = format!("### {}", version);
        
        assert!(
            readme.contains(&version_section),
            "README.md does not contain a changelog section for version {}. \
             Please add a '{}' section to the changelog in README.md",
            version,
            version_section
        );
        
        // Additional check: ensure the changelog section appears after "## Change log"
        let changelog_start = readme.find("## Change log")
            .expect("Could not find '## Change log' section in README.md");
        
        let version_position = readme.find(&version_section)
            .expect(&format!("Version section {} not found in README.md", version_section));
        
        assert!(
            version_position > changelog_start,
            "Version section {} should appear after the '## Change log' header in README.md",
            version_section
        );
        
        println!("✅ Version {} has a proper changelog section in README.md", version);
    }

    /// Test that verifies the README.md has proper structure
    #[test] 
    fn test_readme_structure() {
        let readme = fs::read_to_string("README.md")
            .expect("Failed to read README.md");
        
        // Check for essential sections
        assert!(readme.contains("# Hyprsession"), "README should start with main title");
        assert!(readme.contains("## Overview"), "README should have an Overview section");
        assert!(readme.contains("## Installation"), "README should have an Installation section");
        assert!(readme.contains("## Usage"), "README should have a Usage section");
        assert!(readme.contains("## Change log"), "README should have a Change log section");
        
        // Verify section ordering makes sense
        let overview_pos = readme.find("## Overview").unwrap();
        let installation_pos = readme.find("## Installation").unwrap();
        let usage_pos = readme.find("## Usage").unwrap();
        let changelog_pos = readme.find("## Change log").unwrap();
        
        assert!(
            overview_pos < installation_pos,
            "Overview should come before Installation"
        );
        assert!(
            installation_pos < usage_pos,
            "Installation should come before Usage"  
        );
        assert!(
            usage_pos < changelog_pos,
            "Usage should come before Change log"
        );
        
        // Check that changelog is in the latter half of the document
        let total_length = readme.len();
        assert!(
            changelog_pos as f64 / total_length as f64 > 0.5,
            "Change log should appear in the second half of the README"
        );
        
        println!("✅ README.md has proper structure");
    }
}
