# Common zinit stuff
######################

# OMZ Plugins
zinit snippet OMZ::plugins/git/git.plugin.zsh
zinit snippet OMZ::plugins/history/history.plugin.zsh
zinit snippet OMZ::plugins/github/github.plugin.zsh
zinit ice svn
zinit snippet OMZ::plugins/history-substring-search
zinit snippet OMZ::plugins/systemadmin
zinit snippet OMZ::plugins/sudo
zinit snippet OMZ::plugins/systemd
zinit snippet OMZ::plugins/rsync
zinit snippet OMZ::plugins/common-aliases/common-aliases.plugin.zsh
zinit ice silent wait"0"
zinit snippet OMZ::plugins/per-directory-history/per-directory-history.zsh
# Handlers
# zinit snippet OMZ::plugins/pyenv
# zinit snippet OMZ::plugins/rbenv

# Colors and Highlighting
zinit light "zdharma/fast-syntax-highlighting"
zinit light "zlsun/solarized-man"

# Nix shell
zinit light "chisui/zsh-nix-shell"

# Misc
zinit ice proto'git' pick'init.sh'
zinit light "b4b4r07/enhancd"
zinit light "changyuheng/zsh-interactive-cd"
zinit light "mollifier/cd-gitroot"
zinit ice proto'git' pick'ssh-agent.zsh'
zinit light "bobsoppe/zsh-ssh-agent"
zinit light "zdharma/history-search-multi-word"
zinit light "urbainvaes/fzf-marks"

# Manage NVM better (needs settings before being loaded)
export NVM_LAZY_LOAD=true
zinit light "lukechilds/zsh-nvm"

# Completions
zinit ice blockf
zinit light "zsh-users/zsh-completions"
zinit light "ascii-soup/zsh-url-highlighter"
zinit light "molovo/tipz"
zinit light "srijanshetty/zsh-pip-completion"
zinit ice as"completion"
zinit snippet https://github.com/simonwhitaker/gibo/blob/master/shell-completions/gibo-completion.zsh

# Suggestions
zinit ice wait lucid atload'_zsh_autosuggest_start'
zinit light zsh-users/zsh-autosuggestions

############
# Programs #
############
# junegunn/fzf-bin
zinit ice from"gh-r" as"program"
zinit light junegunn/fzf-bin

# sharkdp/fd
# zinit ice as"command" from"gh-r" mv"fd* -> fd" pick"fd/fd"
# zinit light sharkdp/fd

# sharkdp/bat
# zinit ice as"command" from"gh-r" mv"bat* -> bat" pick"bat/bat"
# zinit light sharkdp/bat

# ogham/exa, replacement for ls
# zinit ice wait"2" lucid from"gh-r" as"program" mv"exa* -> exa"
# zinit light ogham/exa

# mvdan/sh
zinit ice from"gh-r" as"program" mv"shfmt* -> shfmt"
zinit light mvdan/sh

# Archive helper
zinit ice as"command" from"gh-r" mv"archiver* -> archiver" pick"archiver/archiver"
zinit light mholt/archiver

# Makefile alternative
zinit ice as"command" from"gh-r" bpick"*.tar.gz" mv"task* -> task" pick"task/task"
zinit light "go-task/task"

# Better git
zinit ice as"program" make pick"bin/git-fzf.zsh"
zinit light 'b4b4r07/git-fzf'

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
