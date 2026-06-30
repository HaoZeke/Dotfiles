# Terra FireWorks + Slurm (campaigns / studies)

**Canonical LaunchPad**: Mongo on **rg.terra** (`podman` container `amsel-fw-mongo`, `127.0.0.1:27017`, DB `amsel_fw`). Do **not** invent a second LaunchPad; use this server for all campaigns/studies on Terra.

| Piece | Location |
|-------|----------|
| Data + venv | `~/Git/tmp/fw-mongo` |
| Configs | `~/Git/tmp/fw-mongo/configs` (synced from this share) |
| Share (chezmoi) | `~/.local/share/terra-fireworks` |
| Scheduler | Slurm partitions `cpu` / `gpu` / `long` |
| Cosmolab | target `rgam5terra`, `job_mode=slurm` (heavy non-FW work) |

## Bring-up

```bash
# once: Slurm (sudo / tsetup)
sudo bash ~/Git/tmp/fw-mongo/terra_slurm/apply-terra-slurm.sh   # or chezmoi terra-slurm apply

# mongo
podman start amsel-fw-mongo

# configs from chezmoi share
~/.local/share/terra-fireworks/bin/fw-sync-configs.sh
~/.local/share/terra-fireworks/bin/fw-doctor.sh

# register workflows (project-specific), then:
~/.local/share/terra-fireworks/bin/fw-qlaunch-rapidfire.sh
```

## Zazu

`zazu-agent` on Terra reports `slurm.*` (squeue/sinfo) **and** `fw.*` LaunchPad counts (`fw.ready`, `fw.running`, `fw.completed`, `fw.fizzled`, `fw.workflows`) when `$FW_HOME/fw_venv` + `my_launchpad.yaml` exist (default `FW_HOME=~/Git/tmp/fw-mongo`, override `FW_LAUNCHPAD_FILE`). Probe is bounded (`timeout 12`) so a wedged Mongo never hangs collect.

## Cosmolab vs FireWorks

- **Cosmolab `job_mode=slurm`**: interactive/agent `exec` under `srun` (one-off isolation).
- **FireWorks + qadapter**: multi-step **campaigns/studies** with Mongo DAG, `qlaunch` → Slurm.
