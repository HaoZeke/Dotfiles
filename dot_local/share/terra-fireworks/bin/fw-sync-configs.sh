#!/usr/bin/env bash
# Install share configs into FW_HOME/configs (idempotent).
set -euo pipefail
FW_HOME=${FW_HOME:-$HOME/Git/tmp/fw-mongo}
SHARE=${SHARE:-$HOME/.local/share/terra-fireworks}
mkdir -p "$FW_HOME/configs" "$FW_HOME/logs"
for f in my_launchpad.yaml FW_config.yaml my_qadapter.yaml; do
  install -m 644 "$SHARE/configs/$f" "$FW_HOME/configs/$f"
done
# Point FW_config at live configs dir
printf 'CONFIG_FILE_DIR: %s\n' "$FW_HOME/configs" > "$FW_HOME/configs/FW_config.yaml"
echo "Synced configs -> $FW_HOME/configs"
