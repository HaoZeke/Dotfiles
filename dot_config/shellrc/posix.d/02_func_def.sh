# Functions
############

# From https://github.com/xvoland/Extract/
# Now a script.

# Spit out a deduplicated PATH
pathdup() {
	if [ -n "$PATH" ]; then
		old_PATH=$PATH:
		PATH=
		while [ -n "$old_PATH" ]; do
			x=${old_PATH%%:*} # the first remaining entry
			case $PATH: in
				*:"$x":*) ;;        # already there
				*) PATH=$PATH:$x ;; # not there yet
			esac
			old_PATH=${old_PATH#*:}
		done
		PATH=${PATH#:}
		unset old_PATH x
	fi
}

# Calibre Upgrade
caliup() {
	sudo -v && wget --no-check-certificate -nv -O- https://download.calibre-ebook.com/linux-installer.py | sudo python -c "import sys; main=lambda:sys.stderr.write('Download failed\n'); exec(sys.stdin.read()); main()"
}

# Transfer.sh is dead, switched hosts
transfer() {
	# check arguments
	if [ $# -eq 0 ]; then
		echo "No arguments specified." >&2
		echo "Usage:" >&2
		echo "  transfer <file|directory>" >&2
		echo "  ... | transfer <file_name>" >&2
		return 1
	fi

	# upload stdin or file
	if tty -s; then
		file="$1"
		if [ ! -e "$file" ]; then
			echo "$file: No such file or directory" >&2
			return 1
		fi

		file_name=$(basename "$file" | sed -e 's/[^a-zA-Z0-9._-]/-/g')

		# upload file or directory
		if [ -d "$file" ]; then
			# transfer directory
			file_name="$file_name.zip"
			(cd "$file" && zip -r -q - .) | curl --progress-bar --upload-file "-" "https://filepush.co/upload/$file_name" | tee /dev/null
		else
			# transfer file
			cat "$file" | curl --progress-bar --upload-file "-" "https://filepush.co/upload/$file_name" | tee /dev/null
		fi
	else
		# transfer pipe
		file_name=$1
		curl --progress-bar --upload-file "-" "https://filepush.co/upload/$file_name" | tee /dev/null
	fi
}

# Image resizer
smartresize() {
	mogrify -path $3 -filter Triangle -define filter:support=2 -thumbnail $2 -unsharp 0.25x0.08+8.3+0.045 -dither None -posterize 136 -quality 82 -define jpeg:fancy-upsampling=off -define png:compression-filter=5 -define png:compression-level=9 -define png:compression-strategy=1 -define png:exclude-chunk=all -interlace none -colorspace sRGB $1
}

# Libgen Downloader
libgen() {
	local args=$1
	local genio='http://libgen.io/book/index.php?md5='
	local genrus='http://gen.lib.rus.ec/book/index.php?md5='

	if [ -z "${args##*$genio*}" ]; then
		local cHashgenio=${@#$genio}
		echo "[DEBUG] ARGS=${args}"
		echo "[INFO] The hash will be extracted."
		local MD5=${cHashgenio}
	elif [ -z "${args##*$genrus*}" ]; then
		local cHashgenrus=${@#$genrus}
		echo "[DEBUG] ARGS=${args}"
		echo "[INFO] The hash will be extracted."
		local MD5=${cHashgenrus}
	else
		echo "[DEBUG] ARGS=${args}"
		local MD5=${args}
	fi
	echo "[INFO] MD5=${MD5}"

	# The whole Key thing stops working
	# local URL='http://libgen.io/get.php?md5='
	# local KEY=$(curl -sL "${URL}${MD5}" | grep -oE "key=[^']*" | cut -d'=' -f2)
	# echo "[INFO] KEY=${KEY}"

	# local FURL="${URL}${MD5}&key=${KEY}"

	# Non-Key
	local altURL='http://libgen.io/ads.php?md5='
	local FURL="${altURL}${MD5}"

	if which aria2c >/dev/null 2>&1; then
		aria2c -j2 "$FURL"
	elif which wget >/dev/null 2>&1; then
		wget -v -c "${FURL}" -O "${MD5}.libgen"
		wget --trust-server-names -v -c "${FURL}"
	else
		curl -L -J -O "${FURL}" --progress-bar
	fi

	# Check Hash
	# md5sum $file???? == $MD5
}

getCloud() {
	fileName=$(curl -sI "${1}" | grep -o -E 'filename=.*$' | sed -e 's/filename=//')
	echo $fileName
	if [[ ! -a $fileName ]]; then
		if which aria2c >/dev/null 2>&1; then
			aria2c -j2 "$1"
		elif which wget >/dev/null 2>&1; then
			wget -v -c "${1}"
			wget --trust-server-names -v -c "${1}"
		else
			curl -L -J -O "${1}" --progress-bar
		fi
	else
		echo "This file already exists."
	fi
}

# Fuzzy string search from https://github.com/NicolaiRuckel/dotfiles/blob/master/zshrc
fif() {
	if [ ! "$#" -gt 0 ]; then
		echo "Need a string to search for!"
		return 1
	fi
	rg --files-with-matches --no-messages "$1" | fzf --preview "highlight -O ansi -l {} 2> /dev/null | rg --colors 'match:bg:yellow' --ignore-case --pretty --context 10 '$1' || rg --ignore-case --pretty --context 10 '$1' {}"
}

# Modified fast-p (https://github.com/bellecp/fast-p)
pf() {
	open=zathura
	ag -U -g ".pdf$" |
		fast-p |
		fzf --read0 --reverse -e -d $'\t' \
			--preview-window down:80% --preview '
            v=$(echo {q} | tr " " "|");
            echo -e {1}"\n"{2} | grep -E "^|$v" -i --color=always;
        ' |
		cut -z -f 1 -d $'\t' | tr -d '\n' | xargs -r --null $open >/dev/null 2>/dev/null
}

# Cleanup old mosh sessions
killmosh() {
	who | grep -v 'via mosh' | grep -oP '(?<=mosh \[)(\d+)(?=\])' | xargs kill
}
