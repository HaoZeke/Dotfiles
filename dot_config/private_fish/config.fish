# Commands to run in interactive sessions can go here
if status is-interactive
    # Opt-in: the word list comes from the run_once_get_gre_words chezmoi
    # script, which only runs its body when SETUP_GRE_WORDS=true. Absent list,
    # absent greeting.
    function fish_greeting
        if test -d "$HOME/.local/greWords/gre-cli-words"
            "$HOME/.local/greWords/gre-cli-words/random_gre.sh" "$HOME/.local/greWords/gre-cli-words/custom_gre_word_list"
        end
    end
end
