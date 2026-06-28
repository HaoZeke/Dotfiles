# Optimal defaults for Terra / rgam5terra (9950X 16c/32t, ASRock B650I, RTX 5070)
# See Software/Infra/2026-06-27-terra-buildbot.org — leave headroom for sshd/netdata/GPU.
export SCCACHE_DIR="${SCCACHE_DIR:-/var/cache/sccache}"
export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-50G}"
export SCCACHE_IDLE_TIMEOUT="${SCCACHE_IDLE_TIMEOUT:-0}"
export SCCACHE_MAX_FRAME_LENGTH="${SCCACHE_MAX_FRAME_LENGTH:-200000000}"
export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}"
# Prefer mold for host cargo when installed (huge link speedup on 9950X)
if [[ -x /usr/bin/mold ]] && [[ -z "${CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER:-}" ]]; then
  export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
  export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-fuse-ld=mold"
fi
# Job defaults: min(28, nproc-4) on ≥16 threads so interactive/GPU keep 4 threads.
# Explicit 28 is correct for measured 32-thread 9950X; dynamic fallback for smaller hosts.
_bb_n=$(nproc 2>/dev/null || echo 32)
_bb_j=$(( _bb_n > 8 ? _bb_n - 4 : _bb_n ))
[[ $_bb_j -gt 28 ]] && _bb_j=28
export CMAKE_BUILD_PARALLEL_LEVEL="${CMAKE_BUILD_PARALLEL_LEVEL:-$_bb_j}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$_bb_j}"
unset _bb_n _bb_j
