#!/usr/bin/env bash

set -euo pipefail

MACHINE_TYPE=$(uname -m)
readonly MACHINE_TYPE
OS_TYPE=$(uname -s | tr '[:upper:]' '[:lower:]')
readonly OS_TYPE

readonly TEXLIVE_URL="http://mirror.ctan.org/systems/texlive/tlnet/install-tl-unx.tar.gz"

# declarations
TEXLIVE_DIR=""
TEXLIVE_BIN=""
TEXLIVE_PROFILE=""

install_texlive() {
  wget "$TEXLIVE_URL"
  tar -xzf install-tl-unx.tar.gz
  local install_dir=$(ls -d install-tl-20*)
  local install_date=${install_dir##*-}
    if command -v toml &> /dev/null; then
      temp_file=$(mktemp)
      if toml set ~/.config/chezmoi/chezmoi.toml data.tex_date "$install_date" > "$temp_file"; then
        mv "$temp_file" ~/.config/chezmoi/chezmoi.toml
      else
        rm "$temp_file"
        echo "Failed to update chezmoi.toml"
        exit 1
      fi
    fi
  TEXLIVE_DIR="$HOME/.local/share/texlive-$install_date"
  mkdir -p "$TEXLIVE_DIR"
  TEXLIVE_PROFILE="$TEXLIVE_DIR/texlive.profile"
  TEXLIVE_BIN="$TEXLIVE_DIR/bin/$MACHINE_TYPE-$OS_TYPE"
  create_profile
  cd "$install_dir"
  ./install-tl --profile="$TEXLIVE_PROFILE"
  cd ..
  # cleanup
  rm -rf "$install_dir"
  rm -rf install-tl-unx.tar.gz
}

create_profile() {
  cat > "$TEXLIVE_PROFILE" << EOF
selected_scheme scheme-basic
TEXDIR $TEXLIVE_DIR
TEXMFCONFIG ~/.texlive/texmf-config
TEXMFHOME ~/texmf
TEXMFLOCAL $TEXLIVE_DIR/texmf-local
TEXMFSYSCONFIG $TEXLIVE_DIR/texmf-config
TEXMFSYSVAR $TEXLIVE_DIR/texmf-var
TEXMFVAR ~/.texlive/texmf-var
option_doc 0
option_src 0
EOF
}

main() {
  if ! command -v texlua > /dev/null; then

    install_texlive
    rm "$TEXLIVE_PROFILE"

    export PATH="$TEXLIVE_BIN:$PATH"

    tlmgr option -- autobackup 0
    tlmgr update --self --all --no-auto-install

    tlmgr install luatex \
      biber         \
      beamer        \
      xetex         \
      latexmk       \
      etoolbox      \
      minted        \
      texliveonfly

    local texlive_year
    texlive_year=$(tlmgr --version | grep 'TeX Live' | awk '{print $5}')
    echo "TeX Live Year: $texlive_year"

    echo ""
    echo "Add $TEXLIVE_DIR/texmf-dist/doc/man to MANPATH."
    echo "Add $TEXLIVE_DIR/texmf-dist/doc/info to INFOPATH."
    echo "Most importantly, add $TEXLIVE_BIN to your PATH for current and future sessions."
    echo "Logfile: $TEXLIVE_DIR/install-tl.log"

  fi

}

main "$@"
