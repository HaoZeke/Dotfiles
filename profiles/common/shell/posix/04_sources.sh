# Sources
############

# GRE Word List
if [[ $- = *i* ]]; then
	if [[ -d $HOME/.local/greWords/gre-cli-words ]]; then
		$HOME/.local/greWords/gre-cli-words/random_gre.sh $HOME/.local/greWords/gre-cli-words/custom_gre_word_list
		echo "\n"
	else
		echo "You don't have gre-cli-words setup, try running getGREwords"
	fi
fi

# Nix
if [[ $- = *i* ]]; then
	if [[ -f /etc/profile.d/nix.sh ]]; then
		. /etc/profile.d/nix.sh
	elif [[ -d /nix ]]; then
		. $HOME/.nix-profile/etc/profile.d/nix.sh
	else
		echo "You don't have nix installed"
	fi
fi

# For ssh-add
eval $(ssh-agent) >/dev/null 2>&1
