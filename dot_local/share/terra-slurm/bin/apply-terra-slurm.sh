#!/usr/bin/env bash
# Apply Terra single-node Slurm from ~/.local/share/terra-slurm (chezmoi).
# Run in tsetup tmux with sudo password available.
set -euo pipefail
SHARE="${HOME}/.local/share/terra-slurm"
[[ -d "$SHARE/etc" ]] || { echo "missing $SHARE/etc — chezmoi apply first"; exit 1; }
echo "Installing packages (slurm-llnl)…"
sudo pacman -S --needed --noconfirm slurm-llnl || sudo pacman -S --needed --noconfirm slurm
# Detect CPUs / mem for node line
CPUS=$(nproc)
MEM_MB=$(awk '/MemTotal/ {printf "%d", $2/1024}' /proc/meminfo)
sudo mkdir -p /etc/slurm /var/spool/slurmd /var/spool/slurmctld /var/log/slurm
# Ensure slurm user
id slurm >/dev/null 2>&1 || sudo useradd -r -s /usr/bin/nologin slurm
sudo chown slurm:slurm /var/spool/slurmctld || true
sudo chown root:root /var/spool/slurmd || true
# Render node memory/cpu into conf
TMP=$(mktemp)
sed -e "s/CPUs=32/CPUs=${CPUS}/" -e "s/RealMemory=128000/RealMemory=${MEM_MB}/" \
  -e "s/SlurmctldHost=rgam5terra/SlurmctldHost=$(hostname -s)/" \
  -e "s/NodeName=rgam5terra/NodeName=$(hostname -s)/" \
  "$SHARE/etc/slurm.conf" > "$TMP"
sudo install -m 644 "$TMP" /etc/slurm/slurm.conf
sudo install -m 644 "$SHARE/etc/gres.conf" /etc/slurm/gres.conf
sudo install -m 644 "$SHARE/etc/cgroup.conf" /etc/slurm/cgroup.conf
rm -f "$TMP"
sudo systemctl enable --now slurmctld.service slurmd.service || {
  echo "systemd enable failed — try: sudo slurmctld -D & sudo slurmd -D &"
}
sleep 2
scontrol ping || true
sinfo || true
echo "Slurm apply done. Templates: $SHARE/job-templates"
echo "Example: sbatch $SHARE/job-templates/cpu-job.sh"
