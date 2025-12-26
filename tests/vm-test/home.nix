{ pkgs, lib, ... }:
let
  # Test script that will run inside the VM
  testScript = pkgs.writeShellScriptBin "hyprsession-test" ''
    set -e

    RESULT_DIR="/shared/test-results"
    export HYPRSESSION_PATH="/shared/session-data"

    # Wait for Hyprland to be ready
    sleep 5
    
    # Check if we can communicate with Hyprland
    if ! hyprctl version > /dev/null 2>&1; then
      echo "ERROR: Hyprland is not responding"
      exit 1
    fi
    
    echo "=== Starting Hyprsession Integration Test ==="
    echo "hyprsession data path: $HYPRSESSION_PATH"

    hyprsession command coda-qt "flatpak run org.colabora.Office"
    
    # Check if a session already exists
    if hyprsession list | grep -q "test-session"; then
      echo "Found existing test session, cleaning up..."
      hyprsession delete test-session || true
    fi

    rm -rf ~/.mozilla/firefox
    echo "{ \"SKIP_UPDATE_CHECK\": true }" > ~/.config/discord/settings.json    

    echo "=== Phase 1: Loading test applications ==="

    cat /shared/exec.conf | while read -r line; do
      echo "Starting application: $line"
      hyprctl dispatch exec $line
      sleep 1
    done

    sleep 60

    echo "=== Phase 2: Capturing initial state ==="        
    hyprctl clients -j > $RESULT_DIR/expected.json

    echo "=== Phase 3: Saving session ==="
    
    # Save the current session
    echo "Saving session as 'test-session'..."
    hyprsession save test-session
    
    # Verify session was saved
    if ! hyprsession list | grep -q "test-session"; then
     echo "ERROR: Session was not saved properly"
      exit 1
    fi
    
    echo "Session saved successfully!"
    hyprsession list
    
    # echo "=== Phase 4: Clearing workspace ==="
    
    # Close all windows
    echo "Clearing all windows from workspace..."
    hyprsession clear
    sleep 5

    # Verify workspace is clear
    echo "Workspace after cleanup:"
    while hyprctl clients|grep "No clients found" > /dev/null 2>&1; do
      echo "All windows closed."
      break
    done

    echo "=== Phase 5: Loading session ==="
    
    # Load the saved session
    echo "Loading saved session..."
    hyprsession load test-session
    
    # Wait for applications to start
    sleep 10
    
    echo "=== Phase 6: Capturing final state ==="
    
    hyprctl clients -j > $RESULT_DIR/actual.json

    echo "=== Phase 7: Analysis ==="
    
    # Compare the results
    echo "Comparing expected vs actual client states..."
    
    # Extract just the application names for comparison
    function process_json() {
      cat "$1" | jq 'sort_by(.title) | .[] | del(.pid) | del(.address) | del(.focusHistoryID)'|jq -s . > "$2"
    }

    process_json $RESULT_DIR/expected.json $RESULT_DIR/expected_classes.json
    process_json $RESULT_DIR/actual.json $RESULT_DIR/actual_classes.json

    # Check if we have similar applications restored
    if jd $RESULT_DIR/expected_classes.json $RESULT_DIR/actual_classes.json > $RESULT_DIR/diff.txt; then
      echo "✅ SUCCESS: Session restored correctly!"
      echo "PASS" > $RESULT_DIR/result.txt
    else
      echo "⚠️  DIFFERENCES FOUND:"
      cat $RESULT_DIR/diff.txt
      echo "PARTIAL" > $RESULT_DIR/result.txt
    fi
    
    hyprctl dispatch exec kitty

    echo "=== Test Complete ==="
    echo "Results saved in $RESULT_DIR/"
  '';
in {
  # Home Manager configuration
  home = {
    username = "testuser";
    homeDirectory = "/home/testuser";
    stateVersion = "25.11";
  };
      
  # Configure Hyprland via Home Manager
  xdg.configFile."hypr/hyprland.conf" = {
    enable = true;
    force = true;
    text = ''
      monitor = Virtual-1,1920x1080@60,0x0,1
      input {
        kb_layout = gb
        follow_mouse = 1
      }
      
      general {
        gaps_in = 5
        gaps_out = 20
        border_size = 2
        col.active_border = rgba(33ccffee) rgba(00ff99ee) 45deg
        col.inactive_border = rgba(595959aa)
        layout = dwindle
      }
      
      decoration {
        rounding = 10
        blur {
          enabled = true
          size = 3
          passes = 1
        }
      }

      bind = CTRL SUPER, Q, exec, kitty
      bind = CTRL SUPER, C, killactive,
      bind = CTRL SUPER, M, exit,
      bind = CTRL SUPER, E, exec, firefox
      bind = CTRL SUPER, V, togglefloating,
      bind = CTRL SUPER, R, exec, rofi -show drun
      bind = CTRL SUPER, P, pseudo,
      bind = CTRL SUPER, J, togglesplit,

      bind = CTRL SUPER, left, movefocus, l
      bind = CTRL SUPER, right, movefocus, r
      bind = CTRL SUPER, up, movefocus, u
      bind = CTRL SUPER, down, movefocus, d

      bind = CTRL SUPER, 1, workspace, 1
      bind = CTRL SUPER, 2, workspace, 2
      bind = CTRL SUPER, 3, workspace, 3

      bind = CTRL SUPER SHIFT, 1, movetoworkspace, 1
      bind = CTRL SUPER SHIFT, 2, movetoworkspace, 2
      bind = CTRL SUPER SHIFT, 3, movetoworkspace, 3

      bind = CTRL SUPER, mouse_down, workspace, e+1
      bind = CTRL SUPER, mouse_up, workspace, e-1

      exec-once = ${testScript}/bin/hyprsession-test &> /shared/test-results/hyprsession-test.log ;
    '';
  };
}