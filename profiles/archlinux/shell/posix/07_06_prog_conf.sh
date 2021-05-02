# MATLAB Helper
# wmname LG3D  depreciated in favor of script; for breaking xfce4-panel windowmanager.
# Replaced with a shell script hack to prevent or try to suppress crap on the command line.
# export _JAVA_OPTIONS='-Dawt.useSystemAAFontSettings=on -Dswing.aatext=true -Dswing.defaultlaf=com.sun.java.swing.plaf.gtk.GTKLookAndFeel'
export _SILENT_JAVA_OPTIONS='-Dawt.useSystemAAFontSettings=on -Dswing.aatext=true -Dswing.defaultlaf=com.sun.java.swing.plaf.gtk.GTKLookAndFeel'
unset _JAVA_OPTIONS
alias java='java "$_SILENT_JAVA_OPTIONS"'
export JAVA_FONTS=/usr/share/fonts/TTF
