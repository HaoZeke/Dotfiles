# Plugin Management
#####################

ZINIT_HOME="${XDG_DATA_HOME:-${HOME}/.local/share}/zinit/zinit.git"
if [[ ! -d "$ZINIT_HOME/.git" ]]; then
    if command -v git >/dev/null 2>&1; then
        mkdir -p "${ZINIT_HOME:h}"
        git clone --depth=1 https://github.com/zdharma-continuum/zinit.git "$ZINIT_HOME"
    else
        return 0
    fi
fi

[[ -r "$ZINIT_HOME/zinit.zsh" ]] || return 0
source "$ZINIT_HOME/zinit.zsh"

autoload -Uz _zinit
(( ${+_comps} )) && _comps[zinit]=_zinit
