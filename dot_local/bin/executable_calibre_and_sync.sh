#!/bin/bash
# $HOME/.local/bin/calibre_and_sync.sh

# Start Calibre and wait
calibre

# Setup and run sync
zenity --notification --text="Start syncing"
RCLONE_PASSWORD_COMMAND='secret-tool lookup service "rclone" user "calisync"'
foot -- sh -c "rclone sync -P ~/CalibreLibs/Synced pCloud:/Calibre --ask-password=false --password-command=\"$RCLONE_PASSWORD_COMMAND\""
zenity --notification --text="Sync completed"
