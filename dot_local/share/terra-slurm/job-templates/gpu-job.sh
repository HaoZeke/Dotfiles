#!/usr/bin/env bash
#SBATCH --job-name=terra-gpu
#SBATCH --partition=local
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=16G
#SBATCH --gres=gpu:1
#SBATCH --time=02:00:00
#SBATCH --output=%x-%j.out
set -euo pipefail
echo "host=$(hostname) job=$SLURM_JOB_ID gres=${CUDA_VISIBLE_DEVICES:-unset}"
command -v nvidia-smi >/dev/null && nvidia-smi -L || true
# insert GPU workload
sleep 1
