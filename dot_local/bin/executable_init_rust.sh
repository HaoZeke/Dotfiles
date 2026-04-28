#!/usr/bin/env sh

# Installer
command -v rustup >/dev/null || {
    echo "Installing rustup" &&
        rustup_installer=$(mktemp /tmp/rustup-init.XXXXXX) &&
        trap 'rm -f "$rustup_installer"' EXIT &&
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$rustup_installer" &&
        sh "$rustup_installer"
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
