# Don't execute for TRAMP
if [ "$TERM" != "dumb" ]; then
	# Now let's see if we have zsh
	if which zsh >/dev/null 2>&1; then
		# OK then, is it running already?
		if [[ $(ps -p $$ -ocomm=) != "zsh" ]]; then
			export SHELL=$(which zsh)
			exec "$SHELL" -l
		fi
	fi
fi

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
