# Use the official Rust image as base
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install additional system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install Nix (optional for hybrid development)
RUN curl -L https://nixos.org/nix/install | sh -s -- --daemon \
    && echo '. /nix/var/nix/profiles/default/etc/profile.d/nix.sh' >> /home/vscode/.bashrc

# Create nix config directory and enable flakes
RUN mkdir -p /home/vscode/.config/nix \
    && echo "experimental-features = nix-command flakes" > /home/vscode/.config/nix/nix.conf \
    && chown -R vscode:vscode /home/vscode/.config

# Install Rust tools
USER vscode
RUN rustup component add rustfmt clippy rust-src \
    && cargo install cargo-watch cargo-edit cargo-audit cargo-outdated

# Set working directory
WORKDIR /workspace

# Default command
CMD ["sleep", "infinity"]