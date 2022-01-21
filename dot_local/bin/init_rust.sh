#!/usr/bin/env sh

# Installer
command -v rustup > /dev/null || { \
    echo "Installing rustup" && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh \
}

# Grab the nightly
command -v rustc > /dev/null || { \
    rustup install nightly \
}

# Install things
# No OpenSSL + Requires OpenSSL/OpenSSH
set -- "bottom" "ripgrep" "hyperfine" "skim" "mcfly" \
    "bat" "exa" "fd-find" "tealdeer" "starship" "sd" \
    "procs" "zoxide" "bliss" "navi"
for item in "$@"; do
    cargo install $item;
done

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
