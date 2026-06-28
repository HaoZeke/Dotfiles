# Optimal defaults for Terra (see Software/Infra/2026-06-27-terra-buildbot.org)
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
# Sensible default job count for interactive shells (leave headroom)
export CMAKE_BUILD_PARALLEL_LEVEL="${CMAKE_BUILD_PARALLEL_LEVEL:-28}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-28}"
