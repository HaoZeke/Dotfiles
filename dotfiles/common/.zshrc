# First load shell agnostic functions
. ~/.shellrc


# ##################
# #  Common Zsh    #
# ##################

# # # zmodload zsh/zprof

# # if [ ! -f ~/.zshrc.zwc -o ~/.zshrc -nt ~/.zshrc.zwc ]; then
# #     zcompile ~/.zshrc
# # fi

# # compinstall additions 
# ########################

# autoload -Uz compinit
# compinit
# setopt globdots

# # zsh-newuser-install additions
# #################################

# HISTFILE=~/.histfile
# HISTSIZE=100000000
# SAVEHIST=100000000
# setopt appendhistory autocd beep notify
# setopt HIST_IGNORE_DUPS
# bindkey -e


# #  Keys     
# #############

# typeset -A key

# key[Home]=${terminfo[khome]}

# key[End]=${terminfo[kend]}
# key[Insert]=${terminfo[kich1]}
# key[Delete]=${terminfo[kdch1]}
# key[Up]=${terminfo[kcuu1]}
# key[Down]=${terminfo[kcud1]}
# key[Left]=${terminfo[kcub1]}
# key[Right]=${terminfo[kcuf1]}
# key[PageUp]=${terminfo[kpp]}
# key[PageDown]=${terminfo[knp]}

# # setup key accordingly
# [[ -n "${key[Home]}"     ]]  && bindkey  "${key[Home]}"     beginning-of-line
# [[ -n "${key[End]}"      ]]  && bindkey  "${key[End]}"      end-of-line
# [[ -n "${key[Insert]}"   ]]  && bindkey  "${key[Insert]}"   overwrite-mode
# [[ -n "${key[Delete]}"   ]]  && bindkey  "${key[Delete]}"   delete-char
# [[ -n "${key[Up]}"       ]]  && bindkey  "${key[Up]}"       up-line-or-history
# [[ -n "${key[Down]}"     ]]  && bindkey  "${key[Down]}"     down-line-or-history
# [[ -n "${key[Left]}"     ]]  && bindkey  "${key[Left]}"     backward-char
# [[ -n "${key[Right]}"    ]]  && bindkey  "${key[Right]}"    forward-char
# [[ -n "${key[PageUp]}"   ]]  && bindkey  "${key[PageUp]}"   beginning-of-buffer-or-history
# [[ -n "${key[PageDown]}" ]]  && bindkey  "${key[PageDown]}" end-of-buffer-or-history

# # Finally, make sure the terminal is in application mode, when zle is
# # active. Only then are the values from $terminfo valid.
# if (( ${+terminfo[smkx]} )) && (( ${+terminfo[rmkx]} )); then
#     function zle-line-init () {
#         printf '%s' "${terminfo[smkx]}"
#     }
#     function zle-line-finish () {
#         printf '%s' "${terminfo[rmkx]}"
#     }
#     zle -N zle-line-init
#     zle -N zle-line-finish
# fi

# # Functions
# ############

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
POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(dir newline vcs)
POWERLEVEL9K_VCS_CLEAN_FOREGROUND='blue'
POWERLEVEL9K_VCS_CLEAN_BACKGROUND='black'
POWERLEVEL9K_VCS_UNTRACKED_FOREGROUND='yellow'
POWERLEVEL9K_VCS_UNTRACKED_BACKGROUND='black'
POWERLEVEL9K_VCS_MODIFIED_FOREGROUND='blue'
POWERLEVEL9K_VCS_MODIFIED_BACKGROUND='black'
zplug "bhilburn/powerlevel9k", use:powerlevel9k.zsh-theme

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

# Colors and Highlighting
zplug "zsh-users/zsh-syntax-highlighting"
# zplug "zlsun/solarized-man"

# OMZ
# zplug "plugins/common-aliases",             from:oh-my-zsh
# zplug "plugins/colored-man-pages",          from:oh-my-zsh
# zplug "plugins/command-not-found",          from:oh-my-zsh
# zplug "plugins/history",                    from:oh-my-zsh
zplug "plugins/history-substring-search",   from:oh-my-zsh
# zplug "plugins/github",                     from:oh-my-zsh
zplug "plugins/git",                        from:oh-my-zsh
# zplug "plugins/gulp",                       from:oh-my-zsh
# zplug "plugins/systemadmin",                from:oh-my-zsh
# zplug "plugins/sudo",                       from:oh-my-zsh
# zplug "plugins/systemd",                    from:oh-my-zsh
# zplug "plugins/rsync",                      from:oh-my-zsh
# zplug "plugins/tmux",                       from:oh-my-zsh
# zplug "plugins/yarn",                       from:oh-my-zsh

# Misc
# zplug "knu/z", use:"*.sh"

# Completions
# zplug "zsh-users/zsh-completions"
# zplug "ascii-soup/zsh-url-highlighter"
# zplug "djui/alias-tips"
# zplug "srijanshetty/zsh-pip-completion"
# zplug "b4b4r07/enhancd", use:init.sh

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

# # The stuff below is copied from https://github.com/Shougo/shougo-s-github/blob/master/.zshrc

# #####################################################################
# # completions
# #####################################################################

# # # Enable completions
# # if [ -d ~/.zsh/comp ]; then
# #     fpath=(~/.zsh/comp $fpath)
# #     autoload -U ~/.zsh/comp/*(:t)
# # fi

# # zstyle ':completion:*' group-name ''
# # zstyle ':completion:*:messages' format '%d'
# # zstyle ':completion:*:descriptions' format '%d'
# # zstyle ':completion:*:options' verbose yes
# # zstyle ':completion:*:values' verbose yes
# # zstyle ':completion:*:options' prefix-needed yes
# # # Use cache completion
# # # apt-get, dpkg (Debian), rpm (Redhat), urpmi (Mandrake), perl -M,
# # # bogofilter (zsh 4.2.1 >=), fink, mac_apps...
# # zstyle ':completion:*' use-cache true
# # zstyle ':completion:*:default' menu select=1
# # zstyle ':completion:*' matcher-list \
# #     '' \
# #     'm:{a-z}={A-Z}' \
# #     'l:|=* r:|[.,_-]=* r:|=* m:{a-z}={A-Z}'
# # # sudo completions
# # zstyle ':completion:*:sudo:*' command-path /usr/local/sbin /usr/local/bin \
# #     /usr/sbin /usr/bin /sbin /bin /usr/X11R6/bin
# # zstyle ':completion:*' menu select
# # zstyle ':completion:*' keep-prefix
# # zstyle ':completion:*' completer _oldlist _complete _match _ignored \
# #     _approximate _list _history

# # autoload -U compinit

# # if [ ! -f ~/.zcompdump -o ~/.zshrc -nt ~/.zcompdump ]; then
# #     compinit -d ~/.zcompdump
# # fi

# # # Original complete functions
# # compdef _tex platex

# # # cd search path
# # cdpath=($HOME)

# # zstyle ':completion:*:processes' command "ps -u $USER -o pid,stat,%cpu,%mem,cputime,command"


# # #####################################################################
# # # colors
# # #####################################################################

# # # Color settings for zsh complete candidates
# # alias ls='ls -F --show-control-chars --color=always'
# # alias la='ls -aF --show-control-chars --color=always'
# # alias ll='ls -lF --show-control-chars --color=always'
# # export LSCOLORS=ExFxCxdxBxegedabagacad
# # export LS_COLORS='di=01;34:ln=01;35:so=01;32:ex=01;31:bd=46;34:cd=43;34:su=41;30:sg=46;30:tw=42;30:ow=43;30'
# # zstyle ':completion:*' list-colors 'di=;34;1' 'ln=;35;1' 'so=;32;1' 'ex=31;1' 'bd=46;34' 'cd=43;34'

# # # Use prompt colors feature
# # autoload -U colors
# # colors

# # # Use vcs_info
# # autoload -Uz vcs_info
# # zstyle ':vcs_info:git:*' check-for-changes true
# # zstyle ':vcs_info:git:*' stagedstr "%F{yellow}!"
# # zstyle ':vcs_info:git:*' unstagedstr "%F{blue}+"
# # zstyle ':vcs_info:*' formats "%F{green}%c%u[%b]%f"
# # zstyle ':vcs_info:*' actionformats '[%b|%a]'

# # PROMPT='%{[33m%}[%35<..<%~]%{[m%}${vcs_info_msg_0_}
# # %{[$[31+$RANDOM % 7]m%}%U%B%#'"%b%{[m%}%u "

# # if [ -n "${REMOTEHOST}${SSH_CONNECTION}" ] ; then
# #     PROMPT="%{[37m%}${HOST%%.*} ${PROMPT}"
# # fi

# # if [ $UID = "0" ]; then
# #     PROMPT="%B%{[31m%}%/#%{^[[m%}%b "
# #     PROMPT2="%B%{[31m%}%_#%{^[[m%}%b "
# # fi

# # # Multi line prompt
# # PROMPT2="%_%% "
# # # Spell miss prompt
# # SPROMPT="correct> %R -> %r [n,y,a,e]? "


# # #####################################################################
# # # options
# # ######################################################################

# # setopt auto_resume
# # # Ignore <C-d> logout
# # setopt ignore_eof
# # # {a-c} -> a b c
# # setopt brace_ccl
# # # Enable spellcheck
# # setopt correct
# # # Enable "=command" feature
# # setopt equals
# # # Disable flow control
# # setopt no_flow_control
# # # Ignore dups
# # setopt hist_ignore_dups
# # # Reduce spaces
# # setopt hist_reduce_blanks
# # # Save time stamp
# # setopt extended_history
# # # Expand history
# # setopt hist_expand
# # # Better jobs
# # setopt long_list_jobs
# # # Enable completion in "--option=arg"
# # setopt magic_equal_subst
# # # Add "/" if completes directory
# # setopt mark_dirs
# # # Disable menu complete for vimshell
# # setopt no_menu_complete
# # setopt list_rows_first
# # # Expand globs when completion
# # setopt glob_complete
# # # Enable multi io redirection
# # setopt multios
# # # Can search subdirectory in $PATH
# # setopt path_dirs
# # # For multi byte
# # setopt print_eightbit
# # # Print exit value if return code is non-zero
# # setopt print_exit_value
# # setopt pushd_ignore_dups
# # setopt pushd_silent
# # # Short statements in for, repeat, select, if, function
# # setopt short_loops
# # # Ignore history (fc -l) command in history
# # setopt hist_no_store
# # setopt transient_rprompt
# # unsetopt promptcr
# # setopt hash_cmds
# # setopt numeric_glob_sort
# # # Enable comment string
# # setopt interactive_comments
# # # Improve rm *
# # setopt rm_star_wait
# # # Enable extended glob
# # setopt extended_glob
# # # Note: It is a lot of errors in script
# # # setopt no_unset
# # # Prompt substitution
# # setopt prompt_subst
# # setopt always_last_prompt
# # # List completion
# # setopt auto_list
# # setopt auto_param_slash
# # setopt auto_param_keys
# # # List like "ls -F"
# # setopt list_types
# # # Compact completion
# # setopt list_packed
# # setopt auto_cd
# # setopt auto_pushd
# # setopt pushd_minus
# # setopt pushd_ignore_dups
# # # Check original command in alias completion
# # setopt complete_aliases
# # unsetopt hist_verify
