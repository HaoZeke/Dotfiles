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

# Ripgrep
alias rg='rg --ignore-file $HOME/.config/ripgrep/ignore'

# Make directory
alias md='mkdir -p'

# Emacs for the terminal
alias emacs='emacs -nw'

# Better ls
if which exa >/dev/null 2>&1; then
	alias ls=exa
fi

# Indian Time
alias indiaTime="TZ=Asia/Kolkata date +'Asia/Kolkata %a, %b %d, %Y %r'"
