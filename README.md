# Hyprsession
## Overview
Implements session saving for Hyprland. While the program is running it periodicly saves the command, workspace and other properties of running clients found by `hyprctl clients`. These are then saved to a file formatted as a Hyprland config file which can then be sourced so that the session is restored when Hyprland is restarted.

## Installation
Run the command 
```
cargo install hyprsession
``` 
Then add the following line to your Hyprland config file (Usually at ~/.config/hypr/hyprland.conf)
```
exec_once = hyprsession &
```

## Options
Various options can be used to modify the behavior of Hyprsession.

### --mode <mode>
Sets the mode the program runs in 
* Default - Loads the session at startup the saves the current session at regular intervals.
* SaveOnly - As above but skips loading the session
* LoadAndExit - Load the saved session then immediatly exit
* SaveAndExit - Save the current session then exit

### --save-interval n
This sets the interval in seconds between session saves. The default is 60 seconds.

### --session-path
This allows the user to save the session config in an alternative location if desired. If used then ensure that the path following `source =` above points to the correct location.

## TODO
* Create and use a rules file for alternative handling of awkward applications.
* Handle application that create windows in forked processes by creating temporary window rules.
* Save geometry of windows.
