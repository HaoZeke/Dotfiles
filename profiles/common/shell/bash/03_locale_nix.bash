#!/usr/bin/env bash

if [[ "$HAS_NIX" == "yes" || "$HAS_NIX" == "true" ]]; then
    export LOCALE_ARCHIVE="$(nix-build --no-out-link "<nixpkgs>" -A glibcLocales)/lib/locale/locale-archive"
fi
