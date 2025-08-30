# Commands to run in interactive sessions can go here
if status is-interactive
    function fish_greeting
        if test -d "$HOME/.local/greWords/gre-cli-words"
            "$HOME/.local/greWords/gre-cli-words/random_gre.sh" "$HOME/.local/greWords/gre-cli-words/custom_gre_word_list"
        else
            echo "You don't have gre-cli-words setup, try running getGREwords"
        end
    end
end
