######################
# Activation Scripts #
######################

# Rust
#######

# Almost all of these are with cargo install blah blah

if command -v zoxide &> /dev/null; then
    eval "$(zoxide init bash)"
fi

if command -v starship &> /dev/null; then
    eval "$(starship init bash)"
fi

if command -v mcfly &> /dev/null; then
    eval "$(mcfly init bash)"
fi
