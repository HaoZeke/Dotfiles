#!/usr/bin/env bash

# ag <httpss://github.com/ggreer/the_silver_searcher>
# usage: ag-replace.sh [search] [replace]
# caveats: will choke if either arguments contain a forward slash
# notes: will back up changed files to *.bak files

ag -l $1 | xargs perl -pi.bak -e "s/$1/$2/g"

# or if you prefer sed's regex syntax:

#ag -l $1 | xargs -0 sed -ri.bak -e "s/$1/$2/g"