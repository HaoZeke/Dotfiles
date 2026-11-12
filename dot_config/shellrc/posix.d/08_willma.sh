# Willma-backed agents get the key from pass at launch, in memory only.
# omp resolves models.yml `apiKey: "!willma-key"` itself; the wrapper adds the
# mnemopi embedding key through a --config overlay read from a pipe, because
# that setting is a literal string and cannot run a command.
omp() {
    local k
    k=$(willma-key) || { command omp "$@"; return; }
    command omp --config <(printf 'mnemopi:\n  embeddingApiKey: "%s"\n' "$k") "$@"
}
opencode() { willma-run "$(command -v opencode)" "$@"; }
hermes() { willma-run "$(command -v hermes)" "$@"; }
