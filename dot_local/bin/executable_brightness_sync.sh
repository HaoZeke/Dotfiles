#!/bin/bash
# brightness_sync.sh: Sync brightness across internal and external displays
# Usage: brightness_sync.sh AMT WOBSOCK

# Assign input parameters to variables
BRIGHTNESS_CHANGE=$1
WOBSOCK=$2

# Ensure that exactly two arguments are provided
if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <brightness_change> <wobsock>"
  exit 1
fi

# Adjust brightness using brightnessctl and capture any errors
if ! brightnessctl set "$BRIGHTNESS_CHANGE"; then
  echo "Error: Failed to adjust brightness using brightnessctl"
  exit 1
fi

# Extract the new brightness value
BRIGHTNESS_LEVEL=$(brightnessctl -m | awk -F, '{print $4}' | tr -d '%')

# Set the brightness for external monitors using ddcutil
# Get the bus via ddcutil detect
ddcutil --bus 1 setvcp 10 "$BRIGHTNESS_LEVEL"

# Optionally, output the brightness level to WOBSOCK if wob is being used
if [ -n "$WOBSOCK" ]; then
  echo "$BRIGHTNESS_LEVEL" > "$WOBSOCK"
fi
