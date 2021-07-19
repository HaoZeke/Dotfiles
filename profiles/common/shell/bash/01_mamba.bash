if [ "$USE_MAMBA" == "yes" || "$USE_MAMBA" == "true" ]; then
    # mamba initialize >>>
    # !! Contents within this block are managed by 'mamba init' !!
    export MAMBA_EXE="$HOME/.local/bin/micromamba";
    export MAMBA_ROOT_PREFIX="$HOME/.micromamba";
    __mamba_setup="$('$HOME/.local/bin/micromamba' shell hook --shell bash --prefix '$HOME/.micromamba' 2> /dev/null)"
    if [ $? -eq 0 ]; then
        eval "$__mamba_setup"
    else
    if [ -f "$HOME/.micromamba/etc/profile.d/mamba.sh" ]; then
        . "$HOME/.micromamba/etc/profile.d/mamba.sh"
    else
        export PATH="$HOME/.micromamba/bin:$PATH"
    fi
    fi
    unset __mamba_setup
    # <<< mamba initialize <<<
fi
