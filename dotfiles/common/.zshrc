###########################
#  Common Completions     #
###########################

# zmodload zsh/zprof

if [ ! -f ~/.zshrc.zwc -o ~/.zshrc -nt ~/.zshrc.zwc ]; then
    zcompile ~/.zshrc
fi

# compinstall additions 
########################

autoload -Uz compinit
compinit


# zsh-newuser-install additions
#################################

HISTFILE=~/.histfile
HISTSIZE=100000000
SAVEHIST=100000000
setopt appendhistory autocd beep notify
setopt HIST_IGNORE_DUPS
bindkey -e


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

# Finally, make sure the terminal is in application mode, when zle is
# active. Only then are the values from $terminfo valid.
if (( ${+terminfo[smkx]} )) && (( ${+terminfo[rmkx]} )); then
    function zle-line-init () {
        printf '%s' "${terminfo[smkx]}"
    }
    function zle-line-finish () {
        printf '%s' "${terminfo[rmkx]}"
    }
    zle -N zle-line-init
    zle -N zle-line-finish
fi

# Aliases
###########

if [ ! -f ~/blank/ ]; then
  mkdir -p ~/blank/
fi

alias foldel='time rsync -avv --delete /home/$USER/blank/ '

# Termbin (use to pipe output, eg. ls | tb)
alias tb="nc termbin.com 9999"

# Sudo
alias '_'= 'sudo'

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

# Spit out a deduplicated PATH
pathdup() {
if [ -n "$PATH" ]; then
old_PATH=$PATH:; PATH=
 while [ -n "$old_PATH" ]; do
x=${old_PATH%%:*}       # the first remaining entry
case $PATH: in
*:"$x":*) ;;         # already there
*) PATH=$PATH:$x;;    # not there yet
 esac
old_PATH=${old_PATH#*:}
done
PATH=${PATH#:}
unset old_PATH x
fi  
}

# Calibre Upgrade
caliup() {
  sudo -v && wget --no-check-certificate -nv -O- https://download.calibre-ebook.com/linux-installer.py | sudo python -c "import sys; main=lambda:sys.stderr.write('Download failed\n'); exec(sys.stdin.read()); main()"
}

# Transfer.sh
transfer() { if [ $# -eq 0 ]; then echo -e "No arguments specified. Usage:\necho transfer /tmp/test.md\ncat /tmp/test.md | transfer test.md"; return 1; fi
tmpfile=$( mktemp -t transferXXX ); if tty -s; then basefile=$(basename "$1" | sed -e 's/[^a-zA-Z0-9._-]/-/g'); curl --progress-bar --upload-file "$1" "https://transfer.sh/$basefile" >> $tmpfile; else curl --progress-bar --upload-file "-" "https://transfer.sh/$1" >> $tmpfile ; fi; cat $tmpfile; rm -f $tmpfile; }

# Image resizer
smartresize() {
   mogrify -path $3 -filter Triangle -define filter:support=2 -thumbnail $2 -unsharp 0.25x0.08+8.3+0.045 -dither None -posterize 136 -quality 82 -define jpeg:fancy-upsampling=off -define png:compression-filter=5 -define png:compression-level=9 -define png:compression-strategy=1 -define png:exclude-chunk=all -interlace none -colorspace sRGB $1
}

# Libgen Downloader
libgen () {
local args=$1
local genio='http://libgen.io/book/index.php?md5='
local genrus='http://gen.lib.rus.ec/book/index.php?md5='

if [ -z "${args##*$genio*}" ] ;then
  local cHashgenio=${@#$genio}
  echo "[DEBUG] ARGS=${args}"
  echo "[INFO] The hash will be extracted."
  local MD5=${cHashgenio}
elif [ -z "${args##*$genrus*}" ]; then
  local cHashgenrus=${@#$genrus}
  echo "[DEBUG] ARGS=${args}"
  echo "[INFO] The hash will be extracted."
  local MD5=${cHashgenrus}
else
  echo "[DEBUG] ARGS=${args}"
  local MD5=${args}
fi
echo "[INFO] MD5=${MD5}"

# The whole Key thing stops working
# local URL='http://libgen.io/get.php?md5='
# local KEY=$(curl -sL "${URL}${MD5}" | grep -oE "key=[^']*" | cut -d'=' -f2)
# echo "[INFO] KEY=${KEY}"

# local FURL="${URL}${MD5}&key=${KEY}"

# Non-Key
local altURL='http://libgen.io/ads.php?md5='
local FURL="${altURL}${MD5}"

if [[ -x aria2c ]]; then
  aria2c -j2 "$FURL"
elif [[ -x wget ]]; then
  wget -v -c "${FURL}" -O "${MD5}.libgen"
  wget --trust-server-names -v -c "${FURL}"
else
  curl -L -J -O "${FURL}" --progress-bar
fi

# Check Hash
# md5sum $file???? == $MD5
}


# Sources
############

# GRE Word List
if [[ -d $HOME/Github/Linux/gre-cli-words ]]; then
  $HOME/Github/Linux/gre-cli-words/random_gre.sh $HOME/Github/Linux/gre-cli-words/custom_gre_word_list
else
  echo "You don't have gre-cli-words setup"
fi

# Nix
if [[ -d /nix ]]; then
  . $HOME/.nix-profile/etc/profile.d/nix.sh
else
  echo "You don't have nix installed"
fi


# Common Exports
###################

export XDG_CONFIG_HOME="$HOME/.config"


# Tmux
##########

PS1="$PS1"'$([ -n "$TMUX" ] && tmux setenv TMUXPWD_$(tmux display -p "#D" | tr -d %) "$PWD")'
export TERM=xterm-256color
alias tmux="tmux -2"


# Shell Theme
###############

source ~/.shell_prompt.sh


# Plugin Management
#####################

if [[ ! -d ~/.zplug ]]; then
    git clone https://github.com/zplug/zplug ~/.zplug
    source ~/.zplug/init.zsh && zplug update --self
fi

source ~/.zplug/init.zsh


# Specifics 
#############

if [ -f ~/.zshSpecifics ]; then
  . ~/.zshSpecifics
fi


# Common zPlug stuff
######################

# OMZ
zplug "plugins/common-aliases",             from:oh-my-zsh
zplug "plugins/colored-man-pages",          from:oh-my-zsh
zplug "plugins/command-not-found",          from:oh-my-zsh
zplug "plugins/history",                    from:oh-my-zsh
zplug "plugins/history-substring-search",   from:oh-my-zsh
zplug "plugins/github",                     from:oh-my-zsh
zplug "plugins/git",                        from:oh-my-zsh
zplug "plugins/gulp",                       from:oh-my-zsh
zplug "plugins/systemadmin",                from:oh-my-zsh
zplug "plugins/sudo",                       from:oh-my-zsh
zplug "plugins/systemd",                    from:oh-my-zsh
zplug "plugins/rsync",                      from:oh-my-zsh
zplug "plugins/tmux",                       from:oh-my-zsh
zplug "plugins/yarn",                       from:oh-my-zsh

# Misc
zplug "knu/z", use:"*.sh"

# Completions
zplug "zsh-users/zsh-completions"
zplug "ascii-soup/zsh-url-highlighter"
zplug "djui/alias-tips"
zplug "srijanshetty/zsh-pip-completion"
zplug "b4b4r07/enhancd", use:init.sh

# Colors and Highlighting
zplug "zsh-users/zsh-syntax-highlighting"
zplug "zlsun/solarized-man"

# Install plugins if there are plugins that have not been installed
if ! zplug check --verbose; then
    printf "Install? [y/N]: "
    if read -q; then
        echo; zplug install
    fi
fi

# Then, source plugins and add commands to $PATH
zplug load

# The stuff below is copied from https://github.com/Shougo/shougo-s-github/blob/master/.zshrc

#####################################################################
# completions
#####################################################################

# Enable completions
if [ -d ~/.zsh/comp ]; then
    fpath=(~/.zsh/comp $fpath)
    autoload -U ~/.zsh/comp/*(:t)
fi

zstyle ':completion:*' group-name ''
zstyle ':completion:*:messages' format '%d'
zstyle ':completion:*:descriptions' format '%d'
zstyle ':completion:*:options' verbose yes
zstyle ':completion:*:values' verbose yes
zstyle ':completion:*:options' prefix-needed yes
# Use cache completion
# apt-get, dpkg (Debian), rpm (Redhat), urpmi (Mandrake), perl -M,
# bogofilter (zsh 4.2.1 >=), fink, mac_apps...
zstyle ':completion:*' use-cache true
zstyle ':completion:*:default' menu select=1
zstyle ':completion:*' matcher-list \
    '' \
    'm:{a-z}={A-Z}' \
    'l:|=* r:|[.,_-]=* r:|=* m:{a-z}={A-Z}'
# sudo completions
zstyle ':completion:*:sudo:*' command-path /usr/local/sbin /usr/local/bin \
    /usr/sbin /usr/bin /sbin /bin /usr/X11R6/bin
zstyle ':completion:*' menu select
zstyle ':completion:*' keep-prefix
zstyle ':completion:*' completer _oldlist _complete _match _ignored \
    _approximate _list _history

autoload -U compinit

if [ ! -f ~/.zcompdump -o ~/.zshrc -nt ~/.zcompdump ]; then
    compinit -d ~/.zcompdump
fi

# Original complete functions
compdef _tex platex

# cd search path
cdpath=($HOME)

zstyle ':completion:*:processes' command "ps -u $USER -o pid,stat,%cpu,%mem,cputime,command"


#####################################################################
# colors
#####################################################################

# Color settings for zsh complete candidates
alias ls='ls -F --show-control-chars --color=always'
alias la='ls -aF --show-control-chars --color=always'
alias ll='ls -lF --show-control-chars --color=always'
export LSCOLORS=ExFxCxdxBxegedabagacad
export LS_COLORS='di=01;34:ln=01;35:so=01;32:ex=01;31:bd=46;34:cd=43;34:su=41;30:sg=46;30:tw=42;30:ow=43;30'
zstyle ':completion:*' list-colors 'di=;34;1' 'ln=;35;1' 'so=;32;1' 'ex=31;1' 'bd=46;34' 'cd=43;34'

# Use prompt colors feature
autoload -U colors
colors

# Use vcs_info
autoload -Uz vcs_info
zstyle ':vcs_info:git:*' check-for-changes true
zstyle ':vcs_info:git:*' stagedstr "%F{yellow}!"
zstyle ':vcs_info:git:*' unstagedstr "%F{red}+"
zstyle ':vcs_info:*' formats "%F{green}%c%u[%b]%f"
zstyle ':vcs_info:*' actionformats '[%b|%a]'

PROMPT='%{[33m%}[%35<..<%~]%{[m%}${vcs_info_msg_0_}
%{[$[31+$RANDOM % 7]m%}%U%B%#'"%b%{[m%}%u "

if [ -n "${REMOTEHOST}${SSH_CONNECTION}" ] ; then
    PROMPT="%{[37m%}${HOST%%.*} ${PROMPT}"
fi

if [ $UID = "0" ]; then
    PROMPT="%B%{[31m%}%/#%{^[[m%}%b "
    PROMPT2="%B%{[31m%}%_#%{^[[m%}%b "
fi

# Multi line prompt
PROMPT2="%_%% "
# Spell miss prompt
SPROMPT="correct> %R -> %r [n,y,a,e]? "


#####################################################################
# options
######################################################################

setopt auto_resume
# Ignore <C-d> logout
setopt ignore_eof
# {a-c} -> a b c
setopt brace_ccl
# Enable spellcheck
setopt correct
# Enable "=command" feature
setopt equals
# Disable flow control
setopt no_flow_control
# Ignore dups
setopt hist_ignore_dups
# Reduce spaces
setopt hist_reduce_blanks
# Save time stamp
setopt extended_history
# Expand history
setopt hist_expand
# Better jobs
setopt long_list_jobs
# Enable completion in "--option=arg"
setopt magic_equal_subst
# Add "/" if completes directory
setopt mark_dirs
# Disable menu complete for vimshell
setopt no_menu_complete
setopt list_rows_first
# Expand globs when completion
setopt glob_complete
# Enable multi io redirection
setopt multios
# Can search subdirectory in $PATH
setopt path_dirs
# For multi byte
setopt print_eightbit
# Print exit value if return code is non-zero
setopt print_exit_value
setopt pushd_ignore_dups
setopt pushd_silent
# Short statements in for, repeat, select, if, function
setopt short_loops
# Ignore history (fc -l) command in history
setopt hist_no_store
setopt transient_rprompt
unsetopt promptcr
setopt hash_cmds
setopt numeric_glob_sort
# Enable comment string
setopt interactive_comments
# Improve rm *
setopt rm_star_wait
# Enable extended glob
setopt extended_glob
# Note: It is a lot of errors in script
# setopt no_unset
# Prompt substitution
setopt prompt_subst
setopt always_last_prompt
# List completion
setopt auto_list
setopt auto_param_slash
setopt auto_param_keys
# List like "ls -F"
setopt list_types
# Compact completion
setopt list_packed
setopt auto_cd
setopt auto_pushd
setopt pushd_minus
setopt pushd_ignore_dups
# Check original command in alias completion
setopt complete_aliases
unsetopt hist_verify
