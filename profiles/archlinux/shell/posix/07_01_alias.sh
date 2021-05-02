###############################
# ArchLinux shell extensions  #
###############################

# Aliases
##########

alias ratemi='sudo reflector --verbose  -l 20 -p http --sort rate --save /etc/pacman.d/mirrorlist' # --country 'India'
alias pacunlock="sudo rm /var/lib/pacman/db.lck"                                                   # Delete the lock file /var/lib/pacman/db.lck
alias paclock="sudo touch /var/lib/pacman/db.lck"                                                  # Create the lock file /var/lib/pacman/db.lck

alias mediaserve='rygel --config=/home/$USER/.config/rygel.conf'
alias pizza.py='~/Git/Github/pizza-9Oct15/src/pizza.py'
alias FlashTool='sudo FlashTool'
alias cantata='cantata -style=gtk2'
alias cpv="rsync -poghb --backup-dir=/tmp/rsync -e /dev/null --progress --"

# Lammps
alias lmp_mpi='$HOME/Git/Github/Simulations/lammps/src/lmp_mpi'

# Trizen stuff
alias tris="trizen -Ss"
alias trince='trizen -S --noedit --noconfirm'

# Alias for External Hard Disk
alias Storage='/run/media/$USER/Storage'

# IE Alias.
# To get this working install winetricks and run
# WINEARCH=win32 WINEPREFIX=~/.wine-ie6 winetricks ie6
alias ie6=' WINEARCH=win32 WINEPREFIX=~/.wine-ie6 wine ~/.wine-ie6/drive_c/Program\ Files/Internet\ Explorer/IEXPLORE.EXE'

# Check later  $$

# Encfs Safe Files
alias z3op='mkdir /home/$USER/Encfs/Secrets ;encfs -i 5 /run/media/$USER/Storage/.Secrets /home/$USER/Encfs/Secrets'
alias z3cl='sudo umount /home/$USER/Encfs/Secrets ; rm -rf /home/$USER/Encfs/Secrets'

# Docker Aliases
alias airsonicDocker='docker run -v /run/media/$USER/Storage/Music:/airsonic/music -v /run/media/$USER/Storage/Music/airsonicData:/airsonic/data -p 4050:4040 -d airsonic/airsonic'
alias crdroidDocker='docker run -it -h crdroid -v /run/media/$USER/Storage/AndBuild/:/home/build/Android -v /home/$USER/.ccache/:/home/build/.ccache -v /home/$USER/.cache/:/home/build/.cache crdroid'
alias plexDocker='docker run plex'

# Convert this!!
alias calibreDocker='calibre-server --port 8264 --userdb /run/media/$USER/Storage/Library/calServUsers.sqlite --enable-auth --daemonize --log=/$HOME/.calServLog'
