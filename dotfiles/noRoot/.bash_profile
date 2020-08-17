# Don't execute for TRAMP
if [ "$TERM" != "dumb" ]; then
	# Now let's see if we have zsh
	if [ -f ~/.nix-profile/bin/zsh ]; then
		# OK then, is it running already?
		if [[ $(ps -p $$ -ocomm=) != "zsh" ]]; then
			exec ~/.nix-profile/bin/zsh -l
		fi
	fi
fi

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
