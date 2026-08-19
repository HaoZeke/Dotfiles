# shellcheck shell=bash
# Yazi shell integration. The wrapper returns to the directory selected in
# Yazi while keeping the calling shell alive.
y() {
    local tmp cwd
    tmp=$(mktemp -t yazi-cwd.XXXXXX) || return 1
    command yazi "$@" --cwd-file="$tmp"
    if [ -r "$tmp" ]; then
        IFS= read -r cwd <"$tmp" || true
        if [ -n "$cwd" ] && [ -d "$cwd" ]; then
            builtin cd -- "$cwd" || true
        fi
    fi
    command rm -f -- "$tmp"
}
