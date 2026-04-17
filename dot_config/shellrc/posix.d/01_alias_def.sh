# Aliases
###########

if [ ! -f ~/blank/ ]; then
	mkdir -p ~/blank/
fi

alias foldel='time rsync -avv --delete /home/$USER/blank/ '

# Copy anything [ command | copy ]
alias copy='xclip -sel clip'

# Termbin (use to pipe output, eg. ls | tb)
alias tb="nc termbin.com 9999"

# Make directory
alias md='mkdir -p'

# Emacs: prefer the persistent daemon via emacsclient. First launch of
# the day starts the daemon (slow -- Doom init + native-comp); every
# subsequent window is instant.
#   e       GUI window attached to the daemon
#   ec      terminal (-nw) attached to the daemon
#   emacs   plain terminal Emacs (no daemon)
# -c  new frame, -a ''  auto-start daemon if not running, -n  don't
# wait for the frame to close before returning.
alias e='emacsclient -c -n -a ""'
alias ec='emacsclient -nw -a ""'
alias emacs='emacs -nw'

# Better ls
if which exa >/dev/null 2>&1; then
	alias ls=exa
fi

# Indian Time
alias indiaTime="TZ=Asia/Kolkata date +'Asia/Kolkata %a, %b %d, %Y %r'"

# gh CLI with Ruhi's credentials
alias gh-ruhi='GH_TOKEN=$(pass github/ruhi-pat) gh'

# safer deletions
alias rm=rmtrash
