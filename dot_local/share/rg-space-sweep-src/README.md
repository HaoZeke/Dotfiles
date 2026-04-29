# rg-space-sweep

`rg-space-sweep` reports and removes rebuildable cache and build directories
under `$HOME`. It is installed by chezmoi from this source tree with:

```bash
cargo install --path "$HOME/.local/share/rg-space-sweep-src" --root "$HOME/.local" --force --locked
```

## Category Sets

The default category profile is:

```text
rust python tox
```

`all` expands the target set to:

```text
rust python tox pixi venv js
```

Category sets map to rebuildable directories only. The cleaner refuses unexpected
paths and will not remove `$HOME`, `.local/bin`, or `.cargo/bin`.

## Local Default Profile

Use the local profile for day-to-day workstation cleanup.

Always inspect first:

```bash
rg-space-sweep report default
rg-space-sweep clean --dry-run default
```

Run the destructive pass only after the dry-run output looks correct:

```bash
rg-space-sweep clean --yes default
```

For low-space automation, the user service uses the broader profile and only
fires when `$HOME` free space is below the configured threshold:

```bash
rg-space-sweep auto-clean --min-free-gb 10 all
```

`auto-clean` implies `--yes` when the threshold is crossed, so use `report` and
`clean --dry-run` for manual inspection before relying on the timer.

## Remote Machine Profiles

Remote machine cleanup is controlled from the local `rg-space-sweep` binary. The
remote host only needs SSH, `bash`, `find`, `du`, `awk`, `sort`, and standard
coreutils; it does not need `rg-space-sweep` installed.

`cosmolab` and `rg.cosmolab` are built-in aliases for the disposable
`rg.cosmolab` build and deployment host.

Inspect first:

```bash
rg-space-sweep target list
rg-space-sweep target show rg.cosmolab
rg-space-sweep target check rg.cosmolab
rg-space-sweep pressure --target rg.cosmolab
rg-space-sweep report --target cosmolab --limit 20 all
rg-space-sweep clean --target cosmolab --dry-run all
```

Clean only after reviewing the dry-run output:

```bash
rg-space-sweep clean --target cosmolab --yes all
```

Use `all` on cosmolab because remote build hosts accumulate Rust targets,
Python caches, tox environments, pixi/rattler caches, virtual environments, and
JavaScript cache directories from mixed workloads.

Use `--json` with `target list`, `target show`, `target check`, `pressure`,
`report`, or `clean --dry-run` when another tool needs stable output.

The default target config path is `~/.config/rg-space-sweep/targets.toml`;
`--target-config PATH` overrides it. The built-in `rg.cosmolab` target can be
overridden there, including SSH-via-GSocket for cases where the VPN route is not
available. Store the GSocket secret in a file with strict permissions; do not
put raw secrets in TOML, shell history, or repository files.

```toml
[targets."rg.cosmolab"]
host = "rg.cosmolab"
user = "goswami"
home = "/home/goswami"
runner = "gsocket"
gsocket_secret_file = "/run/user/1001/gsocket/rg.cosmolab.secret"
ssh_identity = "/home/rgoswami/.ssh/id_cosmolab"
host_key_alias = "rg.cosmolab-gsocket"
roots = ["/home/goswami"]
prune = ["/home/goswami/Git/.archive"]
exclude = ["/home/goswami/.cache/keep"]
categories = ["python", "rust", "tox"]
min_free_gb = 200
snapshots = false
```

Then run:

```bash
rg-space-sweep target check rg.cosmolab
rg-space-sweep report --target rg.cosmolab
rg-space-sweep clean --target rg.cosmolab --dry-run
```

## Snapshots

Snapshot cleanup is local-only. It inspects `/.snapshots`, writes a root-only
Btrfs cleanup script, and prints the `sudo bash <path>` command to run on the
same machine:

```bash
rg-space-sweep snapshots
sudo bash /run/user/"$(id -u)"/rg-space-sweep/btrfs-snapshot-cleanup.sh
```

The standard snapshot script keeps the newest dated `@` and `@home` snapshot and
deletes older dated pairs. If that plan reports zero deletes and space is still
tight, generate the aggressive variant:

```bash
rg-space-sweep snapshots --aggressive
sudo bash /run/user/"$(id -u)"/rg-space-sweep/btrfs-snapshot-cleanup.sh
```

Do not run snapshot mode as part of cosmolab cleanup. The expected cosmolab
filesystem is ext4 with no Btrfs snapshot tree, so `rg-space-sweep snapshots`
has no useful remote action there and should report no dated snapshots under
`/.snapshots`.
