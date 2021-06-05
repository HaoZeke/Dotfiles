#!/usr/bin/env perl

# PDF settings
$pdf_mode=1;
$pdflatex="xelatex --shell-escape %O %S";
# enable deletion of *.bbl at "latexmk -c"
$bibtex_use = 2;

# Previewer described here: https://github.com/YukihiroMasuoka/dotfiles/blob/master/latexmk/.latexmkrc
if ($^O eq 'MSWin32') {
  if (-f 'C:/Program Files/SumatraPDF/SumatraPDF.exe') {
    $pdf_previewer = '"C:/Program Files/SumatraPDF/SumatraPDF.exe" -reuse-instance';
  }
  elsif (-f 'C:/Program Files (x86)/SumatraPDF/SumatraPDF.exe') {
    $pdf_previewer = '"C:/Program Files (x86)/SumatraPDF/SumatraPDF.exe" -reuse-instance';
  }
  elsif (-f "~/AppData/Local/SumatraPDF/SumatraPDF.exe") {
      $pdf_previewer = '"~/AppData/Local/SumatraPDF/SumatraPDF.exe" -reuse-instance';
  }
  elsif (-f "C:/SumatraPDF/SumatraPDF.exe") {
      $pdf_previewer = '"C:/SumatraPDF/SumatraPDF.exe" -reuse-instance';
  }
  else {
    $pdf_previewer = 'texworks';
  }
}
else {
    if (-f "/mnt/c/SumatraPDF/SumatraPDF.exe"){ # for wsl settings
	$pdf_previewer = '"/mnt/c/SumatraPDF/SumatraPDF.exe" -reuse-instance';
    }
    else {
	$pdf_previewer = 'zathura'; # macos and linux
    }
}

# More cleaning
$clean_ext = 'bbl fdb_latexmk fls nav pdfsync pyg pytxcode run.xml ' .
             'snm synctex.gz thm upa vrb _minted-%R pythontex-files-%R ' .
             '**/*-eps-converted-to.pdf';

# Generated files clean better
@generated_exts = qw(acn acr alg aux code ist fls glg glo gls idx ind lof lot out thm toc tpt);

# Glossary support
push @generated_exts, 'glo', 'gls', 'glg';
push @generated_exts, 'acn', 'acr', 'alg';
sub run_makeglossaries {
  if ( $silent ) {
    system("makeglossaries -q $_[0]");
  }
  else {
    system("makeglossaries $_[0]");
  };
}
$clean_ext .= ' %R.ist %R.xdy';
add_cus_dep('glo', 'gls', 0, 'run_makeglossaries');
add_cus_dep('acn', 'acr', 0, 'run_makeglossaries');

# Overwrite `unlink_or_move` to support clean directory.
use File::Path 'rmtree';
sub unlink_or_move {
    if ( $del_dir eq '' ) {
        foreach (@_) {
            if (-d $_) {
                rmtree $_;
            } else {
                unlink $_;
            }
        }
    }
    else {
        foreach (@_) {
            if (-e $_ && ! rename $_, "$del_dir/$_" ) {
                warn "$My_name:Cannot move '$_' to '$del_dir/$_'\n";
            }
        }
    }
}
# Partially derived from:
# https://github.com/iswunistuttgart/latex-boilerplate/blob/master
