#!/usr/bin/env bash
# Bulletproof health check for Terra FireWorks + Slurm campaigns.
# Uses the existing LaunchPad (amsel-fw-mongo / amsel_fw) — never invents a second DB.
set -euo pipefail
FW_HOME=${FW_HOME:-$HOME/Git/tmp/fw-mongo}
CFG=${CFG:-$FW_HOME/configs}
FWV=${FWV:-$FW_HOME/fw_venv}
LP=${FW_LAUNCHPAD_FILE:-$CFG/my_launchpad.yaml}
export FW_HOME CFG FW_CONFIG_FILE=$CFG/FW_config.yaml FW_LAUNCHPAD_FILE=$LP
export SLURM_CONF=${SLURM_CONF:-/etc/slurm-llnl/slurm.conf}
FAIL=0
echo "=== Paths ==="
echo "FW_HOME=$FW_HOME"
echo "LP=$LP"
echo "SLURM_CONF=$SLURM_CONF"
echo "=== Slurm ==="
if ! sinfo; then
  echo "FAIL sinfo (SLURM_CONF=$SLURM_CONF)"
  FAIL=1
fi
echo "=== Mongo (LaunchPad host) ==="
if command -v podman >/dev/null; then
  if ! podman ps --format '{{.Names}} {{.Status}}' | grep -E 'amsel-fw-mongo|fw-mongo'; then
    echo "WARN: mongo container not in podman ps — start with: podman start amsel-fw-mongo"
  fi
fi
if ss -ltn | grep -q ':27017'; then
  echo "mongo port 27017 listening"
else
  echo "FAIL: 27017 not listening"
  FAIL=1
fi
echo "=== LaunchPad ping (existing server) ==="
if [[ -x "$FWV/bin/python" && -f "$LP" ]]; then
  if ! "$FWV/bin/python" - "$LP" <<'PY'
import sys
from fireworks import LaunchPad
lp = LaunchPad.from_file(sys.argv[1])
print("PING", lp.connection.admin.command("ping"))
for st in ("READY", "RUNNING", "COMPLETED", "FIZZLED", "RESERVED"):
    try:
        n = lp.get_fw_ids({"state": st})
        print(f"FW_{st}", len(n) if n is not None else 0)
    except Exception as e:
        print(f"FW_{st}_ERR", e)
print("WF_COUNT", lp.workflows.count_documents({}))
PY
  then
    echo "FAIL LaunchPad probe"
    FAIL=1
  fi
else
  echo "WARN: need $FWV/bin/python and $LP"
  FAIL=1
fi
echo "=== qadapter (Slurm partitions) ==="
if [[ -f "$CFG/my_qadapter.yaml" ]]; then
  grep -E '^(queue|walltime|_fw_q_type):' "$CFG/my_qadapter.yaml" || true
  if ! grep -q 'SLURM_CONF=/etc/slurm-llnl/slurm.conf' "$CFG/my_qadapter.yaml"; then
    echo "WARN: qadapter pre_rocket should export SLURM_CONF=/etc/slurm-llnl/slurm.conf"
  fi
fi
echo "=== cosmolab target (expect rgam5terra + job_mode slurm) ==="
if [[ -f "$HOME/.config/cosmolab/targets.json" ]]; then
  python3 -c "import json;d=json.load(open('$HOME/.config/cosmolab/targets.json'));print('default',d.get('default'));t=d['targets'].get('rgam5terra',{});print('job_mode',t.get('job_mode'));print('ssh_host',t.get('ssh_host'))"
else
  echo "no cosmolab targets.json (optional for FW campaigns)"
fi
if [[ "$FAIL" -ne 0 ]]; then
  echo "DOCTOR_FAIL"
  exit 1
fi
echo "DOCTOR_OK"
