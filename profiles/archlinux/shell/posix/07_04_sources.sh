# Sources
##########

# For tilix to not complain about login shells.
test -f /etc/profile.d/vte.sh && source /etc/profile.d/vte.sh

# added by travis gem
[ -f $HOME/.travis/travis.sh ] && source $HOME/.travis/travis.sh

# For Amber
test -f /$HOME/Git/Github/amber16/amber.sh && source /$HOME/Git/Github/amber16/amber.sh
