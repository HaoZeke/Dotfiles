#!/usr/bin/env bash

mkdir $HOME/.local/greWords
cd $HOME/.local/greWords
git clone https://github.com/mcaceresb/gre-cli-words
cd gre-cli-words
if command -v julia >/dev/null 2>&1; then
    julia -e "using Pkg; Pkg.add(\"DataFrames\")"
    chmod +x ./generate_gre_words.jl
    ./generate_gre_words.jl
fi
./random_gre.sh $HOME/.local/greWords/gre-cli-words/custom_gre_word_list
