typeset -g POWERLEVEL9K_INSTANT_PROMPT=quiet
# Enable Powerlevel10k instant prompt. Should stay close to the top of ~/.zshrc.
# Initialization code that may require console input (password prompts, [y/n]
# confirmations, etc.) must go above this block; everything else may go below.
if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi

# Shell Theme
###############

# Power level 10k #

zplugin ice depth=1; zplugin light romkatv/powerlevel10k

# source ~/.shell_prompt.sh
POWERLEVEL9K_MODE='nerdfont-complete'
POWERLEVEL9K_VCS_CLEAN_FOREGROUND='blue'
POWERLEVEL9K_VCS_CLEAN_BACKGROUND='black'
POWERLEVEL9K_VCS_UNTRACKED_FOREGROUND='yellow'
POWERLEVEL9K_VCS_UNTRACKED_BACKGROUND='black'
POWERLEVEL9K_VCS_MODIFIED_FOREGROUND='blue'
POWERLEVEL9K_VCS_MODIFIED_BACKGROUND='black'

# Control knobs
# POWERLEVEL9K_PYENV_PROMPT_ALWAYS_SHOW='true'
# POWERLEVEL9K_RBENV_PROMPT_ALWAYS_SHOW='true'

# Right
POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=(status ssh root_indicator background_jobs time)

# Left
POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(dir newline asdf direnv nvm pyenv rbenv virtualenv vcs nix_shell)

# Bullet Train #

# zplug "caiogondim/bullet-train.zsh", use:bullet-train.zsh-theme, defer:3 # defer until other plugins like oh-my-zsh is loaded

# Spaceship #

# zplug denysdovhan/spaceship-prompt, use:spaceship.zsh, from:github, as:theme

# To customize prompt, run `p10k configure` or edit ~/.p10k.zsh.
[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh
