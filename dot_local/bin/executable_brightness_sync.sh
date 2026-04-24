#!/usr/bin/env bash
# brightness_sync.sh: Sync brightness across internal and external displays
# Usage: brightness_sync.sh AMT WOBSOCK

# Parameters
BRIGHTNESS_CHANGE=$1
WOBSOCK=$2
CACHE_FILE="/tmp/ddc_monitors_cache"
LOCK_FILE="/tmp/brightness_sync_ddc.lock"

# 1. IMMEDIATE ACTION: Internal Screen & UI
# Always update on every press so the laptop feels responsive.
NEW_BRIGHTNESS=$(brightnessctl set "$BRIGHTNESS_CHANGE" -m | cut -d, -f4 | tr -d '%')

if [ -z "$NEW_BRIGHTNESS" ]; then
    exit 1
fi

# Update WOBSOCK immediately if it is the expected FIFO
if [ -p "$WOBSOCK" ]; then
    echo "$NEW_BRIGHTNESS" > "$WOBSOCK"
fi

# 2. External monitors via DDC/CI: serialize with a non-blocking flock.
# A burst of F5/F6 presses must not pile up I2C traffic, so any press that
# arrives while a prior round is still running is dropped here. The internal
# panel above still tracked every press.
(
    exec 9>"$LOCK_FILE"
    flock -n 9 || exit 0

    # Monitor discovery (lazy). An empty file means we scanned and found
    # no external screens; respect that and do not re-scan.
    if [ ! -f "$CACHE_FILE" ]; then
        ddcutil detect --terse | \
            awk '/^Display/ {valid=1} /^Invalid/ {valid=0} valid && /I2C bus:/ {print $NF}' | \
            sed 's/.*i2c-//' > "$CACHE_FILE"
    fi

    while read -r bus_id; do
        ddcutil setvcp 10 "$NEW_BRIGHTNESS" --bus "$bus_id" --noverify
    done < "$CACHE_FILE"
) &

# TODO(rg): write up as a short post / blog
# Addenum :: udev hotplug
# sudo nano /etc/udev/rules.d/99-ddc-cache-reset.rules
# ACTION=="change", SUBSYSTEM=="drm", ENV{HOTPLUG}=="1", RUN+="/usr/bin/rm -f /tmp/ddc_monitors_cache"
# sudo udevadm control --reload-rules && sudo udevadm trigger
# To control
# ls -l /tmp/ddc_monitors_cache
# Could alternatively use kanshi but that needs a profile for every single one
