#!/bin/bash

# Start calibre and wait
calibre
wait $!

# Setup and run sync
RCLONE_PASSWORD_COMMAND='secret-tool lookup service "rclone" user "calisync"'
rclone sync -P ~/CalibreLibs/Synced pCloud:/Calibre --ask-password=false
