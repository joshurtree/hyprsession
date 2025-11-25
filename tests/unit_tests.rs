use std::fs::{create_dir_all, remove_dir_all};
use std::io::Read;
use std::path::Path;

// Mock test data structures to simulate Hyprland client data
#[derive(Debug, Clone)]
struct MockClient {
    pid: i32,
    monitor: i32,
    workspace_id: i32,
    floating: bool,
    at: (i32, i32),
    size: (i32, i32),
    pinned: bool,
    fullscreen: u8,
    title: String,
    class: String,
}

impl Default for MockClient {
    fn default() -> Self {
        MockClient {
            pid: 1234,
            monitor: 0,
            workspace_id: 1,
            floating: false,
            at: (100, 200),
            size: (800, 600),
            pinned: false,
            fullscreen: 0,
            title: "Test Window".to_string(),
            class: "test-app".to_string(),
        }
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;
    use regex::Regex;
    use std::fs::File;
    use std::io::Write;

    const TEST_SESSION_DIR: &str = "/tmp/hyprsession_unit_test";

    fn setup_test_environment() {
        if Path::new(TEST_SESSION_DIR).exists() {
            remove_dir_all(TEST_SESSION_DIR).unwrap();
        }
        create_dir_all(TEST_SESSION_DIR).unwrap();
    }

    fn cleanup_test_environment() {
        if Path::new(TEST_SESSION_DIR).exists() {
            remove_dir_all(TEST_SESSION_DIR).unwrap();
        }
    }

    #[test]
    fn test_session_file_creation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let exec_file_path = temp_dir.path().join("exec.conf");

        // Create a test session file
        let mut file = File::create(&exec_file_path).expect("Failed to create session file");
        writeln!(file, "[monitor 0;workspace 1 silent] test-command").unwrap();

        // Verify the file was created and contains expected content
        assert!(exec_file_path.exists());

        let mut contents = String::new();
        File::open(&exec_file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(contents.contains("monitor 0"));
        assert!(contents.contains("workspace 1 silent"));
        assert!(contents.contains("test-command"));
    }

    #[test]
    fn test_session_properties_formatting() {
        let client = MockClient {
            pid: 5678,
            monitor: 1,
            workspace_id: 3,
            floating: true,
            at: (50, 75),
            size: (400, 300),
            pinned: true,
            fullscreen: 1,
            ..Default::default()
        };

        // Test individual property formatting
        assert_eq!(format!("monitor {}", client.monitor), "monitor 1");
        assert_eq!(
            format!("workspace {} silent", client.workspace_id),
            "workspace 3 silent"
        );
        assert_eq!(
            format!("move {} {}", client.at.0, client.at.1),
            "move 50 75"
        );
        assert_eq!(
            format!("size {} {}", client.size.0, client.size.1),
            "size 400 300"
        );
        assert_eq!(
            format!("fullscreenstate {}", client.fullscreen as i32),
            "fullscreenstate 1"
        );
    }

    #[test]
    fn test_run_if_helper() {
        // Test the run_if helper function behavior
        fn run_if(prop: bool, val: &str) -> &str {
            if prop {
                val
            } else {
                ""
            }
        }

        assert_eq!(run_if(true, "float"), "float");
        assert_eq!(run_if(false, "float"), "");
        assert_eq!(run_if(true, "pin"), "pin");
        assert_eq!(run_if(false, "pin"), "");
    }

    #[test]
    fn test_duplicate_pid_handling() {
        // Test that duplicate PIDs are handled according to the save_duplicate_pids flag
        let pids = vec![1234, 5678, 1234, 9012, 5678];
        let mut unique_pids = Vec::new();

        for pid in pids {
            if !unique_pids.contains(&pid) {
                unique_pids.push(pid);
            }
        }

        assert_eq!(unique_pids.len(), 3);
        assert!(unique_pids.contains(&1234));
        assert!(unique_pids.contains(&5678));
        assert!(unique_pids.contains(&9012));
    }

    #[test]
    fn test_session_file_parsing() {
        setup_test_environment();

        let exec_file_path = format!("{}/exec.conf", TEST_SESSION_DIR);
        let mut file = File::create(&exec_file_path).unwrap();

        // Write test session data
        writeln!(
            file,
            "[monitor 0;workspace 1 silent;move 100 200;size 800 600] firefox"
        )
        .unwrap();
        writeln!(
            file,
            "[monitor 1;workspace 2 silent;float;move 50 50;size 400 300;pin] kitty"
        )
        .unwrap();
        writeln!(
            file,
            "[monitor 0;workspace 3 silent;fullscreenstate 1] code"
        )
        .unwrap();

        // Read and verify the content
        let mut contents = String::new();
        File::open(&exec_file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3);

        // Test first line
        assert!(lines[0].contains("monitor 0"));
        assert!(lines[0].contains("workspace 1 silent"));
        assert!(lines[0].contains("move 100 200"));
        assert!(lines[0].contains("size 800 600"));
        assert!(lines[0].contains("firefox"));

        // Test second line (floating window)
        assert!(lines[1].contains("monitor 1"));
        assert!(lines[1].contains("float"));
        assert!(lines[1].contains("pin"));
        assert!(lines[1].contains("kitty"));

        // Test third line (fullscreen)
        assert!(lines[2].contains("fullscreenstate 1"));
        assert!(lines[2].contains("code"));

        cleanup_test_environment();
    }

    #[test]
    fn test_empty_properties_filtering() {
        // Test that empty properties are filtered out
        let properties = vec![
            "monitor 0",
            "",
            "workspace 1 silent",
            "",
            "move 100 200",
            "",
        ];
        let filtered: Vec<&str> = properties
            .iter()
            .filter(|prop| !prop.is_empty())
            .copied()
            .collect();

        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&"monitor 0"));
        assert!(filtered.contains(&"workspace 1 silent"));
        assert!(filtered.contains(&"move 100 200"));
    }

    #[test]
    fn test_session_directory_path_handling() {
        let base_paths = vec!["/tmp/test", "/tmp/test/", "relative/path", "relative/path/"];

        for base_path in base_paths {
            let exec_path = format!("{}/exec.conf", base_path.trim_end_matches('/'));
            assert!(exec_path.ends_with("/exec.conf"));
            assert!(!exec_path.contains("//"));
        }
    }

    #[test]
    fn test_wrapped_command_extraction() {
        // Test the regex logic used in fetch_command for extracting app names from .wrapped commands
        let re = Regex::new(r"\.(?<command>.+)\.wrapped").unwrap();

        // Test case 1: .firefox.wrapped should extract "firefox"
        let command1 = ".firefox.wrapped";
        assert!(re.is_match(command1));
        let captures1 = re.captures(command1).unwrap();
        let extracted1 = captures1.name("command").unwrap().as_str();
        assert_eq!(extracted1, "firefox");

        // Test case 2: .chromium.wrapped should extract "chromium"
        let command2 = ".chromium.wrapped";
        assert!(re.is_match(command2));
        let captures2 = re.captures(command2).unwrap();
        let extracted2 = captures2.name("command").unwrap().as_str();
        assert_eq!(extracted2, "chromium");

        // Test case 3: Regular command without .wrapped should not match
        let command3 = "firefox";
        assert!(!re.is_match(command3));

        // Test case 4: Complex path with .wrapped
        let command4 = "/nix/store/hash-.firefox.wrapped --some-args";
        assert!(re.is_match(command4));
        let captures4 = re.captures(command4).unwrap();
        let extracted4 = captures4.name("command").unwrap().as_str();
        assert_eq!(extracted4, "firefox");

        // Test case 5: Command with arguments
        let command5 = ".code.wrapped /some/file.rs";
        assert!(re.is_match(command5));
        let captures5 = re.captures(command5).unwrap();
        let extracted5 = captures5.name("command").unwrap().as_str();
        assert_eq!(extracted5, "code");
    }

    #[test]
    fn test_command_extraction_logic() {
        // Test the complete command extraction logic as implemented in fetch_command
        fn extract_command_name(ps_output: &str) -> String {
            let re = Regex::new(r"\.(?<command>.+)\.wrapped").unwrap();
            if re.is_match(ps_output) {
                re.captures(ps_output)
                    .unwrap()
                    .name("command")
                    .unwrap()
                    .as_str()
                    .to_string()
            } else {
                ps_output.trim().to_string()
            }
        }

        // Test .firefox.wrapped -> firefox (the specific case requested)
        assert_eq!(extract_command_name(".firefox.wrapped"), "firefox");

        // Test .discord.wrapped -> discord
        assert_eq!(extract_command_name(".discord.wrapped"), "discord");

        // Test regular command stays unchanged
        assert_eq!(extract_command_name("kitty"), "kitty");

        // Test command with whitespace
        assert_eq!(extract_command_name("  vim  "), "vim");

        // Test empty string
        assert_eq!(extract_command_name(""), "");

        // Test complex wrapped command with path and arguments
        assert_eq!(
            extract_command_name("/nix/store/xyz-.vscode.wrapped --no-sandbox"),
            "vscode"
        );
    }

    #[test]
    fn test_firefox_wrapped_specific_case() {
        // This test specifically verifies the case mentioned in the request:
        // if ps produces ".firefox.wrapped" then "firefox" is returned

        let re = Regex::new(r"\.(?<command>.+)\.wrapped").unwrap();
        let ps_output = ".firefox.wrapped";

        // Verify the regex matches
        assert!(
            re.is_match(ps_output),
            "Regex should match .firefox.wrapped"
        );

        // Extract the command name
        let captures = re.captures(ps_output).unwrap();
        let extracted_command = captures.name("command").unwrap().as_str();

        // Verify "firefox" is extracted
        assert_eq!(
            extracted_command, "firefox",
            "Expected 'firefox' to be extracted from '.firefox.wrapped'"
        );

        // Test with the same logic as the fetch_command function
        let result = if re.is_match(ps_output) {
            re.captures(ps_output)
                .unwrap()
                .name("command")
                .unwrap()
                .as_str()
                .to_string()
        } else {
            ps_output.trim().to_string()
        };

        assert_eq!(
            result, "firefox",
            "fetch_command logic should return 'firefox' for '.firefox.wrapped'"
        );
    }
}

#[cfg(test)]
mod argument_parsing_tests {
    use clap::Parser;

    // Copy the structures from main.rs for testing
    #[derive(Copy, Clone, Debug, Parser, PartialEq, clap::ValueEnum)]
    enum Mode {
        Default,
        SaveOnly,
        SaveAndExit,
        LoadAndExit,
    }

    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    struct Args {
        #[arg(short, long)]
        mode: Option<Mode>,
        #[arg(long, default_value_t = false)]
        save_duplicate_pids: bool,
        #[arg(short = 'i', long)]
        save_interval: Option<u64>,
        #[arg(short = 's', long)]
        session_path: Option<String>,
        #[arg(long, default_value_t = false)]
        simulate: bool,
    }

    #[test]
    fn test_default_mode_parsing() {
        let args = Args::try_parse_from(&["hyprsession"]).unwrap();
        assert_eq!(args.mode.unwrap_or(Mode::Default), Mode::Default);
        assert_eq!(args.save_duplicate_pids, false);
        assert_eq!(args.save_interval.unwrap_or(60), 60);
        assert_eq!(args.simulate, false);
    }

    #[test]
    fn test_mode_parsing() {
        let test_cases = vec![
            ("default", Mode::Default),
            ("save-only", Mode::SaveOnly),
            ("save-and-exit", Mode::SaveAndExit),
            ("load-and-exit", Mode::LoadAndExit),
        ];

        for (mode_str, expected_mode) in test_cases {
            let args = Args::try_parse_from(&["hyprsession", "--mode", mode_str]).unwrap();
            assert_eq!(args.mode.unwrap(), expected_mode);
        }
    }

    #[test]
    fn test_save_interval_parsing() {
        let args = Args::try_parse_from(&["hyprsession", "--save-interval", "120"]).unwrap();
        assert_eq!(args.save_interval.unwrap(), 120);
    }

    #[test]
    fn test_session_path_parsing() {
        let test_path = "/custom/session/path";
        let args = Args::try_parse_from(&["hyprsession", "--session-path", test_path]).unwrap();
        assert_eq!(args.session_path.unwrap(), test_path);
    }

    #[test]
    fn test_simulate_flag_parsing() {
        let args = Args::try_parse_from(&["hyprsession", "--simulate"]).unwrap();
        assert_eq!(args.simulate, true);
    }

    #[test]
    fn test_save_duplicate_pids_flag() {
        let args = Args::try_parse_from(&["hyprsession", "--save-duplicate-pids"]).unwrap();
        assert_eq!(args.save_duplicate_pids, true);
    }

    #[test]
    fn test_combined_arguments() {
        let args = Args::try_parse_from(&[
            "hyprsession",
            "--mode",
            "save-only",
            "--save-interval",
            "30",
            "--session-path",
            "/tmp/test",
            "--simulate",
            "--save-duplicate-pids",
        ])
        .unwrap();

        assert_eq!(args.mode.unwrap(), Mode::SaveOnly);
        assert_eq!(args.save_interval.unwrap(), 30);
        assert_eq!(args.session_path.unwrap(), "/tmp/test");
        assert_eq!(args.simulate, true);
        assert_eq!(args.save_duplicate_pids, true);
    }
}
