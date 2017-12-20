#!/bin/bash

# From https://github.com/sampowers/dotfiles/blob/100a4530485ddd36ff8787f9f657f7578983bfc8/bin/java_nonreparenting_wm_hack.sh
# Fix X window manager name properties to work around java bugs with
# non-reparenting window managers.  This is a different solution from
# the wmname utility provided by suckless, as it is NetWM compatible,
# while wmname sets the value of _NET_SUPPORTING_WM_CHECK to root win.

IRONIC_WM_NAME="LG3D"
NET_WIN=$(xprop -root _NET_SUPPORTING_WM_CHECK | awk -F "# " '{print $2}')

if [[ "$NET_WIN" == 0x* ]]; then
    # xprop cannot reliably set UTF8_STRING, so we replace as string.
    # fortunately, jdk is OK with this, but wm-spec says use UTF8_STRING.
    xprop -id "$NET_WIN" -remove _NET_WM_NAME
    xprop -id "$NET_WIN" -f _NET_WM_NAME 8s -set _NET_WM_NAME "$IRONIC_WM_NAME"
else
    # even if we're not net compatible, do java workaround
    xprop -root -remove _NET_WM_NAME
    xprop -root -f _NET_WM_NAME 8s -set _NET_WM_NAME "$IRONIC_WM_NAME"
fi
