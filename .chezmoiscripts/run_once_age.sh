#!/usr/bin/env bash
# This is problematic, and necessary
# Will try, in order:
# - brew
# - source build (with go)
# - prebuilt binaries

if command -v age >/dev/null 2>&1; then
    echo "got age, good to go"
    # has age, continue
else
    if command -v brew >/dev/null 2>&1; then
        brew install age # Typically MacOS
    fi
    if command -v go >/dev/null 2>&1; then
        git clone https://filippo.io/age && cd age
        go build -o . filippo.io/age/cmd/...
        mkdir -p ~/.local/bin
        mv age age-keygen ~/.local/bin
        export PATH=$PATH:$HOME/.local/bin
    fi
fi
