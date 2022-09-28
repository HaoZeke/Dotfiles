# Common zinit stuff
######################

# OMZ Plugins
zinit snippet OMZP::git/git.plugin.zsh
zinit snippet OMZP::history/history.plugin.zsh
zinit snippet OMZP::github/github.plugin.zsh
zinit ice svn
zinit snippet OMZP::systemadmin
zinit snippet OMZP::sudo
zinit snippet OMZP::systemd
zinit snippet OMZP::rsync
zinit snippet OMZP::common-aliases/common-aliases.plugin.zsh
zinit ice silent wait"0"
zinit snippet OMZP::per-directory-history/per-directory-history.zsh

# History substring search
zinit light "zsh-users/zsh-history-substring-search"

# Colors and Highlighting
zinit light "zdharma-continuum/fast-syntax-highlighting"
zinit light "zlsun/solarized-man"

# Nix shell
zinit light "chisui/zsh-nix-shell"

# Misc
zinit light "mollifier/cd-gitroot"
zinit light "zdharma-continuum/history-search-multi-word"
zinit light "urbainvaes/fzf-marks"
zinit light "changyuheng/zsh-interactive-cd"
zinit ice proto'git' pick'init.sh'; zinit light b4b4r07/enhancd
zinit ice pick'ssh-agent.zsh'; zinit light bobsoppe/zsh-ssh-agent

# Completions
zinit ice blockf
zinit light "zsh-users/zsh-completions"
zinit light "ascii-soup/zsh-url-highlighter"
zinit light "molovo/tipz"
zinit light "srijanshetty/zsh-pip-completion"
zinit ice as"completion"
zinit snippet https://github.com/simonwhitaker/gibo/blob/main/shell-completions/gibo-completion.zsh

# Suggestions
zinit ice wait lucid atload'_zsh_autosuggest_start'
zinit light zsh-users/zsh-autosuggestions

# Settings for plugins #

# history-substring-search
# bind UP and DOWN arrow keys
zmodload zsh/terminfo
bindkey "$terminfo[kcuu1]" history-substring-search-up
bindkey "$terminfo[kcud1]" history-substring-search-down

# bind UP and DOWN arrow keys (compatibility fallback
# for Ubuntu 12.04, Fedora 21, and MacOSX 10.9 users)
bindkey '^[[A' history-substring-search-up
bindkey '^[[B' history-substring-search-down

# bind P and N for EMACS mode
bindkey -M emacs '^P' history-substring-search-up
bindkey -M emacs '^N' history-substring-search-down

# bind k and j for VI mode
bindkey -M vicmd 'k' history-substring-search-up
bindkey -M vicmd 'j' history-substring-search-down


# enhancd settings
# export ENHANCD_FILTER="fzf --height 50% --reverse --ansi --preview 'ls -l {}' --preview-window down"
export ENHANCD_FILTER="fzf --height 50% --reverse --ansi"
# export ENHANCD_DOT_SHOW_FULLPATH=1
TIPZ_TEXT='Alias:'


# Common Aliases (Plugin based)
################################

alias cdg=cd-gitroot
