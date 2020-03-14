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

# Tmux helper
if [[ -n "$PS1" ]] && [[ -z "$TMUX" ]] && [[ -n "$SSH_CONNECTION" ]]; then
	tmux attach-session -t ssh_tmux || tmux new-session -s ssh_tmux
fi

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
