# Hyprsession Tests

This directory contains comprehensive tests for the hyprsession project.

## Test Files

### `unit_tests.rs`
Contains unit tests for individual components:
- **Session Tests**: Test session file creation, parsing, property formatting, path handling, and command extraction from wrapped executables
- **Command Extraction Tests**: Test regex-based extraction of application names from `.wrapped` executables (e.g., `.firefox.wrapped` → `firefox`)
- **Argument Parsing Tests**: Test CLI argument parsing with various combinations of flags and options

### `integration_tests.rs`
Contains integration tests that test the application as a whole:
- **Session Loading Tests**: Test loading sessions from files with various states (empty, nonexistent, sample data)
- **CLI Tests**: Test the command-line interface with different arguments and modes

### `performance_tests.rs`
Contains performance and stress tests:
- **Benchmark Tests**: Measure session loading performance with large datasets
- **Robustness Tests**: Test handling of malformed entries and large files

## Running Tests

To run all tests:
```bash
cargo test
```

To run specific test files:
```bash
cargo test --test unit_tests
cargo test --test integration_tests
cargo test --test performance_tests
```

To run tests with output:
```bash
cargo test -- --nocapture
```

## Test Coverage

The tests cover:
- ✅ Session file loading and parsing
- ✅ CLI argument validation
- ✅ Error handling for missing/malformed files
- ✅ Performance with large session files
- ✅ Property formatting and filtering
- ✅ Directory and path handling
- ✅ Mode selection and configuration
- ✅ Command extraction from wrapped executables (Nix-style `.wrapped` files)
- ✅ Regex pattern matching for application name extraction

## Notes

- Tests use temporary directories to avoid affecting the system
- The `simulate` mode is used in tests to avoid requiring a running Hyprland instance
- CLI tests may show warnings in test environments where Hyprland is not available, which is expected