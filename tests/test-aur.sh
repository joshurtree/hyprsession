#!/usr/bin/env bash

# Quick Arch Linux container for testing AUR packages
# Run with: nix-shell arch-container.nix --run ./test-aur.sh

set -e

echo "Starting Arch Linux container for AUR testing..."

# Run an Arch container with necessary tools
docker run -it --rm \
  -v "$(pwd):/workspace:ro" \
  archlinux:latest \
  bash -c '
    # Update system and install build tools
    pacman -Syu --noconfirm
    pacman -S --noconfirm base-devel git sudo

    # Create a build user (AUR packages cannot be built as root)
    useradd -m builder
    echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
    
    # Switch to builder user
    su - builder -c "
      # Clone and build the AUR package
      cd /tmp
      git clone https://aur.archlinux.org/hyprsession.git
      cd hyprsession
      
      echo \"\"
      echo \"=== PKGBUILD Contents ===\"
      cat PKGBUILD
      echo \"\"
      echo \"=== Building package ===\"
      makepkg -si --noconfirm
      
      echo \"\"
      echo \"=== Testing installation ===\"
      hyprsession --version || echo \"Package installed but hyprsession command not found\"
      
      echo \"\"
      echo \"=== Package info ===\"
      pacman -Qi hyprsession
    "
  '

echo ""
echo "AUR package test complete!"
