#!/usr/bin/env bash
# Although I no longer recommend asdf, (shims are awful)
# This is the fastest way to get direnv up and running

if [[ ! -d ~/.asdf ]]; then
    mkdir ~/.asdf
    git clone https://github.com/asdf-vm/asdf.git ~/.asdf
    export PATH=$PATH:$HOME/.asdf/bin
fi

export ASDF_CONFIG_FILE="~/.config/asdf/asdfrc"

asdf plugin-add direnv
asdf install direnv latest
asdf global direnv latest
