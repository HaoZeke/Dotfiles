# Commands
###########

if which ccache >/dev/null 2>&1; then
	ccache -M 50G >/dev/null
	export USE_CCACHE=1
else
	echo "You seem to be missing ccache"
fi
