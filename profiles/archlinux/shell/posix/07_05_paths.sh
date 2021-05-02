# PATH Changes
################

# Pre path [overrides] #
export PATH="$HOME/Git/Github/Dotfiles/Scripts:$HOME/.node_modules/bin:$PATH"

# Post Path [additions] #

# Add snaps to the path
export PATH="$PATH:/var/lib/snapd/snap/bin"

# Add the tlmgr TeX Installation
export PATH="$PATH:/usr/local/texlive/2019/bin/x86_64-linux"

# Build Paths
###############

# Add /usr/local/lib to libraries PATH
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib
export C_INCLUDE_PATH=$C_INCLUDE_PATH:/usr/local/include
export CPLUS_INCLUDE_PATH=$CPLUS_INCLUDE_PATH:/usr/local/include
export CPATH=$CPATH:/usr/local/include

# Gaussian
export PATH=$PATH:$HOME/.local/g16

# Add the path for android-sdk-platform-tools
export PATH=$PATH:/opt/android-sdk/platform-tools/
