#!/bin/sh

# Kanged from https://github.com/NicholasAsimov/dotfiles/blob/master/scripts/swaygrabselection
# Updated by me (HaoZeke)

set -e

filepath=$1

if [ -z "$filepath" ]; then
    echo "Usage: $(basename "$0") filepath"
    echo "If you specify \"clipboard\" as the file path the image will be copied to primary clipboard."
    echo "Creating a time-stamped image"
    slurp | grim -g - $(xdg-user-dir PICTURES)/Pictures/Screenshots/$(date +'%Y-%m-%d-%H%M%S_grim.png')
    exit 0
fi

if [[ "$filepath" == "clipboard" ]]; then
    echo "Copying image to the primary clipboard"
    slurp | grim -g - - | xclip -selection clipboard -t image/png -i
    exit 0
fi

slurp | grim -g - $filepath
