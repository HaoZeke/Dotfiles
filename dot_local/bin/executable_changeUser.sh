#!/usr/bin/env sh

set -e

usage() {
    echo "Usage: $0"
    echo "Changes the git configuration for the current repository."
    exit 1
}

if [ "$#" -ne 0 ]; then
    usage
fi

if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "Error: Not inside a Git repository"
    exit 1
fi

# Print current user details
echo "Current Git Configuration:"
echo "User Name: $(git config --get user.name)"
echo "User Email: $(git config --get user.email)"

# List SSH keys from ~/.ssh and ask user to choose one
echo "Available SSH keys in ~/.ssh:"
select SSH_KEY in ~/.ssh/*_rsa ~/.ssh/*_ed25519; do
    case "$SSH_KEY" in
        *_rsa|*_ed25519 )
            break;;
        * )
            echo "Invalid choice";;
    esac
done

# Ask user for new name and email
read -p "Enter new Git user name: " NEW_NAME
read -p "Enter new Git user email: " NEW_EMAIL

# Set new git configurations
git config core.SshCommand "ssh -i $SSH_KEY -F /dev/null"
git config user.name "$NEW_NAME"
git config user.email "$NEW_EMAIL"
git config commit.gpgsign false

echo "Git configuration updated for this repository."
