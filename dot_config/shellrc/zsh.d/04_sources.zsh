# Common zinit stuff
######################

# OMZ Plugins
zinit snippet OMZP::git/git.plugin.zsh
zinit snippet OMZP::github/github.plugin.zsh
zinit ice svn
zinit snippet OMZP::systemadmin
zinit snippet OMZP::sudo
zinit snippet OMZP::systemd
zinit snippet OMZP::rsync
zinit snippet OMZP::common-aliases/common-aliases.plugin.zsh
zinit ice silent wait"0"

# Colors and Highlighting
zinit light "zdharma-continuum/fast-syntax-highlighting"
zinit light "zlsun/solarized-man"

# Nix shell
zinit light "chisui/zsh-nix-shell"

# Misc
zinit light "mollifier/cd-gitroot"
zinit light "urbainvaes/fzf-marks"
zinit light "changyuheng/zsh-interactive-cd"
zinit ice pick'ssh-agent.zsh'; zinit light bobsoppe/zsh-ssh-agent

# Completions
zinit ice blockf
zinit light "zsh-users/zsh-completions"
zinit light "ascii-soup/zsh-url-highlighter"
zinit light "molovo/tipz"
zinit light "srijanshetty/zsh-pip-completion"

# Suggestions
zinit ice wait lucid atload'_zsh_autosuggest_start'
zinit light zsh-users/zsh-autosuggestions

# wakatime
zinit ice wait lucid depth=1 atload'export ZSH_WAKATIME_BIN="$(pyenv which wakatime-cli)"'; zinit light sobolevn/wakatime-zsh-plugin

# Settings for plugins #

# enhancd
zinit ice wait'1' lucid pick'init.sh'
zinit light "b4b4r07/enhancd"
# export ENHANCD_FILTER="fzf --height 50% --reverse --ansi --preview 'ls -l {}' --preview-window down"
export ENHANCD_COMPLETION_BEHAVIOR=list
export ENHANCD_FILTER="fzf --height 50% --reverse --ansi  --info=inline --margin=1 --padding=1"
# export ENHANCD_DOT_SHOW_FULLPATH=1
TIPZ_TEXT='Alias:'


# Common Aliases (Plugin based)
################################

alias cdg=cd-gitroot
