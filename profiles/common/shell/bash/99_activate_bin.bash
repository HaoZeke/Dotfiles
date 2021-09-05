######################
# Activation Scripts #
######################
if [ "$EVAL_BASH" == "yes" || "$EVAL_BASH" == "true" ]; then

# Misc.
eval "$(direnv hook bash)"

# Rust
#######

# Almost all of these are with cargo install blah blah

# TODO: Update to test that these exist
eval "$(zoxide init bash)"
eval "$(starship init bash)"
eval "$(mcfly init bash)"

fi
