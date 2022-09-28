# Sources
############

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
