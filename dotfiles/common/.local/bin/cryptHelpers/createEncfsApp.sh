#!/usr/bin/env bash

# Usage
# createEncfsApp.sh $appname

# Copyright (c) 2018 Rohit Goswami <rohit dot goswami at yahoo dot com>
 
# Permission is hereby granted, free of charge, to any person obtaining
# a copy of this software and associated documentation files (the
# "Software"), to deal in the Software without restriction, including
# without limitation the rights to use, copy, modify, merge, publish,
# distribute, sublicense, and/or sell copies of the Software, and to
# permit persons to whom the Software is furnished to do so, subject to
# the following conditions:
 
# The above copyright notice and this permission notice shall be
# included in all copies or substantial portions of the Software.
 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
# NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
# LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
# OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
# WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

# TODO Complete

# Get the right directory name
# bash specific
tempName=( $1 )
appName=$(echo "${tempName[@]^}")

cryptDir="$HOME/Encfs/.$appName"
appDir="$HOME/.decrypt/$appName"

echo $cryptDir
echo $appDir

if [ ! -d $appDir ]; then
    mkdir -p $appDir    
fi

if [ ! -d $cryptDir ]; then
    mkdir -p $cryptDir    
fi

# Create the stash

encfs $cryptDir $appDir
