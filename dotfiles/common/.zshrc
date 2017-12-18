###########################
#  Common Completions     #
###########################

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

# Calibre Upgrade
alias caliup= "sudo -v && wget --no-check-certificate -nv -O- https://download.calibre-ebook.com/linux-installer.py | sudo python -c "import sys; main=lambda:sys.stderr.write('Download failed\n'); exec(sys.stdin.read()); main()""

# Functions
############

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

# Transfer.sh
transfer() { if [ $# -eq 0 ]; then echo -e "No arguments specified. Usage:\necho transfer /tmp/test.md\ncat /tmp/test.md | transfer test.md"; return 1; fi
tmpfile=$( mktemp -t transferXXX ); if tty -s; then basefile=$(basename "$1" | sed -e 's/[^a-zA-Z0-9._-]/-/g'); curl --progress-bar --upload-file "$1" "https://transfer.sh/$basefile" >> $tmpfile; else curl --progress-bar --upload-file "-" "https://transfer.sh/$1" >> $tmpfile ; fi; cat $tmpfile; rm -f $tmpfile; }

smartresize() {
   mogrify -path $3 -filter Triangle -define filter:support=2 -thumbnail $2 -unsharp 0.25x0.08+8.3+0.045 -dither None -posterize 136 -quality 82 -define jpeg:fancy-upsampling=off -define png:compression-filter=5 -define png:compression-level=9 -define png:compression-strategy=1 -define png:exclude-chunk=all -interlace none -colorspace sRGB $1
}

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
if [[ ! -d $HOME/Github/Linux/gre-cli-words ]]; then
  $HOME/Github/Linux/gre-cli-words/random_gre.sh $HOME/Github/Linux/gre-cli-words/custom_gre_word_list
  echo "You haven't configured gre-cli-words"
fi

# Nix
if [[ ! -d /nix ]]; then
  . $HOME/.nix-profile/etc/profile.d/nix.sh
else
  echo "You don't have nix installed"
fi

# Common Exports
###################

export XDG_CONFIG_HOME="$HOME/.config"


# Plugin Management
#####################

if [[ ! -d $HOME/.zplug ]]; then
  curl -sL --proto-redir -all,https https://raw.githubusercontent.com/zplug/installer/master/installer.zsh| zsh
fi


# Common zPlug stuff
######################

source ~/.zplug/init.zsh

# OMZ
zplug "plugins/common-aliases",             from:oh-my-zsh
zplug "plugins/colored-man-pages",          from:oh-my-zsh
zplug "plugins/command-not-found",          from:oh-my-zsh
zplug "plugins/history",                    from:oh-my-zsh
zplug "plugins/github",                     from:oh-my-zsh
zplug "plugins/git",                        from:oh-my-zsh
zplug "plugins/gulp",                       from:oh-my-zsh
zplug "plugins/systemadmin",                from:oh-my-zsh
zplug "plugins/sudo",                       from:oh-my-zsh
zplug "plugins/systemd",                    from:oh-my-zsh
zplug "plugins/rsync",                      from:oh-my-zsh
zplug "plugins/tmux",                       from:oh-my-zsh
zplug "plugins/yarn",                       from:oh-my-zsh

# Colors and Highlighting
zplug "zsh-users/zsh-syntax-highlighting"
zplug "zlsun/solarized-man"

# Completions
zplug "zsh-users/zsh-completions"
zgen "ascii-soup/zsh-url-highlighter"
zgen "djui/alias-tips"
zgen "srijanshetty/zsh-pip-completion"

# Install plugins if there are plugins that have not been installed
if ! zplug check --verbose; then
    printf "Install? [y/N]: "
    if read -q; then
        echo; zplug install
    fi
fi

# Then, source plugins and add commands to $PATH
zplug load --verbose

# Tmux
##########

PS1="$PS1"'$([ -n "$TMUX" ] && tmux setenv TMUXPWD_$(tmux display -p "#D" | tr -d %) "$PWD")'
export TERM=xterm-256color
alias tmux="tmux -2"

# Shell Theme
###############

source ~/.shell_prompt.sh

# Specifics 
#############

if [ -f ~/.zshSpecifics ]; then
  . ~/.zshSpecifics
fi