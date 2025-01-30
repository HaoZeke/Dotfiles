######################
# Activation Scripts #
######################

# Rust
#######

# Almost all of these are with cargo install blah blah

if $BASH; then
    if command -v zoxide &> /dev/null; then
        eval "$(zoxide init bash)"
    else
        echo "zoxide not found. Install it with 'cargo install zoxide' or your package manager."
    fi

    if command -v starship &> /dev/null; then
        eval "$(starship init bash)"
    else
        echo "starship not found. Install it with 'cargo install starship' or your package manager."
    fi

    if command -v mcfly &> /dev/null; then
        eval "$(mcfly init bash)"
    else
        echo "mcfly not found. Install it with 'cargo install mcfly' or your package manager."
    fi
fi
