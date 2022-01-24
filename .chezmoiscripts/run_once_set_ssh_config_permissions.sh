#!/usr/bin/env sh

# Details: https://superuser.com/questions/215504/permissions-on-private-key-in-ssh-folder
find $HOME/.ssh/ -type f -not -name "*.pub" -exec chmod 600 {} \;;
find $HOME/.ssh/ -type d -exec chmod 700 {} \;;
find $HOME/.ssh/ -type f -name "*.pub" -exec chmod 644 {} \;
