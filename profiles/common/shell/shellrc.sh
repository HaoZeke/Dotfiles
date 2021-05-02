###########################
# Shell Agnostic Commands #
###########################


# Fix paths
############
export SXHKD_SHELL=/usr/bin/zsh

# Platform
#############

if [ -f ~/.shellPlatform ]; then
	. ~/.shellPlatform
fi

# Specifics
#############

if [ -f ~/.shellSpecifics ]; then
	. ~/.shellSpecifics
fi

# Wayland
#############

if [ -f ~/.waylandEnv ]; then
	. ~/.waylandEnv
fi

# XKB
######

if [ -f ~/.xkbEnv ]; then
	. ~/.xkbEnv
fi

# Nix
######

if [ -f ~/.nixEnv ]; then
	. ~/.nixEnv
fi

# SDKMan
#THIS MUST BE AT THE END OF THE FILE FOR SDKMAN TO WORK!!!
export SDKMAN_DIR="$HOME/.sdkman"
[[ -s "$HOME/.sdkman/bin/sdkman-init.sh" ]] && source "$HOME/.sdkman/bin/sdkman-init.sh"

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
