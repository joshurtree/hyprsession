# Hyprsession
## Overview
Implements session saving for Hyprland. While the program is running it periodicly saves the command, workspace and other properties of running clients found by `hyprctl clients`. These are then saved to a file formatted as a Hyprland config file which can then be sourced so that the session is restored when Hyprland is restarted.

## Installation
As root run the command 
```
cargo install --root /usr/local/bin hyprsession
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
This allows the user to save the session config in an alternative directory, by default its ~/.local/share/hyprsession. 

## TODO
* Create and use a rules file for alternative handling of applications (i.e. do not reload, ignore parameters, additional parameters etc).
* Handle application that create windows in forked processes by creating temporary window rules.

## Change log
### 0.1.1
* Changed --session-path option to point at base directory of session file
### 0.1.2
* Fixed bug which would crash program if no session file existed
