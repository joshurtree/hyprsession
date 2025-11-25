use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use tempfile::TempDir;

use hyprsession::session::load_session;

#[test]
fn benchmark_load_session_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{}/exec.conf", session_path);

    // Create a session file with many entries to test performance
    let mut file = File::create(&exec_file_path).expect("Failed to create test file");
    for i in 0..1000 {
        writeln!(
            file,
            "[monitor 0;workspace {} silent;move {} {};size 800 600] test-app-{}",
            i % 10,
            i * 10,
            i * 5,
            i
        )
        .unwrap();
    }

    // Measure loading time
    let start = Instant::now();
    load_session(&session_path, true);
    let duration = start.elapsed();

    println!("Loading 1000 session entries took: {:?}", duration);

    // Assert that loading takes less than 1 second (should be much faster)
    assert!(
        duration.as_secs() < 1,
        "Session loading took too long: {:?}",
        duration
    );
}

#[test]
fn test_large_session_file_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{session_path}/exec.conf");

    // Create a session file with very long lines
    let mut file = File::create(&exec_file_path).expect("Failed to create test file");
    let long_command = "a".repeat(1000);
    writeln!(file, "[monitor 0;workspace 1 silent] {long_command}").unwrap();

    // This shouldn't crash or fail
    load_session(&session_path, true);
}

#[test]
fn test_malformed_session_entries() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let session_path = temp_dir.path().to_str().unwrap().to_string();
    let exec_file_path = format!("{session_path}/exec.conf");

    // Create a session file with malformed entries
    let mut file = File::create(&exec_file_path).expect("Failed to create test file");
    writeln!(file, "malformed line without brackets").unwrap();
    writeln!(file, "[incomplete bracket entry").unwrap();
    writeln!(file, "missing command]").unwrap();
    writeln!(file, "[monitor 0;workspace 1 silent] valid-command").unwrap();
    writeln!(file, "").unwrap(); // Empty line
    writeln!(file, "   ").unwrap(); // Whitespace only

    // This should handle malformed entries gracefully
    load_session(&session_path, true);
}
