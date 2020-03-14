# First load shell agnostic functions
. ~/.shellrc

#################
#  Common Bash  #
#################

# Platform
#############

if [ -f ~/.bashPlatform ]; then
	. ~/.bashPlatform
fi

# Specifics
#############

if [ -f ~/.bashSpecifics ]; then
	. ~/.bashSpecifics
fi

#THIS MUST BE AT THE END OF THE FILE FOR SDKMAN TO WORK!!!
export SDKMAN_DIR="$HOME/.sdkman"
[[ -s "$HOME/.sdkman/bin/sdkman-init.sh" ]] && source "$HOME/.sdkman/bin/sdkman-init.sh"
