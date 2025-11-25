# Development Container for Hyprsession

This directory contains the development container configuration for the Hyprsession project.

## What's Included

### Base Environment
- **Rust**: Latest stable Rust toolchain with rustfmt, clippy, and rust-src
- **Nix**: Package manager for reproducible builds (supports the project's flake.nix)
- **System Tools**: Essential build tools, git, curl, and development utilities

### VS Code Extensions
- **rust-analyzer**: Advanced Rust language support
- **even-better-toml**: Enhanced TOML file support for Cargo.toml
- **vscode-lldb**: Debugger for Rust applications
- **crates**: Helps manage Cargo dependencies
- **nix-ide**: Nix language support
- **GitHub Copilot**: AI-powered code completion

### Development Tools
- **cargo-watch**: Automatically runs cargo commands on file changes
- **cargo-edit**: Command-line Cargo.toml manipulation
- **cargo-audit**: Security vulnerability scanner
- **cargo-outdated**: Check for outdated dependencies

## Quick Start

1. **Open in VS Code**: Make sure you have the Dev Containers extension installed
2. **Reopen in Container**: When prompted, click "Reopen in Container" or use Command Palette > "Dev Containers: Reopen in Container"
3. **Wait for Setup**: The container will build and run the setup script automatically
4. **Start Coding**: You're ready to develop!

## Available Commands

Once inside the container, you can use these convenient aliases:

```bash
# Cargo shortcuts
cr     # cargo run
cb     # cargo build
ct     # cargo test
cc     # cargo check
cf     # cargo fmt
ccl    # cargo clippy
cw     # cargo watch (auto-rebuild on changes)

# Nix shortcuts
ndev   # nix develop
nbuild # nix build
```

## Development Workflows

### Rust Development
```bash
# Run the application
cargo run

# Run with arguments
cargo run -- --help

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy

# Watch for changes and auto-rebuild
cargo watch -x check -x test -x run
```

### Nix Development
```bash
# Enter nix development shell
nix develop

# Build with nix
nix build

# Run the built package
./result/bin/hyprsession
```

### Debugging
- Set breakpoints in VS Code
- Use F5 to start debugging
- The container includes LLDB debugger support

## Container Features

### Volume Mounts
- **Workspace**: Your project files are mounted and cached for performance
- **Cargo Registry**: Shared cargo registry to speed up dependency downloads

### Port Forwarding
- Configured to forward any ports your application might use
- Add ports to the `forwardPorts` array in devcontainer.json if needed

### Environment Variables
- `RUST_BACKTRACE=1`: Enhanced error reporting
- `CARGO_TARGET_DIR`: Dedicated target directory for builds

## Customization

### Adding Extensions
Edit `.devcontainer/devcontainer.json` and add extension IDs to the `extensions` array.

### Installing Additional Tools
Modify `.devcontainer/setup.sh` to install additional tools or packages.

### Changing the Base Image
Edit the `image` field in `devcontainer.json` or use the custom `Dockerfile`.

## Alternative Setup

If you prefer using docker-compose:

```bash
cd .devcontainer
docker-compose up -d
docker-compose exec hyprsession-dev bash
```

## Troubleshooting

### Container Won't Start
- Check Docker is running
- Try rebuilding: Command Palette > "Dev Containers: Rebuild Container"

### Extensions Not Loading
- Wait for the container to fully initialize
- Try reloading the window: Command Palette > "Developer: Reload Window"

### Nix Commands Fail
- Source the nix profile: `source /nix/var/nix/profiles/default/etc/profile.d/nix.sh`
- Or restart your shell session

### Performance Issues
- Ensure you're using the cached volume mounts
- Close unused applications to free up system resources

## Support

For issues specific to the dev container setup, check:
1. VS Code Dev Containers documentation
2. The project's main README.md
3. Open an issue in the project repository