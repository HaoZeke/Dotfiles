#!/usr/bin/env zsh

if [[ ! -d ~/.zi ]]; then
    source <(curl -sL git.io/zi-loader); zzinit
fi

autoload -Uz _zi
(( ${+_comps} )) && _comps[zi]=_zi

exec zsh
