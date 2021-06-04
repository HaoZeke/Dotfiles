# PDF settings
$pdf_previewer = 'zathura';

# More cleaning
$clean_ext = 'bbl fdb_latexmk fls nav pdfsync pyg pytxcode run.xml ' .
             'snm synctex.gz thm upa vrb _minted-%R pythontex-files-%R ' .
             '**/*-eps-converted-to.pdf';
$clean_ext .= " acr acn alg glo gls glg"; # Glossary files
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
