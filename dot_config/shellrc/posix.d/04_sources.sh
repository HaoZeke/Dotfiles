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
