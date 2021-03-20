typeset -g POWERLEVEL9K_INSTANT_PROMPT=quiet
# Enable Powerlevel10k instant prompt. Should stay close to the top of ~/.zshrc.
# Initialization code that may require console input (password prompts, [y/n]
# confirmations, etc.) must go above this block; everything else may go below.
if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi

#######################
# Startup debugging   #
# tools for profiling #
#######################

# Start Profiler
# Test with:
# time  zsh -i -c exit
# To check without results try:
# for i in $(seq 1 10); do time zsh -i -c exit; done
if [[ "${ZSH_PROFILE}" == 1 ]]; then
    zmodload zsh/zprof
else
    ZSH_PROFILE=0
fi


# First load shell agnostic functions
. ~/.shellrc

##################
#  Common Zsh    #
##################

if [ ! -f ~/.zshrc.zwc -o ~/.zshrc -nt ~/.zshrc.zwc ]; then
    zcompile ~/.zshrc
fi

#  Keys
#############

typeset -A key

key[Home]=${terminfo[khome]}

key[End]=${terminfo[kend]}
key[Insert]=${terminfo[kich1]}
key[Delete]=${terminfo[kdch1]}
key[Up]=${terminfo[kcuu1]}
key[Down]=${terminfo[kcud1]}
key[Left]=${terminfo[kcub1]}
key[Right]=${terminfo[kcuf1]}
key[PageUp]=${terminfo[kpp]}
key[PageDown]=${terminfo[knp]}

# setup key accordingly
[[ -n "${key[Home]}"     ]]  && bindkey  "${key[Home]}"     beginning-of-line
[[ -n "${key[End]}"      ]]  && bindkey  "${key[End]}"      end-of-line
[[ -n "${key[Insert]}"   ]]  && bindkey  "${key[Insert]}"   overwrite-mode
[[ -n "${key[Delete]}"   ]]  && bindkey  "${key[Delete]}"   delete-char
[[ -n "${key[Up]}"       ]]  && bindkey  "${key[Up]}"       up-line-or-history
[[ -n "${key[Down]}"     ]]  && bindkey  "${key[Down]}"     down-line-or-history
[[ -n "${key[Left]}"     ]]  && bindkey  "${key[Left]}"     backward-char
[[ -n "${key[Right]}"    ]]  && bindkey  "${key[Right]}"    forward-char
[[ -n "${key[PageUp]}"   ]]  && bindkey  "${key[PageUp]}"   beginning-of-buffer-or-history
[[ -n "${key[PageDown]}" ]]  && bindkey  "${key[PageDown]}" end-of-buffer-or-history

# Functions
############

# Reload complete functions
r() {
    source ~/.zshrc
    if [ -d ~/.zsh/comp ]; then
        local f
        f=(~/.zsh/comp/*(.))
        unfunction $f:t 2> /dev/null
        autoload -U $f:t
    fi
}


# Plugin Management
#####################

if [[ ! -d ~/.zinit ]]; then
    mkdir ~/.zinit
    git clone https://github.com/zdharma/zinit.git ~/.zinit/bin
fi

source ~/.zinit/bin/zinit.zsh

autoload -Uz _zinit
(( ${+_comps} )) && _comps[zinit]=_zinit

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
POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(dir newline nvm pyenv rbenv virtualenv vcs nix_shell)

# Bullet Train #

# zplug "caiogondim/bullet-train.zsh", use:bullet-train.zsh-theme, defer:3 # defer until other plugins like oh-my-zsh is loaded

# Spaceship #

# zplug denysdovhan/spaceship-prompt, use:spaceship.zsh, from:github, as:theme

# Platform
#############

if [ -f ~/.zshPlatform ]; then
  . ~/.zshPlatform
fi


# Specifics
#############

if [ -f ~/.zshSpecifics ]; then
  . ~/.zshSpecifics
fi

# HPC
#######
if [ -f ~/.zshlmod ]; then
  . ~/.zshlmod
fi

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
zinit ice as"command" from"gh-r" mv"fd* -> fd" pick"fd/fd"
zinit light sharkdp/fd

# sharkdp/bat
zinit ice as"command" from"gh-r" mv"bat* -> bat" pick"bat/bat"
zinit light sharkdp/bat

# ogham/exa, replacement for ls
zinit ice wait"2" lucid from"gh-r" as"program" mv"exa* -> exa"
zinit light ogham/exa

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


# End Profiler
if [[ "${ZSH_PROFILE}" == 1 ]]; then
zprof
fi

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:

# To customize prompt, run `p10k configure` or edit ~/.p10k.zsh.
[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh


source "/home/haozeke/.local/share/dephell/_dephell_zsh_autocomplete"

PATH="/home/haozeke/perl5/bin${PATH:+:${PATH}}"; export PATH;
PERL5LIB="/home/haozeke/perl5/lib/perl5${PERL5LIB:+:${PERL5LIB}}"; export PERL5LIB;
PERL_LOCAL_LIB_ROOT="/home/haozeke/perl5${PERL_LOCAL_LIB_ROOT:+:${PERL_LOCAL_LIB_ROOT}}"; export PERL_LOCAL_LIB_ROOT;
PERL_MB_OPT="--install_base \"/home/haozeke/perl5\""; export PERL_MB_OPT;
PERL_MM_OPT="INSTALL_BASE=/home/haozeke/perl5"; export PERL_MM_OPT;
