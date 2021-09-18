# Exports
###########

# Horrible hack for themes to work
# export XDG_CURRENT_DESKTOP=KDE

export MPD_HOST='192.168.31.4'

export WINEPREFIX=$HOME/.wine/
export WINEARCH=win32

export QT_QPA_PLATFORMTHEME=qt5ct
export QT_AUTO_SCREEN_SCALE_FACTOR=0

export LD=ld.gold

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export LANGUAGE=en_US.UTF-8

# Prevent ROM builds breaking..
export USE_HOST_LEX=yes

# Otherwise bspwm kinda kills java GUI's
export _JAVA_AWT_WM_NONREPARENTING=1

# Fixes pipenv
export WORKON_HOME=~/.venvs

# Use pacman's keyring for gnupg (only temporary)
#export GNUPGHOME=/etc/pacman.d/gnupg

# Stop breaking Wine
export WINEARCH=win32

# For Docker pushes
export DOCKER_ID_USER=$USER

# Makes fonts darker and thicker
export INFINALITY_FT_BRIGHTNESS="-10"

# Not too sharp, not too smooth
export INFINALITY_FT_FILTER_PARAMS="16 20 28 20 16"
