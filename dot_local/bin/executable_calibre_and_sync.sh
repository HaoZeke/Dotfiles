#!/bin/bash
# $HOME/.local/bin/calibre_and_sync.sh

# Start Calibre and wait
calibre

# Setup and run sync
RCLONE_PASSWORD_COMMAND='secret-tool lookup service "rclone" user "calisync"'

(
  rclone sync -P ~/CalibreLibs/Synced pCloud:/Calibre --ask-password=false 2>&1 | \
  while read -r line; do
    # Check if the line contains the "Checks" status
    if [[ $line == *"Checks:"* ]]; then
      # Extract the percentage from the "Checks" line
      percent=$(echo $line | awk -F ',' '{print $2}' | grep -oP '\s+\K[0-9]+(?=%)')
      echo $percent
    fi
    echo "# $line"
  done
) |
zenity --progress \
  --title="Syncing Calibre Library with pCloud" \
  --text="Syncing..." \
  --percentage=0 \
  --auto-close \
  --auto-kill
