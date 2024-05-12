# Hyprsession
## Overview
Implements session saving for Hyprland. While the program is running it periodicly saves the command, workspace and other properties of running clients found by `hyprctl clients`. These are then saved to a file formatted as a Hyprland config file which can then be sourced so that the session is restored when Hyprland is restarted.

## Installation
In a terminal switch to a directory where you want store the program. Then run the command (using `sudo` or `doas` if necassary)
```
curl https://raw.githubusercontent.com/joshurtree/hyprsession/main/hyprsession -o hyprsession && chmod +x hyprsession
```
Then add the following lines to your Hyprland config file (Usually at ~/.config/hypr/hyprland.conf)
```
exec_once = hyprsession &
source = ~/.local/share/hyprsession/session.conf
```
The first line ensures that the session is saved at regular intervals (every 60 seconds by default) and the second includes the file where the session is stored in the configuration so that it is loaded at startup.

## Options
Various options can be used to modify the behavior of Hyprsession.

### --save-once
This can be used to save the current session before immediatly exiting.

### --save-immmediately
Usually Hyprsession doesn't save the session until after the set interval (see --save-interval). This prevents it from overwriting the previous session when invoked at startup. If you are running an instance manually from a terminal then this option ensures that the session is immediately saved if needed.

### --save-interval n
This sets the interval in seconds between session saves. The default is 60 seconds.

### --session-path
This allows the user to save the session config in an alternative location if desired. If used then ensure that the path following `source =` above points to the correct location.

## TODO
* Create and use a rules file for alternative handling of awkward applications.
* Handle application that create windows in forked processes by creating temporary window rules.
* Save geometry of windows.
