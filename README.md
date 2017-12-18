# Dots
## A dotfiles repository

These are various settings for essential utilities. [zsh,tmux,vim]

This is now managed with [dotgit](https://github.com/kobus-v-schoor/dotgit).

### Usage
Several targets (groups) have been defined.

* archlinux --> This is for machines with ArchLinux.
* noroot --> This is when you haven't got root. [TO-BE-IMPLEMENTED]


### Installation

As far as dotgit and the configs go simply grab it like so:

```bash
git clone https://github.com/Cube777/dotgit
mkdir -p ~/.bin
cp -r dotgit/bin/dotgit* ~/.bin
cat dotgit/bin/bash_completion >> ~/.bash_completion
rm -rf dotgit
# echo 'export PATH="$PATH:$HOME/.bin"' >> ~/.bashrc
echo 'export PATH="$PATH:$HOME/.bin"' >> ~/.zshrc
```

Then get use these:

```bash
# Get this repo
git clone https://github.com/HaoZeke/Dots
cd Dots
# Use the targets listed above
dotgit restore $target
```

### Full Setup
There are a bunch of bootstrap scripts depending on your OS and needs.
**TODO**

The Xfce4 Panel configs are very much biased to the bspwm settings.

### Required Packages
It is necessary for the following packagages to be installed.
* The Silver Searcher [ag] --> {CtrlP}
* Python2 --> {For YouCompleteMe}
* Npm --> {Markdown, JSlint}
* Virtualenv --> {For vim compatible python2}
* PNF Powerline Fonts --> {For the symbols}
