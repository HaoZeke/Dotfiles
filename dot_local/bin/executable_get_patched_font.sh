#!/usr/bin/env bash

if [ ! -d $HOME/save/nerd-fonts ]; then

# With some inputs from https://github.com/pandoc-extras/pandoc-portable

  # Make the folder
  mkdir -p $HOME/save

  cd $HOME/save

  git clone https://github.com/ryanoasis/nerd-fonts.git

else
  cd $HOME/save/nerd-fonts
  chmod +x install.sh
  if [[ $1 == "all" ]]; then
    ./install.sh
  else
    ./install.sh $1
  fi

fi
