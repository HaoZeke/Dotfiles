if [[ $- != *i* ]]; then
	# shell is non-interactive. Do nothing and return
	return
fi

if [[ $TERM = dumb ]]; then
	return # fixes TRAMP
fi

export shellHome=$HOME/.config/shellrc

# Load all files from the shell.d directory
if [ -d $shellHome/shell.d ]; then
	for file in $shellHome/shell.d/*.sh; do
		source $file
	done
fi

# Load all files from the bashrc.d directory
if [ -d $shellHome/bash.d ]; then
	for file in $shellHome/bash.d/*.bash; do
		source $file
	done
fi
