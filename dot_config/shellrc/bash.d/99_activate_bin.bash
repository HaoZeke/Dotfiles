######################
# Activation Scripts #
######################
# Misc.
eval "$(asdf exec direnv hook bash)"
direnv() { asdf exec direnv "$@"; }

# Rust
#######

# Almost all of these are with cargo install blah blah

# TODO: Update to test that these exist
eval "$(zoxide init bash)"
eval "$(starship init bash)"
eval "$(mcfly init bash)"
