#!/usr/bin/env bash

{{ if eq .setup_nix "yes" }}

if [[ "$HAS_NIX" == "yes" || "$HAS_NIX" == "true" ]]; then
    export LOCALE_ARCHIVE="$(nix-build --no-out-link "<nixpkgs>" -A glibcLocales)/lib/locale/locale-archive"
fi

# Nix
if [[ $- = *i* ]]; then
    if [[ -f /etc/profile.d/nix.sh ]]; then
        . /etc/profile.d/nix.sh
    elif [[ -d /nix ]]; then
        if [[ ! $(uname)=="Darwin" ]]; then
            source "$HOME/.nix-profile/etc/profile.d/nix.sh"
        fi
    else
        if [[ -v DEBUG_RGDOTS ]]; then
            echo "You don't have nix installed"
        fi
    fi
fi

{{ end }}
