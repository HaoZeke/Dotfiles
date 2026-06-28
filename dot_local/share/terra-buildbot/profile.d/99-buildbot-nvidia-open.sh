# Terra / rgam5terra: RTX 5070 Blackwell — use nvidia-open stack only (not nvidia-dkms).
# Kernel modules: nvidia-open-dkms (Dual MIT/GPL). Userspace: nvidia-utils matching branch.
export NVIDIA_DRIVER_CAPABILITIES="${NVIDIA_DRIVER_CAPABILITIES:-compute,utility}"
# Prefer host CUDA toolkit when present (see pacman cuda package).
