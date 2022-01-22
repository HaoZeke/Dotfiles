#!/usr/bin/env bash

dumbDown() {
  if command -v aria2c >/dev/null 2>&1; then
    aria2c -j2 "$1"
  elif command -v wget >/dev/null 2>&1; then
    wget -v -c "${1}"
    wget --trust-server-names -v -c "${1}"
  else
    curl -L -J -O "${1}" --progress-bar
  fi
}

if [[ ! -d $HOME/.tmux/plugins/tpm ]]; then
  echo "You have tmux so you're getting tpm"
  if command -v git >/dev/null 2>&1; then
    git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm
  else
    mkdir -p ~/.tmux/plugins
    cd ~/.tmux/plugins
    dumbDown https://github.com/tmux-plugins/tpm/archive/master.zip
    unzip tpm-master.zip
    mv tpm-master tpm
  fi
fi
