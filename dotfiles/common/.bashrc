# No Root
##########

if [ -f ~/.norootrc ]; then
	. ~/.norootrc
fi

# First load shell agnostic functions
. ~/.shellrc

#################
#  Common Bash  #
#################

# Works only in bash
# Version comparision
function version_gt() { test "$(printf '%s\n' "$@" | sort -V | head -n 1)" != "$1"; }

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

