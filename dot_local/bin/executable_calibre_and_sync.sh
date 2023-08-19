#!/bin/bash
# $HOME/.local/bin/calibre_and_sync.sh

# Start Calibre and wait
calibre

# Setup and run sync
zenity --notification --text="Start syncing"
RCLONE_PASSWORD_COMMAND='secret-tool lookup service "rclone" user "calisync"'
(
  rclone sync -P ~/CalibreLibs/Synced/Study pCloud:/Calibre/Study --ask-password=false \
      --password-command="$RCLONE_PASSWORD_COMMAND" 2>&1 | \
  while read -r line; do
    echo "# $line"
  done
) |
zenity --progress \
  --title="Syncing Calibre Library with pCloud" \
  --text="Syncing..." \
  --percentage=0 \
  --pulsate \
  --auto-close \
  --auto-kill
zenity --notification --text="Sync completed"
