#!/usr/bin/env bash
set -euo pipefail
umask 022

container_build() {
    local build_uid=$1 build_gid=$2 builder build_home=/home/paru-builder
    shift 2

    pacman -Syu --needed --noconfirm --noprogressbar git less sudo
    paru --version
    if ! getent group "$build_gid" >/dev/null; then
        groupadd -g "$build_gid" builder
    fi
    if ! builder=$(id -nu "$build_uid" 2>/dev/null); then
        builder=builder
        useradd -M -u "$build_uid" -g "$build_gid" -s /bin/bash "$builder"
    fi
    printf '%s ALL=(root) NOPASSWD: /usr/bin/pacman\n' "$builder" > /etc/sudoers.d/paru-builder
    chmod 0440 /etc/sudoers.d/paru-builder
    install -d -o "$build_uid" -g "$build_gid" "$build_home" "$build_home/packages"
    printf '[options]\nPgpFetch\nRemoveMake\n' > /etc/paru-build.conf

    cd "$build_home"
    runuser -u "$builder" -- env HOME="$build_home" PARU_CONF=/etc/paru-build.conf \
        PKGDEST="$build_home/packages" MAKEFLAGS=-j6 paru -S --aur --needed -- "$@"

    local package archive
    local -A installed_foreign=() explicit=()
    for package in "$@"; do
        explicit[$package]=1
    done
    while IFS= read -r package; do
        installed_foreign[$package]=1
    done < <(pacman -Qmq)
    shopt -s nullglob
    for archive in "$build_home"/packages/*.pkg.tar*; do
        [[ -f "$archive" && ! -L "$archive" && "$archive" != *.sig ]] || continue
        read -r package _ < <(pacman -Qp -- "$archive")
        [[ -v 'installed_foreign[$package]' ]] || continue
        install -m 0644 -o "$build_uid" -g "$build_gid" -- "$archive" /out/
    done
    for package in "$@"; do
        [[ -v 'installed_foreign[$package]' ]] || {
            printf 'The requested AUR package was not installed in the build container: %s\n' "$package" >&2
            return 1
        }
    done
    install -m 0644 -o "$build_uid" -g "$build_gid" /dev/null /out/runtime-dependencies
    while IFS= read -r package; do
        if [[ ! -v 'explicit[$package]' ]]; then
            printf '%s\n' "$package"
        fi
    done < <(pacman -Qmdq) > /out/runtime-dependencies
}

case "$1" in
    inside)
        shift
        container_build "$@"
        ;;
    build)
        shift
        job_dir=$(cd -- "$(dirname -- "$0")" && pwd -P)
        mkdir -m 0700 -- "$job_dir/out"
        srun --partition=cpu --time=02:00:00 --cpus-per-task=6 --mem=8G --pty \
            podman run --rm --pull=newer --cgroups=disabled --userns=keep-id --user=0 --interactive --tty \
            --volume "$job_dir/build.sh:/build.sh:ro" \
            --volume "$job_dir/out:/out" \
            --volume /usr/bin/paru:/usr/local/bin/paru:ro \
            docker.io/library/archlinux:base-devel \
            /bin/bash /build.sh inside "$(id -u)" "$(id -g)" "$@"
        ;;
    export)
        cd -- "$(dirname -- "$0")/out"
        shopt -s nullglob
        archives=( *.pkg.tar* )
        [[ -v "archives[0]" ]] || exit 1
        tar -cf - -- runtime-dependencies "${archives[@]}"
        ;;
    *)
        exit 2
        ;;
esac
