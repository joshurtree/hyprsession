#!/usr/bin/env bash

# Hyprsession Integration Test Runner
# This script provides a convenient interface for running integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                  Hyprsession Integration Tests               ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_usage() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  run, test     Run the full integration test (default)"
    echo "  build         Build the test VM only"
    echo "  vm            Run the VM manually (no auto-test)"
    echo "  clean         Clean test results and build artifacts"
    echo "  shell         Enter development shell"
    echo "  help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Run full test"
    echo "  $0 run          # Same as above"
    echo "  $0 vm           # Start VM for manual testing"
    echo "  $0 clean        # Clean up generated files"
}

check_dependencies() {
    if ! command -v nix &> /dev/null; then
        echo -e "${RED}Error: Nix is required but not installed${NC}"
        echo "Please install Nix: https://nixos.org/download.html"
        exit 1
    fi
    
    if ! nix --version | grep -q "nix (Nix) 2\|nix (Nix) [3-9]"; then
        echo -e "${YELLOW}Warning: This flake requires Nix with flake support${NC}"
        echo "Please ensure you have Nix 2.4+ with experimental features enabled"
    fi
}

run_test() {
    echo -e "${BLUE}Starting Hyprsession Integration Test...${NC}"
    echo ""
    
    cd "$SCRIPT_DIR"
    
    # Clean previous results
    echo -e "${YELLOW}Cleaning previous test results...${NC}"
    rm -rf ./test-results
    mkdir -p ./test-results
    
    # Build the VM
    echo -e "${BLUE}Building test VM (this may take a while on first run)...${NC}"
    if nix build .#vm --no-link --out-link vm-result; then
        echo -e "${BLUE}Starting test VM...${NC}"
        echo "VM will auto-login and start the test automatically."
        echo "Close the VM window when testing is complete."
        echo ""
        
        # Run VM in background and wait for it to finish
        TMPDIR=/tmp ./vm-result/bin/run-hyprsession-test-vm  &
        VM_PID=$!
        
        echo "VM started with PID $VM_PID"
        echo "Waiting for VM to complete tests..."
        echo "(You can close this with Ctrl+C and the VM will continue running)"

        # Wait for the VM process
        wait $VM_PID || true
        
        echo ""
        echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                        TEST COMPLETE                         ║${NC}"
        echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
        
        # Show results if available
        if [ -f "./test-results/result.txt" ]; then
            result=$(cat ./test-results/result.txt)
            case "$result" in
                "PASS")
                    echo -e "${GREEN}✅ Test Result: PASS - Session restored successfully!${NC}"
                    ;;
                "PARTIAL")
                    echo -e "${YELLOW}⚠️  Test Result: PARTIAL - Some differences found${NC}"
                    echo -e "${YELLOW}Check ./test-results/diff.txt for details${NC}"
                    ;;
                "FAIL")
                    echo -e "${RED}❌ Test Result: FAIL - Session restore failed${NC}"
                    echo -e "${RED}Check ./test-results/ for error details${NC}"
                    ;;
                *)
                    echo -e "${YELLOW}Test completed but result is unclear${NC}"
                    ;;
            esac
        else
            echo -e "${YELLOW}No test results found - VM may have been closed early${NC}"
        fi
        
        echo ""
        echo "Test artifacts saved to: ./test-results/"
        echo "  - expected.txt: Original window state"
        echo "  - actual.txt: Restored window state" 
        echo "  - diff.txt: Comparison results"
    else
        echo -e "${RED}❌ VM build failed${NC}"
        exit 1
    fi
}

build_vm() {
    echo -e "${BLUE}Building test VM...${NC}"
    cd "$SCRIPT_DIR"
    nix build .#vm --no-link --out-link vm-result
    echo -e "${GREEN}✅ VM built successfully${NC}"
    echo "Run with: ./vm-result/bin/run-nixos-vm"
}

run_vm_manual() {
    echo -e "${BLUE}Starting VM for manual testing...${NC}"
    cd "$SCRIPT_DIR"
    
    # Ensure VM is built
    if [ ! -L "./vm-result" ]; then
        echo "Building VM first..."
        nix build .#vm --no-link --out-link vm-result
    fi
    
    # Create test results directory
    mkdir -p ./test-results
    
    echo ""
    echo -e "${YELLOW}Manual Testing Instructions:${NC}"
    echo "1. VM will auto-login as user 'testuser'"
    echo "2. Hyprland will start automatically"
    echo "3. Open applications with Super+Q (terminal), Super+E (firefox)"
    echo "4. Test hyprsession commands:"
    echo "   - hyprsession save <name>"
    echo "   - hyprsession load <name>"
    echo "   - hyprsession list"
    echo "5. Results in /shared/test-results/ (mapped to ./test-results/)"
    echo ""
    echo "Starting VM..."
    
    TMPDIR=/tmp ./vm-result/bin/run-nixos-vm -virtfs local,path=./test-results,mount_tag=shared,security_model=none,id=shared
}

clean_artifacts() {
    echo -e "${BLUE}Cleaning test artifacts...${NC}"
    cd "$SCRIPT_DIR"
    
    rm -rf ./test-results
    rm -f ./result ./vm-result
    
    echo -e "${GREEN}✅ Cleaned test results and build artifacts${NC}"
}

enter_shell() {
    echo -e "${BLUE}Entering development shell...${NC}"
    cd "$SCRIPT_DIR"
    nix develop
}

main() {
    print_header
    check_dependencies
    
    case "${1:-run}" in
        "run"|"test"|"")
            run_test
            ;;
        "build")
            build_vm
            ;;
        "vm"|"manual")
            run_vm_manual
            ;;
        "clean")
            clean_artifacts
            ;;
        "shell"|"dev")
            enter_shell
            ;;
        "help"|"-h"|"--help")
            print_usage
            ;;
        *)
            echo -e "${RED}Unknown command: $1${NC}"
            echo ""
            print_usage
            exit 1
            ;;
    esac
}

main "$@"