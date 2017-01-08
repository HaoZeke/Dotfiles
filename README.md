Dotfiles
================================
These are various settings for essential Linux utilities. [zsh,tmux,vim]


###Installation###
The Xfce4 Panel configs are very much biased to the bspwm settings.

####Required Packages####
It is necessary for the following packagages to be installed.
* The Silver Searcher [ag] --> {CtrlP}
* Python2 --> {For YouCompleteMe}
* Npm --> {Markdown, JSlint}
* Virtualenv --> {For vim compatible python2}
* PNF Powerline Fonts --> {For the symbols}

---------

**Ready the files.**

```bash
# Create a Github directory. [This is useful in general too.]
$ mkdir -p ~/Github
# Get the repo.
$ git clone git://github.com/HaoZeke/Dotfiles.git ~/Github/Dotfiles
```
**Create Symlinks for configs.**

```bash
# Symlinks.
$ ln -sfd ~/Github/Dotfiles/bspwm ~/.config/bspwm
$ ln -sfd ~/Github/Dotfiles/sxhkd ~/.config/sxhkd
$ ln -sf ~/Github/Dotfiles/Misc/compton.conf ~/.config/compton.conf
$ ln -sfd ~/Github/Dotfiles/xfce4/panel ~/.config/xfce4/panel
$ ln -sfd ~/Github/Dotfiles/xfce4/xfconf ~/.config/xfce4/xfconf
$ ln -sfd ~/Github/Dotfiles/Roxterm/roxterm.sourceforge.net ~/.config/roxterm.sourceforge.net
$ ln -sf ~/Github/Dotfiles/vim/vimrc ~/.vimrc
$ ln -sf ~/Github/Dotfiles/Zsh/zshrc ~/.zshrc
$ ln -sf ~/Github/Dotfiles/Zsh/shell_prompt.sh ~/.shell_prompt.sh
$ ln -sf ~/Github/Dotfiles/tmux/tmux.conf ~/.tmux.conf
$ ln -sf ~/Github/Dotfiles/tmux/tmuxlinesource ~/.tmuxline
$ ln -sfd ~/Github/Dotfiles/vim ~/.vim
$ ln -sfd ~/Github/Dotfiles/tilda ~/.config/tilda
$ ln -sfd ~/Github/Dotfiles/Sublime\ Text\ 3/Settings ~/.config/sublime-text-3/Packages/User/
$  ln -sfd ~/Github/Dotfiles/Thunar ~/.config/Thunar
```
**Now get the submodules for zgen.**

```bash
# Get those Submodules!
$ cd ~/Github/Dotfiles
$ git submodule init
$ git submodule update
```
####Sublime Text 3####

For [Sublime Text 3](http://www.sublimetext.com/3 "Install Link"), Additionally, first get [Package Control](https://packagecontrol.io/installation "Cornerstone").


####Windows####
The sublime text 3 settings may be applied to windows with suitable modifications as shown...

```bash
$ cd "$env:appdata\Sublime Text 3\Packages\"
$ rmdir -recurse User
$ cmd /c mklink /D User $env:userprofile\Github\Dotfiles\Sublime Text 3\Settings
```
