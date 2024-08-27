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
BRIGHTNESS_LEVEL=$(brightnessctl set "$BRIGHTNESS_CHANGE" | sed -En 's/.*\(([0-9]+)%\).*/\1/p')

# Check if the brightness level was successfully retrieved
if [ -z "$BRIGHTNESS_LEVEL" ]; then
  echo "Error: Failed to retrieve the brightness level"
  exit 1
fi

# Set the brightness for external monitors using ddcutil
# Get the bus via ddcutil detect
ddcutil --dsa --bus 1 setvcp 10 "$BRIGHTNESS_LEVEL"

# Optionally, output the brightness level to WOBSOCK if wob is being used
if [ -n "$WOBSOCK" ]; then
  echo "$BRIGHTNESS_LEVEL" > "$WOBSOCK"
fi
