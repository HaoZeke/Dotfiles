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
