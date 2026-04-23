#!/usr/bin/env bash
#
# btrfs-snapshot-cleanup.sh
#
# Deletes all dated snapshots under /.snapshots except the newest one of
# each prefix (@ and @home), then runs a metadata+data balance so df
# reflects the freed space.
#
# Safe to rerun.
# Run with: sudo bash /tmp/btrfs-snapshot-cleanup.sh

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "must run as root (use sudo)" >&2
  exit 1
fi

SNAP_DIR=/.snapshots

if [[ ! -d "$SNAP_DIR" ]]; then
  echo "no $SNAP_DIR; nothing to do" >&2
  exit 0
fi

echo "=== before ==="
df -h /home / 2>/dev/null | awk 'NR==1 || /\/(home)?$/'
echo

# Collect dated snapshot names for a given prefix ("@" or "@home"),
# newest last (lexical sort works because names are ISO-style timestamps).
list_snaps() {
  local prefix="$1"
  find "$SNAP_DIR" -mindepth 1 -maxdepth 1 -type d \
    -name "${prefix}.[0-9]*T[0-9]*" -printf '%f\n' 2>/dev/null | sort
}

delete_old() {
  local prefix="$1"
  local snaps
  mapfile -t snaps < <(list_snaps "$prefix")
  local count=${#snaps[@]}
  if (( count <= 1 )); then
    echo "[$prefix] $count snapshot(s); keeping all."
    return
  fi
  local keep="${snaps[-1]}"
  echo "[$prefix] $count snapshot(s); keeping $keep, deleting $((count - 1))."
  local i
  for (( i = 0; i < count - 1; i++ )); do
    local target="$SNAP_DIR/${snaps[i]}"
    echo "  delete $target"
    btrfs subvolume delete "$target"
  done
}

delete_old "@"
delete_old "@home"

# Free-space does not update until extents are balanced. -dusage=50 is a
# reasonable first pass; escalate if df still shows full.
echo
echo "=== balancing /home (may take a few minutes) ==="
btrfs balance start -dusage=50 /home || true
echo
echo "=== balancing / (may take a few minutes) ==="
btrfs balance start -dusage=50 / || true

echo
echo "=== after ==="
df -h /home / 2>/dev/null | awk 'NR==1 || /\/(home)?$/'
echo
echo "btrfs fi usage /home:"
btrfs fi usage /home | head -20

echo
echo "Done. If df still shows full, rerun with -dusage=75 then 90:"
echo "  sudo btrfs balance start -dusage=75 /home"
echo "  sudo btrfs balance start -dusage=90 /home"
