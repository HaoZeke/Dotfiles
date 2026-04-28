use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

const FIND_BIN: &str = "/usr/bin/find";
const DU_BIN: &str = "/usr/bin/du";

/// First-pass btrfs snapshot cleanup: keeps the newest dated snapshot of
/// each prefix (@ and @home) as a safety net.
const BTRFS_CLEANUP_SCRIPT: &str = include_str!("btrfs-cleanup.sh");

/// Aggressive variant: removes TODAY's snapshots too and runs a full
/// `-dusage=100 -musage=100` balance so previously-snapshot-held blocks
/// are actually released.
const BTRFS_CLEANUP_SCRIPT_AGGRESSIVE: &str = r#"#!/usr/bin/env bash
# btrfs-snapshot-cleanup-aggressive.sh
# Removes ALL dated snapshots and runs full balance. Loses today's
# rollback. Run with: sudo bash <this-script>
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "must run as root (use sudo)" >&2
  exit 1
fi

SNAP_DIR=/.snapshots

echo "=== before ==="
df -h /home / 2>/dev/null | awk 'NR==1 || /\/(home)?$/'
echo

mapfile -t snaps < <(
  find "$SNAP_DIR" -mindepth 1 -maxdepth 1 -type d \
    -regextype posix-extended \
    -regex ".*/@(home)?\.[0-9]+T[0-9]+$" -printf '%p\n' 2>/dev/null | sort
)

if ((${#snaps[@]} == 0)); then
  echo "no dated snapshots found under $SNAP_DIR"
else
  echo "deleting ${#snaps[@]} dated snapshot(s):"
  for s in "${snaps[@]}"; do
    echo "  $s"
    btrfs subvolume delete "$s"
  done
fi

echo
echo "=== balance /home (-dusage=100 -musage=100) ==="
btrfs balance start -dusage=100 -musage=100 /home || true
echo
echo "=== balance / ==="
btrfs balance start -dusage=100 -musage=100 / || true

echo
echo "=== after ==="
df -h /home / 2>/dev/null | awk 'NR==1 || /\/(home)?$/'
btrfs fi usage /home | head -20
"#;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Category {
    Rust,
    Python,
    Pixi,
    Tox,
    Venv,
    Js,
}

impl Category {
    fn label(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Pixi => "pixi",
            Self::Tox => "tox",
            Self::Venv => "venv",
            Self::Js => "js",
        }
    }

    fn expand(token: &str) -> Result<Vec<Self>, String> {
        match token {
            "default" => Ok(vec![Self::Rust, Self::Python, Self::Tox]),
            "all" => Ok(vec![
                Self::Rust,
                Self::Python,
                Self::Pixi,
                Self::Tox,
                Self::Venv,
                Self::Js,
            ]),
            "rust" => Ok(vec![Self::Rust]),
            "python" => Ok(vec![Self::Python]),
            "pixi" => Ok(vec![Self::Pixi]),
            "tox" => Ok(vec![Self::Tox]),
            "venv" => Ok(vec![Self::Venv]),
            "js" => Ok(vec![Self::Js]),
            other => Err(format!("unknown category: {other}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Report,
    Clean,
    /// Check free space on $HOME; clean only if it is below the threshold.
    /// Always implies `--yes` when it fires. Intended for a systemd timer.
    AutoClean,
    /// Report dated btrfs snapshots under /.snapshots and write a root-only
    /// cleanup script. Cannot delete subvolumes directly because btrfs
    /// operations require root; the user runs the generated script.
    Snapshots,
}

#[derive(Debug)]
struct Options {
    mode: Mode,
    categories: Vec<Category>,
    dry_run: bool,
    yes: bool,
    limit: usize,
    /// For AutoClean: only clean when free space on $HOME drops below N GB.
    min_free_gb: u64,
    /// For Snapshots: where to write the cleanup script.
    script_path: PathBuf,
    /// For Snapshots: also include TODAY's snapshots (aggressive).
    aggressive: bool,
}

#[derive(Clone, Debug)]
struct Candidate {
    category: Category,
    path: PathBuf,
}

#[derive(Clone, Debug)]
struct Entry {
    category: Category,
    path: PathBuf,
    size: u64,
}

fn default_snapshot_script_path_for(runtime_dir: Option<&Path>, uid: u32) -> PathBuf {
    match runtime_dir {
        Some(dir) if dir.is_absolute() => dir.join("rg-space-sweep/btrfs-snapshot-cleanup.sh"),
        _ => PathBuf::from(format!(
            "/tmp/rg-space-sweep-{uid}/btrfs-snapshot-cleanup.sh"
        )),
    }
}

fn default_snapshot_script_path() -> PathBuf {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    default_snapshot_script_path_for(runtime_dir.as_deref(), unsafe { libc::geteuid() })
}

fn ensure_script_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;

    let is_default_private_dir = parent
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "rg-space-sweep" || name.starts_with("rg-space-sweep-"))
        .unwrap_or(false);
    if is_default_private_dir {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(parent)
            .map_err(|e| format!("stat {}: {e}", parent.display()))?
            .permissions();
        perms.set_mode(0o700);
        fs::set_permissions(parent, perms)
            .map_err(|e| format!("chmod {}: {e}", parent.display()))?;
    }

    Ok(())
}

fn usage() -> &'static str {
    "\
usage: rg-space-sweep [report|clean|auto-clean|snapshots] [--dry-run] [--yes] [--limit N] [--min-free-gb N] [--script-path PATH] [default|all|rust|python|pixi|tox|venv|js]

report
    Show category totals and the largest matching cache/build directories.

clean
    Remove the matching directories. Requires --yes, or use --dry-run to preview.

auto-clean
    Check free space on $HOME; clean only if below --min-free-gb (default 10).
    Always implies --yes when firing. Intended for a systemd timer.

snapshots
    Report dated btrfs snapshots under /.snapshots and write a root-only
    cleanup script (keeps newest @ and @home, deletes older pairs, runs
    balance). The default path is under the user-scoped runtime directory.
    Override via --script-path. Run the script with `sudo bash <path>`.

default
    rust python tox

all
    default + pixi + venv + js"
}

fn parse_args() -> Result<Options, String> {
    let mut mode = Mode::Report;
    let mut dry_run = false;
    let mut yes = false;
    let mut limit = 20usize;
    let mut min_free_gb: u64 = 10;
    let mut script_path = default_snapshot_script_path();
    let mut aggressive = false;
    let mut category_tokens = Vec::new();

    let mut args = env::args().skip(1);
    if let Some(first) = args.next() {
        match first.as_str() {
            "report" => mode = Mode::Report,
            "clean" => mode = Mode::Clean,
            "auto-clean" => mode = Mode::AutoClean,
            "snapshots" => mode = Mode::Snapshots,
            "-h" | "--help" | "help" => return Err(usage().to_string()),
            other => category_tokens.push(other.to_string()),
        }
    }

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--yes" => yes = true,
            "--aggressive" => aggressive = true,
            "--limit" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--limit requires a value".to_string())?;
                limit = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --limit value: {value}"))?;
            }
            "--min-free-gb" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--min-free-gb requires a value".to_string())?;
                min_free_gb = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --min-free-gb value: {value}"))?;
            }
            "--script-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--script-path requires a value".to_string())?;
                script_path = PathBuf::from(value);
            }
            "-h" | "--help" | "help" => return Err(usage().to_string()),
            other => category_tokens.push(other.to_string()),
        }
    }

    if category_tokens.is_empty() {
        category_tokens.push(match mode {
            Mode::AutoClean => "all".to_string(),
            _ => "default".to_string(),
        });
    }

    let mut seen = BTreeSet::new();
    let mut categories = Vec::new();
    for token in category_tokens {
        for category in Category::expand(&token)? {
            if seen.insert(category) {
                categories.push(category);
            }
        }
    }

    if mode == Mode::Clean && !dry_run && !yes {
        return Err("refusing to clean without --yes; use --dry-run to preview first".to_string());
    }

    Ok(Options {
        mode,
        categories,
        dry_run,
        yes,
        limit,
        min_free_gb,
        script_path,
        aggressive,
    })
}

/// List dated btrfs snapshots under /.snapshots, newest last.
fn dated_snapshots() -> Vec<PathBuf> {
    let snap_dir = Path::new("/.snapshots");
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(snap_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        // Matches @.<digits>T<digits> and @home.<digits>T<digits>
        let rest = if let Some(r) = name_str.strip_prefix("@home.") {
            r
        } else if let Some(r) = name_str.strip_prefix("@.") {
            r
        } else {
            continue;
        };
        let parts: Vec<&str> = rest.splitn(2, 'T').collect();
        if parts.len() == 2
            && !parts[0].is_empty()
            && !parts[1].is_empty()
            && parts[0].chars().all(|c| c.is_ascii_digit())
            && parts[1].chars().all(|c| c.is_ascii_digit())
        {
            out.push(entry.path());
        }
    }
    out.sort();
    out
}

fn write_snapshot_script(options: &Options) -> Result<(), String> {
    let snaps = dated_snapshots();
    if snaps.is_empty() {
        println!("no dated snapshots found under /.snapshots; nothing to script");
        return Ok(());
    }
    println!("found {} dated snapshot(s) under /.snapshots:", snaps.len());
    for s in &snaps {
        println!("  {}", s.display());
    }
    let script = if options.aggressive {
        BTRFS_CLEANUP_SCRIPT_AGGRESSIVE
    } else {
        BTRFS_CLEANUP_SCRIPT
    };
    ensure_script_parent(&options.script_path)?;
    fs::write(&options.script_path, script)
        .map_err(|e| format!("write {}: {e}", options.script_path.display()))?;
    // Keep the generated root script readable only by the current user.
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(&options.script_path) {
        let mut p = meta.permissions();
        p.set_mode(0o700);
        let _ = fs::set_permissions(&options.script_path, p);
    }
    let mode_label = if options.aggressive {
        "aggressive (removes TODAY's snapshots too)"
    } else {
        "standard (keeps newest of each prefix as safety)"
    };
    println!();
    println!(
        "wrote {} script to: {}",
        mode_label,
        options.script_path.display()
    );
    println!("run with: sudo bash {}", options.script_path.display());
    if !options.aggressive {
        println!();
        println!("if space is still tight after the first pass, rerun with --aggressive:");
        println!("  rg-space-sweep snapshots --aggressive");
    }
    Ok(())
}

/// Return free bytes on the filesystem that owns `path`, via statvfs.
fn free_bytes_for(path: &Path) -> Result<u64, String> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c_path =
        CString::new(path.as_os_str().as_bytes()).map_err(|e| format!("path contains NUL: {e}"))?;
    // SAFETY: libc::statvfs writes into a zeroed struct we own; we check the
    // return value before reading fields.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return Err(format!(
                "statvfs({}) failed: {}",
                path.display(),
                std::io::Error::last_os_error()
            ));
        }
        Ok(stat.f_bavail as u64 * stat.f_frsize as u64)
    }
}

fn home_dir() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn run_output(program: &str, args: &[OsString]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run {program}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{program} exited with status {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("invalid utf-8 from {program}: {err}"))
}

fn run_find_output(args: &[OsString]) -> Result<String, String> {
    let output = Command::new(FIND_BIN)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .map_err(|err| format!("failed to run {FIND_BIN}: {err}"))?;
    String::from_utf8(output.stdout).map_err(|err| format!("invalid utf-8 from {FIND_BIN}: {err}"))
}

fn find_dirs(home: &Path, terms: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut args = vec![
        home.as_os_str().to_os_string(),
        OsString::from("-xdev"),
        OsString::from("("),
        OsString::from("-path"),
        home.join(".cache").into_os_string(),
        OsString::from("-o"),
        OsString::from("-path"),
        home.join(".cargo").into_os_string(),
        OsString::from("-o"),
        OsString::from("-path"),
        home.join(".local/share/containers").into_os_string(),
        OsString::from("-o"),
        OsString::from("-path"),
        home.join(".local/share/Trash").into_os_string(),
        OsString::from(")"),
        OsString::from("-prune"),
        OsString::from("-o"),
        OsString::from("-type"),
        OsString::from("d"),
    ];
    args.extend(terms.iter().map(OsString::from));
    args.push(OsString::from("-prune"));
    args.push(OsString::from("-print"));
    Ok(run_find_output(&args)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn find_named_dirs(home: &Path, name: &str) -> Result<Vec<PathBuf>, String> {
    find_dirs(home, &["-name", name])
}

fn find_path_dirs(home: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    find_dirs(home, &["-path", pattern])
}

fn has_cargo_parent(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    parent.join("Cargo.toml").is_file() || parent.join("Cargo.lock").is_file()
}

fn looks_like_rust_target(path: &Path) -> bool {
    path.join("debug").is_dir()
        || path.join("release").is_dir()
        || path.join(".fingerprint").is_dir()
        || path.join(".rustc_info.json").exists()
}

fn find_cargo_targets(home: &Path) -> Result<Vec<PathBuf>, String> {
    Ok(find_named_dirs(home, "target")?
        .into_iter()
        .filter(|path| has_cargo_parent(path) && looks_like_rust_target(path))
        .collect())
}

fn emit_category_paths(home: &Path, category: Category) -> Result<Vec<PathBuf>, String> {
    let paths = match category {
        Category::Rust => {
            let mut paths = find_cargo_targets(home)?;
            paths.push(home.join(".cargo/registry/cache"));
            paths.push(home.join(".cargo/git/db"));
            paths
        }
        Category::Python => {
            let mut paths = Vec::new();
            for name in [".pytest_cache", ".mypy_cache", ".ruff_cache", ".hypothesis"] {
                paths.extend(find_named_dirs(home, name)?);
            }
            paths.extend([
                home.join(".cache/pip"),
                home.join(".cache/uv"),
                home.join(".cache/pre-commit"),
                home.join(".local/share/hatch"),
            ]);
            paths
        }
        Category::Pixi => vec![home.join(".cache/rattler/cache")],
        Category::Tox => find_named_dirs(home, ".tox")?,
        Category::Venv => find_named_dirs(home, ".venv")?,
        Category::Js => {
            let mut paths = find_path_dirs(home, "*/node_modules/.cache")?;
            paths.push(home.join(".npm"));
            paths
        }
    };
    Ok(paths)
}

fn collect_candidates(home: &Path, categories: &[Category]) -> Result<Vec<Candidate>, String> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();

    for &category in categories {
        for path in emit_category_paths(home, category)? {
            if !path.exists() || !seen.insert(path.clone()) {
                continue;
            }
            entries.push(Candidate { category, path });
        }
    }

    Ok(entries)
}

fn size_entries(candidates: &[Candidate]) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let size = path_size_bytes(&candidate.path)?;
        entries.push(Entry {
            category: candidate.category,
            path: candidate.path.clone(),
            size,
        });
    }
    entries.sort_by(|left, right| {
        right
            .size
            .cmp(&left.size)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(entries)
}

fn path_size_bytes(path: &Path) -> Result<u64, String> {
    let output = run_output(
        DU_BIN,
        &[
            OsString::from("-s"),
            OsString::from("--block-size=1"),
            path.as_os_str().to_os_string(),
        ],
    )?;
    let first = output
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("unable to parse du output for {}", path.display()))?;
    first
        .parse::<u64>()
        .map_err(|err| format!("invalid du size for {}: {err}", path.display()))
}

fn format_bytes(bytes: u64) -> String {
    format!("{:6.1}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
}

fn display_path(home: &Path, path: &Path) -> String {
    if let Ok(stripped) = path.strip_prefix(home) {
        if stripped.as_os_str().is_empty() {
            "~".to_string()
        } else {
            format!("~/{}", stripped.display())
        }
    } else {
        path.display().to_string()
    }
}

fn print_report(home: &Path, entries: &[Entry], limit: usize) {
    let mut totals: BTreeMap<Category, (u64, usize)> = BTreeMap::new();
    let mut grand_total = 0u64;

    for entry in entries {
        grand_total += entry.size;
        let item = totals.entry(entry.category).or_insert((0, 0));
        item.0 += entry.size;
        item.1 += 1;
    }

    println!("Category totals");
    for (category, (bytes, count)) in totals.iter().rev() {
        println!(
            "{}  {:<6} ({:>2} paths)",
            format_bytes(*bytes),
            category.label(),
            count
        );
    }

    println!();
    println!("Top paths");
    for entry in entries.iter().take(limit) {
        println!(
            "{}  {:<6}  {}",
            format_bytes(entry.size),
            entry.category.label(),
            display_path(home, &entry.path)
        );
    }

    println!();
    println!(
        "Grand total: {} across {} matched paths",
        format_bytes(grand_total),
        entries.len()
    );
}

fn safe_to_remove(home: &Path, entry: &Candidate) -> bool {
    if !entry.path.starts_with(home) || entry.path == home {
        return false;
    }
    if entry.path.starts_with(home.join(".local/bin"))
        || entry.path.starts_with(home.join(".cargo/bin"))
    {
        return false;
    }

    let exact = [
        home.join(".cargo/registry/cache"),
        home.join(".cargo/git/db"),
        home.join(".cache/pip"),
        home.join(".cache/uv"),
        home.join(".cache/rattler/cache"),
        home.join(".cache/pre-commit"),
        home.join(".local/share/hatch"),
        home.join(".npm"),
    ];
    if exact.iter().any(|candidate| candidate == &entry.path) {
        return true;
    }

    let Some(name) = entry.path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    match entry.category {
        Category::Rust => {
            name == "target" && has_cargo_parent(&entry.path) && looks_like_rust_target(&entry.path)
        }
        Category::Python => matches!(
            name,
            ".pytest_cache" | ".mypy_cache" | ".ruff_cache" | ".hypothesis"
        ),
        Category::Pixi => entry.path.ends_with(Path::new(".cache/rattler/cache")),
        Category::Tox => name == ".tox",
        Category::Venv => name == ".venv",
        Category::Js => {
            name == ".cache"
                && entry
                    .path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|value| value.to_str())
                    == Some("node_modules")
        }
    }
}

fn clean_entries(home: &Path, entries: &[Candidate], yes: bool) -> Result<(), String> {
    if !yes {
        return Err("refusing to clean without --yes".to_string());
    }

    for entry in entries {
        if !safe_to_remove(home, entry) {
            return Err(format!(
                "refusing to remove unexpected path: {}",
                entry.path.display()
            ));
        }
    }

    for entry in entries {
        println!(
            "removing {:<6}  {}",
            entry.category.label(),
            display_path(home, &entry.path)
        );
        fs::remove_dir_all(&entry.path)
            .map_err(|err| format!("failed to remove {}: {err}", entry.path.display()))?;
    }

    Ok(())
}

fn main() {
    let options = match parse_args() {
        Ok(options) => options,
        Err(err) if err == usage() => {
            println!("{}", usage());
            return;
        }
        Err(err) => {
            eprintln!("{err}");
            eprintln!();
            eprintln!("{}", usage());
            process::exit(64);
        }
    };

    let result = home_dir()
        .and_then(|home| {
            collect_candidates(&home, &options.categories).map(|entries| (home, entries))
        })
        .and_then(|(home, entries)| {
            match options.mode {
                Mode::Report => print_report(&home, &size_entries(&entries)?, options.limit),
                Mode::Clean => {
                    if options.dry_run {
                        let sized = size_entries(&entries)?;
                        print_report(&home, &sized, options.limit);
                        println!();
                        println!("Dry run");
                        for entry in &sized {
                            println!(
                                "would remove {}  {:<6}  {}",
                                format_bytes(entry.size),
                                entry.category.label(),
                                display_path(&home, &entry.path)
                            );
                        }
                    } else {
                        clean_entries(&home, &entries, options.yes)?;
                    }
                }
                Mode::AutoClean => {
                    let free = free_bytes_for(&home)?;
                    let threshold = options.min_free_gb.saturating_mul(1024 * 1024 * 1024);
                    if free >= threshold {
                        println!(
                            "free={} on {}, above threshold={} GB; no-op",
                            format_bytes(free),
                            home.display(),
                            options.min_free_gb
                        );
                        return Ok(());
                    }
                    println!(
                        "free={} below threshold={} GB; cleaning",
                        format_bytes(free),
                        options.min_free_gb
                    );
                    clean_entries(&home, &entries, true)?;
                    let after = free_bytes_for(&home)?;
                    println!(
                        "post-clean free={} ({} reclaimed)",
                        format_bytes(after),
                        format_bytes(after.saturating_sub(free))
                    );
                    // Still tight? Suggest the snapshot route so the user
                    // does not have to remember it.
                    if after < threshold && Path::new("/.snapshots").is_dir() {
                        println!();
                        println!(
                            "still below threshold. btrfs snapshots likely hold reclaimable space."
                        );
                        println!(
                            "run: rg-space-sweep snapshots, then run the printed sudo command"
                        );
                    }
                }
                Mode::Snapshots => {
                    write_snapshot_script(&options)?;
                }
            }
            Ok(())
        });

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn default_snapshot_script_path_uses_user_scoped_runtime_dir() {
        let path = default_snapshot_script_path_for(Some(Path::new("/run/user/1001")), 1001);

        assert_eq!(
            path,
            PathBuf::from("/run/user/1001/rg-space-sweep/btrfs-snapshot-cleanup.sh")
        );
    }

    #[test]
    fn default_snapshot_script_path_falls_back_to_user_scoped_tmp_dir() {
        let path = default_snapshot_script_path_for(None, 1001);

        assert_eq!(
            path,
            PathBuf::from("/tmp/rg-space-sweep-1001/btrfs-snapshot-cleanup.sh")
        );
    }
}
