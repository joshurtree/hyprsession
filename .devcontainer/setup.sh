#!/bin/bash

set -e

echo "🚀 Setting up Hyprsession development environment..."

# Install additional system dependencies
echo "📦 Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget

# Install Rust components
echo "🦀 Installing Rust components..."
rustup component add rustfmt clippy rust-src

# Install cargo tools
echo "🔧 Installing cargo tools..."
cargo install --quiet cargo-watch cargo-edit cargo-audit cargo-outdated

# Verify Nix installation and configure
echo "❄️ Configuring Nix..."
if command -v nix &> /dev/null; then
    echo "Nix is available"
    # Enable flakes
    mkdir -p ~/.config/nix
    echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
else
    echo "⚠️ Nix not found, skipping nix configuration"
fi

# Install project dependencies
echo "📚 Installing project dependencies..."
cd /workspace
cargo fetch

# Create useful aliases
echo "⚙️ Setting up aliases..."
cat >> ~/.bashrc << 'EOF'

# Hyprsession development aliases
alias cr='cargo run'
alias cb='cargo build'
alias ct='cargo test'
alias cc='cargo check'
alias cf='cargo fmt'
alias ccl='cargo clippy'
alias cw='cargo watch -x check -x test -x run'
alias ndev='nix develop'
alias nbuild='nix build'

EOF

echo "✅ Development environment setup complete!"
echo ""
echo "🎯 Available commands:"
echo "  cr    - cargo run"
echo "  cb    - cargo build" 
echo "  ct    - cargo test"
echo "  cc    - cargo check"
echo "  cf    - cargo fmt"
echo "  ccl   - cargo clippy"
echo "  cw    - cargo watch (runs check, test, run on file changes)"
echo "  ndev  - nix develop"
echo "  nbuild- nix build"
echo ""
echo "🔧 VS Code extensions for Rust development are being installed..."
echo "🏁 Ready to code!"