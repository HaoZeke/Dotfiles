# First load shell agnostic functions
. ~/.shellrc


##################
#  Common Zsh    #
##################

# zmodload zsh/zprof

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

if [[ ! -d ~/.zplug ]]; then
    git clone https://github.com/zplug/zplug ~/.zplug
    source ~/.zplug/init.zsh && zplug update --self
fi

source ~/.zplug/init.zsh


# Shell Theme
###############

#source ~/.shell_prompt.sh
POWERLEVEL9K_MODE='nerdfont-complete'
POWERLEVEL9K_VCS_CLEAN_FOREGROUND='blue'
POWERLEVEL9K_VCS_CLEAN_BACKGROUND='black'
POWERLEVEL9K_VCS_UNTRACKED_FOREGROUND='yellow'
POWERLEVEL9K_VCS_UNTRACKED_BACKGROUND='black'
POWERLEVEL9K_VCS_MODIFIED_FOREGROUND='blue'
POWERLEVEL9K_VCS_MODIFIED_BACKGROUND='black'

# Right
POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=(status root_indicator background_jobs time)

# Left
POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(dir newline rbenv virtualenv vcs)

zplug "bhilburn/powerlevel9k", use:powerlevel9k.zsh-theme, defer:3

# zplug "caiogondim/bullet-train.zsh", use:bullet-train.zsh-theme, defer:3 # defer until other plugins like oh-my-zsh is loaded

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


# Common zPlug stuff
######################

# OMZ Plugins
zplug "plugins/history",                    from:oh-my-zsh
zplug "plugins/history-substring-search",   from:oh-my-zsh
zplug "plugins/github",                     from:oh-my-zsh
zplug "plugins/git",                        from:oh-my-zsh
zplug "plugins/gulp",                       from:oh-my-zsh
zplug "plugins/systemadmin",                from:oh-my-zsh
zplug "plugins/sudo",                       from:oh-my-zsh
zplug "plugins/systemd",                    from:oh-my-zsh
zplug "plugins/rsync",                      from:oh-my-zsh

# No OMZ
zplug "j-arnaiz/common-aliases"
zplug "hcgraf/zsh-sudo"

# Colors and Highlighting
zplug "zdharma/fast-syntax-highlighting", defer:2
zplug "zlsun/solarized-man"

# Misc
zplug "b4b4r07/enhancd", use:init.sh
zplug "changyuheng/zsh-interactive-cd"
zplug "mollifier/cd-gitroot"
zplug "bobsoppe/zsh-ssh-agent", use:ssh-agent.zsh, from:github
zplug "zdharma/history-search-multi-word"

# Completions
zplug "zsh-users/zsh-completions"
zplug "ascii-soup/zsh-url-highlighter"
zplug "molovo/tipz" 
zplug "srijanshetty/zsh-pip-completion"

# Install plugins if there are plugins that have not been installed
if ! zplug check --verbose; then
    printf "Install? [y/N]: "
    if read -q; then
        echo; zplug install
    fi
fi

# Then, source plugins and add commands to $PATH
zplug load

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
