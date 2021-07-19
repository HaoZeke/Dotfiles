# Path Additions
##################

# Linuxbrew
export PATH=$HOME/.linuxbrew/bin:$PATH

# TeX (overloaded variables with bombadil)
export PATH="/usr/local/texlive/__[tex_year]__/bin/__[machine_type]__/:$PATH"

# Poetry
export PATH="$HOME/.poetry/bin:$PATH"

# Doom
export PATH=$PATH:$HOME/.emacs.d/bin

# Rust
export PATH=$PATH:$HOME/.cargo/bin

# For the bundled scripts
export PATH="$HOME/.local/bin:$PATH"
export PATH="$PATH:$HOME/.local/bin/cronDrivers"
export PATH="$PATH:$HOME/.local/bin/cryptHelpers"
export PATH="$PATH:$HOME/.local/bin/dockerHelpers"
export PATH="$PATH:$HOME/.local/bin/droidHelpers"
export PATH="$PATH:$HOME/.local/bin/fileHelpers"
export PATH="$PATH:$HOME/.local/bin/gitHelpers"
export PATH="$PATH:$HOME/.local/bin/installHelpers"
export PATH="$PATH:$HOME/.local/bin/shellHelpers"
export PATH="$PATH:$HOME/.local/bin/webHelpers"

# NVM is now handled by zsh-nvm

# Fixes python venvs
# curl https://pyenv.run | bash
export WORKON_HOME=~/.venvs
export PATH="$HOME/.pyenv/bin:$PATH"
if which pyenv >/dev/null 2>&1; then
    eval "$(pyenv init -)"
    eval "$(pyenv virtualenv-init -)"
else
	echo "You seem to be missing pyenv"
fi

# Go Paths
###########

export GOPATH=$HOME/.go
export GOBIN=$HOME/.go/bin
export GOCACHE=$HOME/.cache/go-build

export PATH=$PATH:$GOBIN

# Yarn Paths
#############
export PATH=$HOME/.yarn/bin:$PATH

# Perl Paths
#############
PATH="$HOME/perl5/bin${PATH:+:${PATH}}"; export PATH;
PERL5LIB="$HOME/perl5/lib/perl5${PERL5LIB:+:${PERL5LIB}}"; export PERL5LIB;
PERL_LOCAL_LIB_ROOT="$HOME/perl5${PERL_LOCAL_LIB_ROOT:+:${PERL_LOCAL_LIB_ROOT}}"; export PERL_LOCAL_LIB_ROOT;
PERL_MB_OPT="--install_base \"$HOME/perl5\""; export PERL_MB_OPT;
PERL_MM_OPT="INSTALL_BASE=$HOME/perl5"; export PERL_MM_OPT;

# Snap for rofi
export XDG_DATA_DIRS=/usr/share/:/usr/local/share/:/var/lib/snapd/desktop

# Conditionals
###############

# Sets language server locations on remote machines
if [ "$IS_REMOTE" == "yes" || "$IS_REMOTE" == "true" ]; then
    export PATH=$HOME/.local/lsp/bin:$PATH
fi
