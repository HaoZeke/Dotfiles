#!/usr/bin/env bash

# Get a filename
case "$#" in
0)
	echo "No arguments, so enter the filename"
	read -p 'RMD File: ' rmdfile
	;;
1)
	echo "OK, using the filename"
	rmdfile="$1"
	;;
*)
	echo "Illegal number of parameters"
	exit
	;;
esac

# Now using the filename
Rscript -e "library(rmarkdown); render('$rmdfile.Rmd',c('pdf_document','html_document'));"
if [[ -f "$rmdfile.html" ]]; then
	{
		echo "Conversion succeeded, rendering math"
		mv $rmdfile.html tmp.html
		hash mjpage 2>/dev/null || {
			echo >&2 "I require mjpage from mathjax-node-page but it's not installed. Not converting math"
			mv tmp.html $rmdfile.html
		}
		mjpage --output MML <tmp.html >$rmdfile.html
		rm -rf tmp.html
	}
fi

# Move things
finame=${rmdfile// /-}
mkdir -p "out/$finame"
mv "$finame".{html,pdf} "out/$finame"
