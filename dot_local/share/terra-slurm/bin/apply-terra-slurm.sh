#!/usr/bin/env bash
# Apply Terra single-node Slurm from ~/.local/share/terra-slurm (chezmoi).
# Arch Linux package slurm-llnl reads /etc/slurm-llnl/slurm.conf (NOT /etc/slurm/).
# Run with sudo in tsetup tmux.
set -euo pipefail
SHARE="${HOME}/.local/share/terra-slurm"
[[ -d "$SHARE/etc" ]] || { echo "missing $SHARE/etc — sync chezmoi share first"; exit 1; }

echo "Installing packages (slurm-llnl + munge)…"
sudo pacman -S --needed --noconfirm slurm-llnl munge || true

CPUS=$(nproc)
MEM_MB=$(awk '/MemTotal/ {printf "%d", $2/1024}' /proc/meminfo)
HN=$(hostname -s)

# Arch paths
ETC=/etc/slurm-llnl
sudo mkdir -p "$ETC" /var/spool/slurmd /var/spool/slurmctld /var/log/slurm
id slurm >/dev/null 2>&1 || sudo useradd -r -s /usr/bin/nologin -d /var/lib/slurm -M slurm
sudo chown -R slurm:slurm /var/spool/slurmctld
sudo chown root:root /var/spool/slurmd
sudo chmod 755 /var/spool/slurmd /var/spool/slurmctld

# Munge key if missing
if [[ ! -f /etc/munge/munge.key ]]; then
  sudo mkdir -p /etc/munge
  sudo dd if=/dev/urandom of=/etc/munge/munge.key bs=1 count=1024 status=none
  sudo chown munge:munge /etc/munge/munge.key
  sudo chmod 400 /etc/munge/munge.key
fi
sudo systemctl enable --now munge.service

TMP=$(mktemp)
sed -e "s/CPUs=32/CPUs=${CPUS}/" \
    -e "s/RealMemory=[0-9]*/RealMemory=${MEM_MB}/" \
    -e "s/SlurmctldHost=.*/SlurmctldHost=${HN}/" \
    -e "s/NodeName=[^ ]*/NodeName=${HN}/" \
    -e "s/Nodes=rgam5terra/Nodes=${HN}/" \
    -e "s|SlurmctldPidFile=.*|SlurmctldPidFile=/run/slurmctld/slurmctld.pid|" \
    -e "s|SlurmdPidFile=.*|SlurmdPidFile=/run/slurm/slurmd.pid|" \
    "$SHARE/etc/slurm.conf" > "$TMP"

# Ensure a localhost controller addr for single-node (avoid DNS SRV lookups).
# Note: SlurmdAddr is NOT a valid slurm.conf key (removed in modern Slurm); node
# addresses belong on NodeName via NodeAddr. Injecting it makes the whole config
# fail to parse, which silently drops Slurm back to the DNS SRV config source.
grep -q '^SlurmctldAddr=' "$TMP" || sed -i "/^SlurmctldHost=/a\\
SlurmctldAddr=127.0.0.1" "$TMP"

sudo install -m 644 -o root -g root "$TMP" "$ETC/slurm.conf"
sudo install -m 644 -o root -g root "$SHARE/etc/gres.conf" "$ETC/gres.conf"
sudo install -m 644 -o root -g root "$SHARE/etc/cgroup.conf" "$ETC/cgroup.conf"
rm -f "$TMP"

# Compat symlink if anything still looks under /etc/slurm
if [[ ! -e /etc/slurm ]]; then
  sudo ln -sfn slurm-llnl /etc/slurm
fi

# Print detected node hardware for sanity (slurmd -C, not slurmctld -C, is the
# node-detection flag in modern Slurm; slurmctld -C just prints usage).
/usr/bin/slurmd -C 2>&1 | tee /tmp/slurmd-C.out || echo "WARN: slurmd -C failed (see /tmp/slurmd-C.out)"

sudo systemctl daemon-reload
sudo systemctl reset-failed slurmctld.service slurmd.service 2>/dev/null || true
sudo systemctl enable slurmctld.service slurmd.service
sudo systemctl restart slurmctld.service
sleep 2
sudo systemctl restart slurmd.service
sleep 2

echo "=== status ==="
systemctl is-active munge slurmctld slurmd || true
scontrol ping || true
sinfo || true
echo "Templates: $SHARE/job-templates"
echo "Example: sbatch $SHARE/job-templates/cpu-job.sh"
