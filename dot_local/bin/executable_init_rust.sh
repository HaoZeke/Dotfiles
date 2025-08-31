#!/usr/bin/env sh

# Installer
command -v rustup >/dev/null || {
    echo "Installing rustup" &&
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
}

# Grab the nightly
command -v rustc >/dev/null || {
    rustup install nightly
}

# Install things
# No OpenSSL + Requires OpenSSL/OpenSSH
# Most of these are now with x cmd but the ones with features need to be here
set -- "ripgrep --features pcre2" \
    "toml-cli"
for item in "$@"; do
    cargo binstall "$item"
done

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
