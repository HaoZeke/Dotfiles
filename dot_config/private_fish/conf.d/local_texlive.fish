#!/usr/bin/env fish

set -l base_path "$HOME/.local/share/"
set -l texlive_dir (ls -d -1 "$base_path"/texlive-*/bin/x86_64-linux/ 2>/dev/null)[1]

if test -d "$texlive_dir"
    if not contains "$texlive_dir" $PATH
        fish_add_path "$texlive_dir"
    end
end
