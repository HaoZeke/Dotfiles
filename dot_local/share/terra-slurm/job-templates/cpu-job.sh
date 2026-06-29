#!/usr/bin/env bash
#SBATCH --job-name=terra-cpu
#SBATCH --partition=local
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=8G
#SBATCH --time=01:00:00
#SBATCH --output=%x-%j.out
# Hygiene: explicit resources, no interactive GPU hogging
set -euo pipefail
echo "host=$(hostname) job=$SLURM_JOB_ID cpus=$SLURM_CPUS_ON_NODE"
# insert workload
sleep 1
