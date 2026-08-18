# Add a locally installed TeX Live to PATH, if one is there.
#
# Let fish expand the glob rather than shelling out to `ls`: an unmatched
# wildcard in a command substitution aborts the whole snippet with an error
# that `2>/dev/null` cannot reach, because it is fish's expansion failing and
# not the command's. `set` is one of the builtins where an unmatched wildcard
# expands to nothing instead, which is exactly the wanted behaviour on a host
# with no local TeX Live.
set -l texlive_dirs $HOME/.local/share/texlive-*/bin/x86_64-linux

if set -q texlive_dirs[1]; and test -d $texlive_dirs[1]
    fish_add_path $texlive_dirs[1]
end
