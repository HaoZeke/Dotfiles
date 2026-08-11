#!/usr/bin/env bash
# rgam5terra: delete the snapshots that still hold the collected nix store.
#
# Replaces step 2 of reclaim-disk.sh, which enumerated snapshots by parsing
# `snapper list`. That command goes through snapperd over D-Bus and returns
# nothing when the call times out; the parse then yielded an empty list and the
# step reported "deleting: none" instead of failing. Enumerate the subvolume
# directories under /.snapshots instead, which needs no daemon.
#
# A snapshot taken before the collection holds every path the collection
# unlinked, so its store is larger than the live one. Snapshots whose store
# matches the live store contain nothing the live tree has dropped, and are kept
# as rollback points.
#
# Run as root on rgam5terra, after /nix has moved to @nix.

set -euo pipefail

[[ $EUID -eq 0 ]] || { echo "must run as root" >&2; exit 1; }
[[ -d /.snapshots ]] || { echo "/.snapshots missing" >&2; exit 1; }

echo "=== before ==="
df -h /

live=$(find /nix/store -mindepth 1 -maxdepth 1 -printf . 2>/dev/null | wc -c)
[[ $live -gt 0 ]] || { echo "live store looks empty, refusing to compare" >&2; exit 1; }
echo "live store entries: $live"

mapfile -t all < <(ls /.snapshots 2>/dev/null | grep -E '^[0-9]+$' | sort -n)
[[ ${#all[@]} -gt 0 ]] || { echo "no snapshots found, nothing to do" >&2; exit 1; }
echo "snapshots present: ${all[*]}"

# Count without a pipeline that can fail: once /nix is its own subvolume a fresh
# snapshot carries an empty /nix mountpoint and no store directory at all, so ls
# exits non-zero and pipefail would take the whole script down.
store_entries() {
    local d=$1
    if [[ -d $d ]]; then
        find "$d" -mindepth 1 -maxdepth 1 -printf . 2>/dev/null | wc -c
    else
        echo 0
    fi
}

doomed=()
for s in "${all[@]}"; do
    n=$(store_entries "/.snapshots/$s/snapshot/nix/store")
    if [[ $n -gt $live ]]; then
        doomed+=("$s")
        echo "  $s holds $n store entries -> delete"
    else
        echo "  $s holds $n store entries -> keep"
    fi
done

[[ ${#doomed[@]} -gt 0 ]] || { echo "nothing pins the collected store; done"; exit 0; }

echo
echo "deleting: ${doomed[*]}"
for s in "${doomed[@]}"; do
    if snapper -c root delete "$s"; then
        echo "  snapper deleted $s"
    else
        echo "  snapper failed on $s, deleting the subvolume directly"
        btrfs subvolume delete "/.snapshots/$s/snapshot"
        rm -rf "/.snapshots/$s"
    fi
done

echo
echo "waiting for the btrfs cleaner to release the extents"
btrfs filesystem sync /
for _ in $(seq 1 12); do
    sleep 10
    btrfs filesystem sync /
done

echo
echo "=== after ==="
df -h /
echo "snapshots kept: $(ls /.snapshots | grep -E '^[0-9]+$' | sort -n | tr '\n' ' ')"
