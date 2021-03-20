#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p "python38.withPackages(ps: [ ps.numpy ps.sh ])"

from pathlib import Path
import sh

# For generic IMAP maildirs

ISYNC_LABELS = ["rgoswami.iitk", "rog32"]

for isync in ISYNC_LABELS:
    sh.mbsync("-V",isync,_bg=True)


# Gmaileer
GMAIL_IDENTIFIERS = ["gmail", "univ", "ieee"]

path = Path(r"/run/media/haozeke/Storage/.mail/")

for dirs in path.iterdir():
    if dirs.is_dir():
        for gmi in GMAIL_IDENTIFIERS:
            if gmi in dirs.name:
                print(f"Syncing {dirs.name}")
                sh.gmi("sync", _cwd=dirs, _fg=True)
