######################
# Activation Scripts #
######################

# Rust
#######

# https://www.x-cmd.com/
[ ! -f "$HOME/.x-cmd.root/X" ] || . "$HOME/.x-cmd.root/X"

# ble.sh isn't part of x-cmd
BLE_SH_PATH="$HOME/.local/share/blesh/ble.sh"
if [ -f "$BLE_SH_PATH" ]; then
  source "$BLE_SH_PATH"
fi

# Almost all of these are with cargo install blah blah
programs=(zoxide starship mcfly)

for prog in "${programs[@]}"; do
    if command -v "$prog" &>/dev/null; then
        eval "$("$prog" init bash)"
    fi
done

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
