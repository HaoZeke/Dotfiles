# Dots
## A dotfiles repository

These are various settings for essential utilities. [zsh,tmux,vim]

This is now managed with [dotgit](https://github.com/kobus-v-schoor/dotgit).

As of the 11th of April, 2019, the configuration for my `emacs` setup, with `doom-emacs` has been moved to [a repository of its own](https://github.com/HaoZeke/dotDoom).
### Usage
Several targets (groups) have been defined.

* archlinuxMono --> This is for machines with ArchLinux and a single monitor.
* archlinuxMulti --> This is for machines with ArchLinux and 2 monitors (VGA-0 and DVI-0)
* myarchbox --> This is customized for my desktop. Don't use this.
* noroot --> This is when you haven't got root. [**TODO**]



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


#### TO-DO

* Consolidate multi and single monitor setups (panel remaining)
* Add update scripts for the install helpers
* ~~Switch from tilda to tdrop+kitty (dropdowns)~~
* Use a more monitor aware panel
* Use a configuration management system like puppet, ansible or saltstack for full automation
* Add aura setups for Salt
* Finish creating targets including
  + Root target
