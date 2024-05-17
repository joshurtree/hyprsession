#!/usr/bin/python
#
# Session saver for Hyprland. 
# (This is an archived version of the original python script)
#
# Author: Josh Andrews
# Version: 0.01
#
# Released under the GPL v3 licence

import json
import subprocess
import re
import types
import os.path
import time
import argparse


CLIENT_INFO_CMD = "hyprctl -j clients"
PS_CMD = "ps --no-headers -o cmd -p "
SHARE_DIR = os.environ["HOME"] + "/.local/share/hyprsession"
SESSION_PATH = SHARE_DIR + "/session.conf"

def fetchCommand(info) :
    output = subprocess.Popen(PS_CMD + str(info["pid"]), shell=True, stdout=subprocess.PIPE)
    return output.communicate()[0].decode("utf-8")

def make_lambdas(vals) :
    make_lambda = lambda val : val if type(val) is types.LambdaType else lambda info : val
    retVal = [make_lambda(val) for val in vals]
    return retVal

PROPS = [make_lambdas(val) for val in [
    (lambda info : f"monitor {info['monitor']}", True),
    (lambda info : f"workspace {info['workspace']['id']} silent", True),
    ("float", lambda info : info["floating"]),
    (lambda info : f"move {info['at']['x']} {info['at']['y']}", lambda info : info["floating"]),
    (lambda info : f"size {info['size']['width']} {info['size']['height']}", lambda info : info["floating"]),
    ("pin", lambda info : info["pinned"]),
    ("fullscreen", lambda info : info["fullscreen"]),
    ("fakefullscreen", lambda info : info["fakefullscreen"])
]]

def save_session(session_path) :
    output = subprocess.Popen(CLIENT_INFO_CMD, shell=True, stdout=subprocess.PIPE)
    clientInfo = json.loads(output.communicate()[0])

    session_config = open(session_path, "w")

    for info in clientInfo :
        exec_opts = [val[0](info) for val in PROPS if val[1](info)]
        session_config.write(f"exec-once = [{';'.join(exec_opts)}] {fetchCommand(info)}")

    session_config.close()

parser = argparse.ArgumentParser(description='Save Hyprland sessions.')
parser.add_argument('-o', '--save-once', action='store_true', help='Save session and exit. (implies --save-immediately)')
parser.add_argument('-s', '--save-immediately', action='store_true', help='Saves session immediately after starting program')
parser.add_argument('-i', '--save-interval', type=int, default=60, help='Specifies interval in seconds between saving session')
parser.add_argument('-p', '--session-path', type=str, default=SESSION_PATH, help='Specify alternative path for the session file')
args = parser.parse_args()

if args.save_interval < 1 :
    print("Save interval needs to be a positive integer")
    exit(1)

if not os.path.exists(SHARE_DIR) :
    os.mkdir(SHARE_DIR)

try :
    open(args.session_path, 'w')
    session_path = args.session_path
except IOError:
    print("Cannot create session file")
    exit(1)
except AttributeError :
    pass # use default path

if args.save_once or args.save_immediately :
    save_session(session_path)
    if args.save_once :
        exit(0)

while True :
    time.sleep(args.save_interval)
    save_session(session_path)
