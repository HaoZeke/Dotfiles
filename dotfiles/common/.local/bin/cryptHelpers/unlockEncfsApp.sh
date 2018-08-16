#!/usr/bin/bash

# Usage
# unlockEncfsApp.sh $appname

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

#
# Program Implementation
#

# Determine the appropriate ASKPASS program

if which zenity >/dev/null 2>&1; then
    askPass='zenity --password --title="Unlock Thunderbird"'
elif which git >/dev/null 2>&1; then
    askPass='/usr/lib/git-core/git-gui--askpass "Unlock Thunderbird"'
else echo "ERROR: No valid (zenity or git) askpass program available\n"
     fi

# Create the directories
# Get the right directory name
# bash specific
tempName=( $1 )
appName=$(echo "${tempName[@]^}")

cryptDir="$HOME/Encfs/.$appName"
appDir="$HOME/.decrypt/$appName"

#
# TODO work on the logic when the folder is mounted
#

if [ ! -d $appDir ]; then
    mkdir -p $appDir
elif [[ $(findmnt -M "$appDir") ]]; then
    echo "Mounted"
    # Added later
    killall -9 $1
    encfs -u $appDir
    exit 1
else
    echo "Not mounted"
fi

if [ ! -d $cryptDir ]; then
    mkdir -p $cryptDir
fi

# Mount the stash

# TODO handle cases where the stash is created for the first time

encfs --extpass="$askPass" $cryptDir $appDir

# Run the program is the stash was mounted

RESULT=$?
if [ $RESULT -eq 0 ]; then
  echo success
  $1
else
  echo failed
  rm -rf $appDir
fi

