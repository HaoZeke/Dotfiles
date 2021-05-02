if [[ ! -o interactive ]]; then
	# non-interactive, return
	return
fi

export shellHome=$HOME/.config/shellrc

# Load all files from the shell.d directory
if [ -d $shellHome/shell.d ]; then
	for file in $shellHome/shell.d/*.sh; do
		source $file
	done
fi

# Load all files from the zsh.d directory
if [ -d $shellHome/zsh.d ]; then
	for file in $shellHome/zsh.d/*.zsh; do
		source $file
	done
fi

# Common Zsh #
if [ ! -f ~/.zshrc.zwc -o ~/.zshrc -nt ~/.zshrc.zwc ]; then
    zcompile ~/.zshrc
fi

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

# HPC
#######
if [ -f ~/.zshlmod ]; then
  . ~/.zshlmod
fi

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
