# Functions
#############

# Orphaned Packages.
orphans() {
	if [[ ! -n $(pacaur -Qdt) ]]; then
		echo "No orphans to remove."
	else
		sudo pacman -Rns $(pacman -Qdtq)
	fi
}

droidBuild() {
	echo "Setting up environment variables\n"
	export USE_HOST_LEX=yes # Prevent builds breaking..
	# Sets the right path for linkers. cm-12.1 only.
	#export LD_LIBRARY_PATH=/run/media/$USER/Storage/AndBuild/cm-12.1/prebuilts/gcc/linux-x86/arm/arm-linux-androideabi-4.8/lib/
	echo "Getting the right Python\n"
	source /run/media/$USER/Storage/AndBuild/venv/bin/activate
	echo "Getting the right tools for Gello \n"
	export PATH=$HOME/Git/Github/Linux/depot_tools:$PATH
	echo "Enjoy.\n"
}
