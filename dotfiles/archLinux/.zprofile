if [ -z "$DISPLAY" ] && [ -n "$XDG_VTNR" ] && [ "$XDG_VTNR" -eq 1 ]; then
	exec startx
fi

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:

export PATH="$HOME/.poetry/bin:$PATH"
