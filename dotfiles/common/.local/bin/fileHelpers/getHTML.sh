#!/usr/bin/env bash
read -p 'RMD File: ' rmdfile
Rscript -e "library(rmarkdown); render('$rmdfile.Rmd',c('html_document'));"
mv $rmdfile.html tmp.html
mjpage --output MML <tmp.html >$rmdfile.html
rm -rf tmp.html
