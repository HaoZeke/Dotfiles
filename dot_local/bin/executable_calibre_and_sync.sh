#!/usr/bin/env bash
# $HOME/.local/bin/calibre_and_sync.sh
#
# Start Calibre, and when it closes push the synced library to pCloud.
# The library lives in ~/CalibreLibs/Synced and the remote is the rclone
# remote `pCloud`, path /Calibre. `calisync-restore` pulls the other way
# on a fresh machine.
set -u
lib="$HOME/CalibreLibs/Synced"
remote="pCloud:/Calibre"

notify() { command -v zenity >/dev/null && zenity --notification --text="$1" || echo "calisync: $1"; }

if ! command -v calibre >/dev/null; then
    notify "calibre is not installed (pacman -S calibre)"; exit 1
fi
if ! rclone listremotes 2>/dev/null | grep -qx 'pCloud:'; then
    notify "rclone has no pCloud remote (rclone config create pCloud pcloud)"; exit 1
fi
# A library that is not here yet must be pulled, never pushed: an empty
# local tree would sync the remote away.
if [ ! -f "$lib/metadata.db" ]; then
    notify "no library at $lib; run calisync-restore first"; exit 1
fi

# Start Calibre and wait
calibre --with-library "$lib"

# Setup and run sync
notify "Start syncing"
# An encrypted rclone.conf keeps its password in the keyring; a plain one needs none.
pw=()
if secret-tool lookup service rclone user calisync >/dev/null 2>&1; then
    pw=(--ask-password=false --password-command='secret-tool lookup service rclone user calisync')
fi
foot -- sh -c "rclone sync -P --bwlimit 2M --exclude '.calnotes/**' '$lib' '$remote' ${pw[*]}"
notify "Sync completed"
