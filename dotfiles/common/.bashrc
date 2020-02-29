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
export SDKMAN_DIR="/home/haozeke/.sdkman"
[[ -s "/home/haozeke/.sdkman/bin/sdkman-init.sh" ]] && source "/home/haozeke/.sdkman/bin/sdkman-init.sh"
