######################
# Activation Scripts #
######################

# Rust
#######

# Almost all of these are with cargo install blah blah
programs=(zoxide atuin starship)

# Cache the generated init instead of regenerating it per shell.
#
# `starship init bash` emits a stub whose only job is to run `starship init
# bash --print-full-init`, so the prompt costs two starship invocations before
# it draws anything: 5.4 ms and 6.4 ms measured under xtrace. Asking for the
# full init directly and keeping the result on disk removes both, and does the
# same for zoxide.
#
# The cache is rebuilt when the binary is newer than it, which is a builtin
# test. Steady state is a source with no fork at all.
_init_cache="${XDG_CACHE_HOME:-$HOME/.cache}/shell-init"

for prog in "${programs[@]}"; do
    _prog_bin=$(command -v "$prog") || continue
    _prog_cache="$_init_cache/$prog.bash"
    if [[ ! -s $_prog_cache || $_prog_bin -nt $_prog_cache ]]; then
        [[ -d $_init_cache ]] || mkdir -p "$_init_cache"
        case $prog in
            starship) "$_prog_bin" init bash --print-full-init >|"$_prog_cache" ;;
            *) "$_prog_bin" init bash >|"$_prog_cache" ;;
        esac
    fi
    source "$_prog_cache"
done
unset prog _prog_bin _prog_cache _init_cache

# Emacs Stuff (cross platform)
# Local Variables:
# mode: shell-script
# End:
