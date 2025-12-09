#!/bin/bash
# brightness_sync.sh: Sync brightness across internal and external displays
# Usage: brightness_sync.sh AMT WOBSOCK

# Parameters
BRIGHTNESS_CHANGE=$1
WOBSOCK=$2
CACHE_FILE="/tmp/ddc_monitors_cache"

# 1. Kill previous instances to prevent I2C bus pile-ups
pgrep -f "brightness_sync.sh" | grep -v $$ | xargs -r kill

# 2. Monitor Discovery (Cached & Fixed)
# We regenerate the cache if it doesn't exist or is empty.
if [ ! -s "$CACHE_FILE" ]; then
    # We use awk to:
    # 1. Detect "Display" blocks (valid) vs "Invalid display" blocks.
    # 2. Extract the I2C bus path only from valid blocks.
    # 3. Use sed to strip everything but the number.
    ddcutil detect --terse |
        awk '/^Display/ {valid=1} /^Invalid/ {valid=0} valid && /I2C bus:/ {print $NF}' |
        sed 's/.*i2c-//' >"$CACHE_FILE"
fi

# 3. Adjust Internal Brightness (brightnessctl)
NEW_BRIGHTNESS=$(brightnessctl set "$BRIGHTNESS_CHANGE" -m | cut -d, -f4 | tr -d '%')

if [ -z "$NEW_BRIGHTNESS" ]; then
    exit 1
fi

# 4. Adjust External Monitors (ddcutil)
# Loop through the cached bus numbers (e.g., 1, 10)
while read -r bus_id; do
    # --bus avoids scanning
    # --noverify speeds up execution
    # & runs them in parallel
    ddcutil setvcp 10 "$NEW_BRIGHTNESS" --bus "$bus_id" --noverify &
done <"$CACHE_FILE"

# 5. Output to WOBSOCK
if [ -n "$WOBSOCK" ]; then
    echo "$NEW_BRIGHTNESS" >"$WOBSOCK"
fi
