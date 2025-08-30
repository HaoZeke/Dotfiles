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
set -- "bottom" "ripgrep --features pcre2" "hyperfine" "skim" "mcfly" \
    "bat" "exa" "fd-find" "tealdeer" "starship" "sd" \
    "procs" "zoxide" "bliss" "git-delta" "watchexec-cli" \
    "du-dust" "toml-cli" "amber" "hexyl" "tokei" "typos-cli" \
    "silicon"
for item in "$@"; do
    cargo binstall "$item"
done

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
