# rg-space-sweep

`rg-space-sweep` reports and removes rebuildable cache and build directories
under `$HOME`. It is installed by chezmoi from this source tree with:

```bash
cargo install --path "$HOME/.local/share/rg-space-sweep-src" --root "$HOME/.local" --force --locked
```

## Targets

The default category profile is:

```text
rust python tox
```

`all` expands the target set to:

```text
rust python tox pixi venv js
```

Targets map to rebuildable directories only. The cleaner refuses unexpected
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

## Cosmolab Remote Orchestration Profile

`rg.cosmolab` is a disposable remote build and deployment host. Treat cleanup
there as remote orchestration of the same CLI, not as a separate local target.

Inspect first over SSH:

```bash
ssh rg.cosmolab 'rg-space-sweep report all'
ssh rg.cosmolab 'rg-space-sweep clean --dry-run all'
```

Clean only after reviewing the dry-run output:

```bash
ssh rg.cosmolab 'rg-space-sweep clean --yes all'
```

Use `all` on cosmolab because remote build hosts accumulate Rust targets,
Python caches, tox environments, pixi/rattler caches, virtual environments, and
JavaScript cache directories from mixed workloads.

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
