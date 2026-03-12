#!/usr/bin/env bash
# brightness_sync.sh: Sync brightness across internal and external displays
# Usage: brightness_sync.sh AMT WOBSOCK

# Parameters
BRIGHTNESS_CHANGE=$1
WOBSOCK=$2
CACHE_FILE="/tmp/ddc_monitors_cache"

# 1. Clean up concurrency
# Kill previous instances to prevent I2C bus pile-ups.
pgrep -f "brightness_sync.sh" | grep -v $$ | xargs -r kill

# 2. IMMEDIATE ACTION: Internal Screen & UI
# We do this first so the laptop feels responsive instantly.
NEW_BRIGHTNESS=$(brightnessctl set "$BRIGHTNESS_CHANGE" -m | cut -d, -f4 | tr -d '%')

if [ -z "$NEW_BRIGHTNESS" ]; then
    exit 1
fi

# Update WOBSOCK immediately if it exists
if [ -n "$WOBSOCK" ]; then
    echo "$NEW_BRIGHTNESS" > "$WOBSOCK"
fi

# 3. Monitor Discovery (Lazy Loading)
# Change: We use -f (file exists) instead of -s (file has content).
# If the file exists but is empty, it means we scanned and found NO screens.
# We respect that empty state and do not re-scan.
if [ ! -f "$CACHE_FILE" ]; then
    ddcutil detect --terse | \
    awk '/^Display/ {valid=1} /^Invalid/ {valid=0} valid && /I2C bus:/ {print $NF}' | \
    sed 's/.*i2c-//' > "$CACHE_FILE"
fi

# 4. Adjust External Monitors (Background)
# If CACHE_FILE is empty (no screens), this loop simply doesn't run.
while read -r bus_id; do
    ddcutil setvcp 10 "$NEW_BRIGHTNESS" --bus "$bus_id" --noverify &
done < "$CACHE_FILE"

# TODO(rg): write up as a short post / blog
# Addenum :: udev hotplug
# sudo nano /etc/udev/rules.d/99-ddc-cache-reset.rules
# ACTION=="change", SUBSYSTEM=="drm", ENV{HOTPLUG}=="1", RUN+="/usr/bin/rm -f /tmp/ddc_monitors_cache"
# sudo udevadm control --reload-rules && sudo udevadm trigger
# To control
# ls -l /tmp/ddc_monitors_cache
# Could alternatively use kanshi but that needs a profile for every single one
