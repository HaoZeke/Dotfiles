# Completion

# fixme - the load process here seems a bit bizarre
zmodload -i zsh/complist

WORDCHARS=''

# autoload
autoload -Uz run-help
autoload -Uz add-zsh-hook
autoload -Uz colors && colors
autoload -Uz compinit && compinit -u
autoload -Uz is-at-least
zstyle ':completion:*' menu select

# Options
unsetopt menu_complete # do not autoselect the first completion entry
unsetopt flowcontrol
unsetopt nomatch # fixes rake completions https://thoughtbot.com/blog/how-to-use-arguments-in-a-rake-task

setopt auto_cd
setopt multios
setopt prompt_subst        # for prompt setups
setopt interactivecomments # recognize comments
setopt auto_menu           # show completion menu on successive tab press
setopt complete_in_word
setopt always_to_end
setopt HIST_IGNORE_SPACE # Don't save things with a space
setopt no_global_rcs
setopt append_history
setopt extended_history
setopt hist_expire_dups_first
setopt hist_ignore_dups # ignore duplication command history list
setopt hist_ignore_space
setopt hist_verify
setopt inc_append_history
setopt share_history # share command history data

# should this be in keybindings?
bindkey -M menuselect '^o' accept-and-infer-next-history
zstyle ':completion:*:*:*:*:*' menu select

# case insensitive (all), partial-word and substring completion
if [[ "$CASE_SENSITIVE" = true ]]; then
	zstyle ':completion:*' matcher-list 'r:|=*' 'l:|=* r:|=*'
else
	if [[ "$HYPHEN_INSENSITIVE" = true ]]; then
		zstyle ':completion:*' matcher-list 'm:{a-zA-Z-_}={A-Za-z_-}' 'r:|=*' 'l:|=* r:|=*'
	else
		zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}' 'r:|=*' 'l:|=* r:|=*'
	fi
fi
unset CASE_SENSITIVE HYPHEN_INSENSITIVE

zstyle ':completion:*' list-colors ''
zstyle ':completion:*:*:kill:*:processes' list-colors '=(#b) #([0-9]#) ([0-9a-z-]#)*=01;34=0=01'

if [[ "$OSTYPE" = solaris* ]]; then
	zstyle ':completion:*:*:*:*:processes' command "ps -u $USER -o pid,user,comm"
else
	zstyle ':completion:*:*:*:*:processes' command "ps -u $USER -o pid,user,comm -w -w"
fi

# disable named-directories autocompletion
zstyle ':completion:*:cd:*' tag-order local-directories directory-stack path-directories

# Use caching so that commands like apt and dpkg complete are useable
zstyle ':completion::complete:*' use-cache 1
zstyle ':completion::complete:*' cache-path $ZSH_CACHE_DIR

# Don't complete uninteresting users
zstyle ':completion:*:*:*:users' ignored-patterns \
	adm amanda apache at avahi avahi-autoipd beaglidx bin cacti canna \
	clamav daemon dbus distcache dnsmasq dovecot fax ftp games gdm \
	gkrellmd gopher hacluster haldaemon halt hsqldb ident junkbust kdm \
	ldap lp mail mailman mailnull man messagebus mldonkey mysql nagios \
	named netdump news nfsnobody nobody nscd ntp nut nx obsrun openvpn \
	operator pcap polkitd postfix postgres privoxy pulse pvm quagga radvd \
	rpc rpcuser rpm rtkit scard shutdown squid sshd statd svn sync tftp \
	usbmux uucp vcsa wwwrun xfs '_*'

# ... unless we really want to.
zstyle '*' single-ignored show

# History
# Command history configuration
if [ -z "$HISTFILE" ]; then
	HISTFILE=$HOME/.zsh_history
fi
# History size in memory
export HISTSIZE=10000
# The number of histsize
export SAVEHIST=1000000
# The size of asking history
export LISTMAX=50
# Do not add in root
if [[ $UID == 0 ]]; then
	unset HISTFILE
	export SAVEHIST=0
fi

# available $INTERACTIVE_FILTER
export INTERACTIVE_FILTER="fzf:peco:percol:gof:pick"

bindkey '\ew' kill-region # [Esc-w] - Kill from the cursor to the mark

# From https://github.com/bunnyfly/dotfiles/blob/master/zshrc
# bindkey -v # Use vim key bindings

if [ -f ~/.zshColemak ]; then
	. ~/.zshColemak
fi
