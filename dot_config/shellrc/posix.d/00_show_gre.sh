# GRE Word List
if [[ $- = *i* ]]; then
    if [[ -d $HOME/.local/greWords/gre-cli-words ]]; then
        $HOME/.local/greWords/gre-cli-words/random_gre.sh $HOME/.local/greWords/gre-cli-words/custom_gre_word_list
        echo "\n"
    else
        echo "You don't have gre-cli-words setup, try running getGREwords"
    fi
fi

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
