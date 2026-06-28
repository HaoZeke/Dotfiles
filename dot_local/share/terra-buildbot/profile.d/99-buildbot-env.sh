# /etc/profile.d/99-buildbot-env.sh
export SCCACHE_DIR="${SCCACHE_DIR:-/var/cache/sccache}"
export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-50G}"
export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}"
export CMAKE_C_COMPILER_LAUNCHER="${CMAKE_C_COMPILER_LAUNCHER:-sccache}"
export CMAKE_CXX_COMPILER_LAUNCHER="${CMAKE_CXX_COMPILER_LAUNCHER:-sccache}"
export CMAKE_CUDA_COMPILER_LAUNCHER="${CMAKE_CUDA_COMPILER_LAUNCHER:-sccache}"
export APPTAINER_CACHEDIR="${APPTAINER_CACHEDIR:-/var/lib/apptainer-cache}"
export SINGULARITY_CACHEDIR="${SINGULARITY_CACHEDIR:-$APPTAINER_CACHEDIR}"
# Prefer builds on quota'd subvol
export BUILDS_ROOT="${BUILDS_ROOT:-/var/lib/builds}"
export TMPDIR="${TMPDIR:-/var/lib/builds/tmp}"
# Nix (if multi-user daemon present)
if [[ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]]; then
  # shellcheck source=/dev/null
  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
elif [[ -e "$HOME/.nix-profile/etc/profile.d/nix.sh" ]]; then
  # shellcheck source=/dev/null
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"
fi
# Android (user layout)
if [[ -f "$HOME/.config/hzlinux/android-env.sh" ]]; then
  # shellcheck source=/dev/null
  . "$HOME/.config/hzlinux/android-env.sh"
fi
# mold as default linker for gcc/clang when using ld.lld/mold wrappers is project-specific;
# expose helper:
buildbot-j() { nproc 2>/dev/null || echo 32; }
