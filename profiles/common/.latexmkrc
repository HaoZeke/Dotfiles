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

# Partially derived from:
# https://github.com/iswunistuttgart/latex-boilerplate/blob/master
