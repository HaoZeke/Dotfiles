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
        if [[ ! $(uname)=="Darwin" ]]; then
		    . $HOME/.nix-profile/etc/profile.d/nix.sh
        fi
	else
		echo "You don't have nix installed"
	fi
fi

# For ssh-add
eval $(ssh-agent) >/dev/null 2>&1

# Language Management
######################
if [[ ! -d ~/.asdf ]]; then
    mkdir ~/.asdf
    git clone https://github.com/asdf-vm/asdf.git ~/.asdf
fi

export ASDF_CONFIG_FILE="~/.config/asdf/asdfrc"
