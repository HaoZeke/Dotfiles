# # Warnings
# ############
# if ! which git >/dev/null 2>&1; then
# 	echo "You do not have git.\n DO NOT DO ANYTHING WITHOUT GIT\n"
# 	echo "You might want to install brew with getBrew\n"
# 	echo "Once you have brew just run\nbrew install git"
# fi

# # Tmux
# ##########

# # PS1="$PS1"'$([ -n "$TMUX" ] && tmux setenv TMUXPWD_$(tmux display -p "#D" | tr -d %) "$PWD")'
# # alias tmux="tmux -2"

# if which tmux >/dev/null 2>&1; then
# 	getTpm
# else
# 	echo "You appear to be missing tmux"
# fi
