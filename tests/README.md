# Hyprsession Integration Tests

This directory contains a comprehensive integration test suite for hyprsession using NixOS VMs.

## Quick Start

```bash
cd tests
nix run .#run-test
```

This will:
1. Build a NixOS VM with Hyprland and hyprsession
2. Auto-start the integration test
3. Save results to `./test-results/`

## What the Test Does

The integration test performs a complete workflow:

1. **Environment Setup**: Starts a NixOS VM with Hyprland, hyprsession, and test applications
2. **Application Loading**: Opens multiple test applications:
   - Firefox (regular)
   - Firefox (Flatpak, if available) 
   - Kitty terminal
   - GNOME Calculator
3. **State Capture**: Records current window state with `hyprctl clients`
4. **Session Save**: Uses `hyprsession save test-session` to save the current state
5. **Workspace Clear**: Closes all applications to simulate a fresh start
6. **Session Restore**: Uses `hyprsession load test-session` to restore applications
7. **State Comparison**: Compares before/after states and reports success/failure

## Test Results

Results are saved to `./test-results/` with these files:

- `expected.txt` - Original `hyprctl clients` output before session save
- `actual.txt` - `hyprctl clients` output after session restore  
- `expected_classes.txt` - Extracted application class names (sorted)
- `actual_classes.txt` - Restored application class names (sorted)
- `diff.txt` - Differences between expected and actual states
- `result.txt` - Overall test result (`PASS`, `PARTIAL`, or `FAIL`)

## Manual Testing

You can also run manual tests inside the VM:

```bash
# Build and run VM without auto-test
nix build .#vm
./result/bin/run-nixos-vm

# Inside the VM (auto-login as testuser):
hyprsession save my-session
# ... open some applications ...
hyprsession load my-session
hyprsession list
hyprsession delete my-session
```

## Available Nix Commands

- `nix run .#run-test` - Full automated test run
- `nix build .#vm` - Build VM image only
- `nix run .#vm` - Run built VM manually
- `nix develop` - Enter development shell with tools
- `nix build .#hyprsession` - Build just the hyprsession binary
- `nix run .#test-script` - Run just the test script (for debugging)

## VM Configuration

The test VM includes:
- **OS**: NixOS with Hyprland as the window manager
- **Memory**: 4GB RAM, 4 CPU cores
- **Display**: 1920x1080 with hardware acceleration
- **Auto-login**: User `testuser` (password: `test`)
- **Shared Folder**: `./test-results/` mounted as `/shared/test-results/`
- **Applications**: Firefox, Kitty, Calculator, Flatpak support

## Hyprland Configuration

The VM uses a minimal Hyprland config optimized for testing:
- Basic window management and animations
- Simple keybindings (Super+Q for terminal, Super+E for Firefox)
- Auto-execution of the test script on startup

## Troubleshooting

### VM Won't Start
- Ensure KVM/QEMU is available: `ls /dev/kvm`
- Check virtualization is enabled in BIOS
- Try without hardware acceleration: modify `qemu.options` in flake.nix

### Test Fails
- Check `test-results/diff.txt` for specific differences
- Some applications may start slower - increase sleep timers in test script
- Flatpak apps require first-time setup - may need manual initialization

### Applications Don't Start
- Verify applications are installed: check `environment.systemPackages` in flake.nix
- Check Hyprland logs: `journalctl -u display-manager`
- Test manually inside VM before running automated test

### Permissions Issues
- Ensure test-results directory is writable
- Check 9p filesystem mounting in VM
- Try running with `--option sandbox false` if using Nix sandbox

## Extending Tests

To add new test applications:

1. Add package to `environment.systemPackages` in flake.nix
2. Add startup command to test script
3. Adjust sleep timers as needed for application startup
4. Update expected application classes if needed

To test Flatpak applications:

1. The VM includes Flatpak support
2. You may need to add Flathub repository setup to the test script
3. Install test apps in the VM configuration or test script

## Performance Notes

- First run downloads ~2GB of Nix packages
- Subsequent runs use cached packages
- VM startup takes 30-60 seconds
- Full test cycle takes 2-3 minutes
- Results are cached until test-results directory is cleared