######################
# Activation Scripts #
######################

# Rust
#######

# Almost all of these are with cargo install blah blah
programs=(zoxide atuin starship)

for prog in "${programs[@]}"; do
    if command -v "$prog" &>/dev/null; then
        eval "$("$prog" init bash)"
    fi
done

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
