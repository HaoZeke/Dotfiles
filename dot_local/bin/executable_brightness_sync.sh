#!/usr/bin/env bash
# brightness_sync.sh: Sync brightness across internal and external displays
# Usage: brightness_sync.sh AMT WOBSOCK

# Parameters
BRIGHTNESS_CHANGE=$1
WOBSOCK=$2
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp}"
case "$RUNTIME_DIR" in
    /run/user/*|/tmp) ;;
    *)
        echo "brightness_sync.sh: refusing unsafe runtime directory: $RUNTIME_DIR" >&2
        exit 1
        ;;
esac
CACHE_DIR="$RUNTIME_DIR/brightness-sync"
mkdir -p -m 700 "$CACHE_DIR"
CACHE_FILE="$CACHE_DIR/ddc_monitors_cache"
LOCK_FILE="$CACHE_DIR/ddc.lock"

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

# Delete "$CACHE_FILE" after monitor topology changes to force DDC
# rediscovery.
