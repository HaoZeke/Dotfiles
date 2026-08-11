#!/usr/bin/env bash
# rgam5terra: install the corrected disk alarm, release the snapshot-pinned nix
# store, and move /nix onto its own subvolume.
#
# /nix lives inside @, so every snapper timeline snapshot holds a reference to
# the store. `nix-collect-garbage` unlinks the paths from the live tree and the
# snapshots keep the extents, so the collection frees nothing. Snapper keeps two
# monthly and ten yearly snapshots, which is long enough for a store the size of
# a build root to sit pinned indefinitely. Step 2 releases what is already
# collected; step 3 stops it recurring.
#
# Run as root on rgam5terra. Expects /var/tmp/disk-space-buildbot.conf staged.

set -euo pipefail

UUID=8dc6c228-c906-4402-9629-1527d85ef043
MNTOPTS="rw,noatime,compress=zstd:3,ssd,space_cache=v2"
STAGED=/var/tmp/disk-space-buildbot.conf

[[ $EUID -eq 0 ]] || { echo "must run as root" >&2; exit 1; }
[[ -f $STAGED ]] || { echo "missing $STAGED" >&2; exit 1; }

echo "=== before ==="
df -h /

echo
echo "=== step 1: disk alarm ==="
install -m 0644 "$STAGED" /etc/netdata/health.d/disk-space-buildbot.conf
systemctl reload netdata
echo "installed and reloaded"

echo
echo "=== step 2: delete root timeline snapshots ==="
mapfile -t snaps < <(snapper -c root list \
    | awk -F'|' 'NR>2 {gsub(/ /,"",$1); print $1}' \
    | grep -E '^[0-9]+$' | grep -vx 0)
echo "deleting: ${snaps[*]:-none}"
for s in "${snaps[@]:-}"; do
    [[ -n $s ]] && snapper -c root delete "$s"
done
btrfs filesystem sync /
df -h /

echo
echo "=== step 3: move /nix to subvolume @nix ==="
systemctl stop nix-daemon.service nix-daemon.socket

TOP=$(mktemp -d)
cleanup() { mountpoint -q "$TOP" && umount "$TOP"; rmdir "$TOP" 2>/dev/null || true; }
trap cleanup EXIT

mount -t btrfs -o "$MNTOPTS,subvolid=5" "UUID=$UUID" "$TOP"
btrfs subvolume create "$TOP/@nix"
cp -a --reflink=auto /nix/. "$TOP/@nix/"

live=$(find /nix -mindepth 1 | wc -l)
copied=$(find "$TOP/@nix" -mindepth 1 | wc -l)
echo "entries: live=$live copied=$copied"
[[ $live -eq $copied ]] || { echo "entry count mismatch, not swapping" >&2; exit 1; }

umount "$TOP"
mv /nix /nix.old
mkdir /nix
cp /etc/fstab /etc/fstab.bak
printf 'UUID=%s\t/nix\tbtrfs\t%s,subvol=/@nix\t0 0\n' "$UUID" "$MNTOPTS" >> /etc/fstab
systemctl daemon-reload
mount /nix

findmnt -no SOURCE,FSROOT,TARGET /nix
mounted=$(findmnt -no FSROOT /nix)
[[ $mounted == /@nix ]] || { echo "/nix is not the new subvolume, keeping /nix.old" >&2; exit 1; }
now=$(find /nix -mindepth 1 | wc -l)
[[ $now -eq $live ]] || { echo "mounted tree incomplete, keeping /nix.old" >&2; exit 1; }

systemctl start nix-daemon.socket
rm -rf /nix.old
btrfs filesystem sync /

echo
echo "=== after ==="
df -h /
echo "fstab backup at /etc/fstab.bak"
