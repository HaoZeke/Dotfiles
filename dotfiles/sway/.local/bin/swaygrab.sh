#!/bin/sh

# Kanged from https://github.com/NicholasAsimov/dotfiles/blob/master/scripts/swaygrabselection

set -e

filepath=$1

if [ -z "$filepath" ]; then
    echo "Usage: $(basename "$0") filepath"
    echo "If you specify \"clipboard\" as the file path the image will be copied to primary clipboard."
    exit 1
fi

if [[ "$filepath" == "clipboard" ]]; then
    slurp | grim -g - - | xclip -selection clipboard -t image/png -i
    exit 0
fi

slurp | grim -g - $filepath
