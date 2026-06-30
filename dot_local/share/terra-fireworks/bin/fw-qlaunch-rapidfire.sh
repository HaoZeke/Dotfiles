#!/usr/bin/env bash
# Drain READY fireworks into Terra Slurm (cpu partition). Uses existing LaunchPad.
set -euo pipefail
FW_HOME=${FW_HOME:-$HOME/Git/tmp/fw-mongo}
CFG=$FW_HOME/configs
FWV=$FW_HOME/fw_venv
export SLURM_CONF=${SLURM_CONF:-/etc/slurm-llnl/slurm.conf}
export FW_CONFIG_FILE=$CFG/FW_config.yaml
export PATH=$FWV/bin:$PATH
MAX_QUEUE=${MAX_QUEUE:-8}
sinfo >/dev/null
cd "$CFG"
exec qlaunch -c "$CFG" -q my_qadapter.yaml -l my_launchpad.yaml rapidfire \
  --nlaunches infinite --sleep 30 \
  --maxjobs_queue "$MAX_QUEUE" --maxjobs_block "$MAX_QUEUE"
