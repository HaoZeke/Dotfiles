#!/usr/bin/env sh

# Helpers
chck() { command -v $1 > /dev/null && echo $? }

# Brew
command -v brew > /dev/null || { \
    echo "Homebrew not found, installing"  &&  \
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" \
}

# Basics
# Always grab these, just in case
brew install git gh \
    termux openssh openssl \
    mosh ssh-copy-id \
    aria2c wget tree fzf \
    wakatime-cli neofetch \
    gfortran gcc borg yq

# Encryption + TeX
set -- "brew" "age" "gpg-tui" \
    "tectonic" "biber"
for item in "$@"; do
    if [ $(chck "$item") ]; then
        echo "Found $item";
    else
        echo "Installing $item" && \
        brew install $item;
    fi;
done

# Silver searcher
command -v ag > /dev/null || { \
    brew install the_silver_searcher \
}

# Zathura
# https://github.com/zegervdv/homebrew-zathura
command -v zathura > /dev/null || { \
    brew tap zegervdv/zathura && \
    brew install zathura-pdf-mupdf && \
    mkdir -p $(brew --prefix zathura)/lib/zathura  && \
    ln -s $(brew --prefix zathura-pdf-mupdf)/libpdf-mupdf.dylib $(brew --prefix zathura)/lib/zathura/libpdf-mupdf.dylib \
}

# Common includes
# Rust has a unified installer, for MacOS and Linux
# TODO: Handle Windows
{{ include ".local/bin/init_rust.sh" }}

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
