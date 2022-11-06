#!/usr/bin/env bash

if [[ ! -d ~/.zi ]]; then
    typeset -Ag ZI
    typeset -gx ZI[HOME_DIR]="${HOME}/.zi" ZI[BIN_DIR]="${ZI[HOME_DIR]}/bin"
    command mkdir -p "$ZI[BIN_DIR]"

    compaudit | xargs chown -R "$(whoami)" "$ZI[HOME_DIR]"
    compaudit | xargs chmod -R go-w "$ZI[HOME_DIR]"
    command git clone https://github.com/z-shell/zi.git "$ZI[BIN_DIR]"
fi
