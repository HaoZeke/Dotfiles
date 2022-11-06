######################
# Activation Scripts #
######################

# Rust
#######

# Almost all of these are with cargo install blah blah

# TODO: Update to test that these exist
if $BASH; then
    eval "$(zoxide init bash)"
    eval "$(starship init bash)"
    eval "$(mcfly init bash)"
fi
