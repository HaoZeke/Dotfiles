######################
# Activation Scripts #
######################
# if [[ "$EVAL_ZSH" == "yes" || "$EVAL_ZSH" == "true" ]]; then

# Misc.
eval "$(asdf exec direnv hook zsh)"
direnv() { asdf exec direnv "$@"; }

# Rust
#######

# Almost all of these are with cargo install blah blah

# TODO: Update to test that these exist
eval "$(zoxide init zsh)"
eval "$(starship init zsh)"
eval "$(mcfly init zsh)"

# fi
