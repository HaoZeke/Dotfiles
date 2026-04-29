use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

use serde::Deserialize;

const FIND_BIN: &str = "/usr/bin/find";
const DU_BIN: &str = "/usr/bin/du";

/// First-pass btrfs snapshot cleanup: keeps the newest dated snapshot of
/// each prefix (@ and @home) as a safety net.
const BTRFS_CLEANUP_SCRIPT: &str = include_str!("btrfs-cleanup.sh");

/// Aggressive variant: removes every dated @/@home snapshot and runs a full
/// `-dusage=100 -musage=100` balance so snapshot-held blocks are released.
const BTRFS_CLEANUP_SCRIPT_AGGRESSIVE: &str = r#"#!/usr/bin/env bash
# btrfs-snapshot-cleanup-aggressive.sh
# Removes every dated @/@home snapshot and runs full balance.
# Run with: sudo bash <this-script>
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "must run as root (use sudo)" >&2
  exit 1
fi

SNAP_DIR=/.snapshots

echo "=== before ==="
df -h /home / 2>/dev/null | awk 'NR==1 || /\/(home)?$/'
echo
echo "=== btrfs subvolume snapshots ==="
btrfs subvolume list -s / || true
echo
echo "=== btrfs filesystem usage before ==="
btrfs filesystem usage -T / || true
echo

mapfile -t snaps < <(
  find "$SNAP_DIR" -mindepth 1 -maxdepth 1 -type d \
    -regextype posix-extended \
    -regex ".*/@(home)?\.[0-9]+T[0-9]+$" -printf '%p\n' 2>/dev/null | sort
)

echo "=== snapshot deletion plan ==="
if ((${#snaps[@]} == 0)); then
  echo "no dated snapshots found under $SNAP_DIR"
else
  echo "aggressive mode will delete ${#snaps[@]} dated snapshot(s):"
  for s in "${snaps[@]}"; do
    echo "  $s"
  done
  echo
  for s in "${snaps[@]}"; do
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
echo
echo "=== btrfs filesystem usage after ==="
btrfs filesystem usage -T / || true
echo
echo "btrfs fi usage /home:"
btrfs fi usage /home | head -20 || true
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum Target {
    Local,
    Remote(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum RemoteRunner {
    Ssh,
    Gsocket,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RemoteProfile {
    name: String,
    host: String,
    home: PathBuf,
    runner: RemoteRunner,
    min_free_gb: u64,
    snapshots: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RemoteInvocation {
    program: String,
    args: Vec<String>,
    stdin: String,
}

#[derive(Clone, Debug)]
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
    /// Where to execute report/clean operations.
    target: Target,
    /// Optional TOML file containing remote target profiles.
    target_config_path: Option<PathBuf>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SnapshotDeletePlan {
    prefix: &'static str,
    found: usize,
    standard_deletes: usize,
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
usage: rg-space-sweep [--target local|NAME] [--target-config PATH] [report|clean|auto-clean|snapshots] [--dry-run] [--yes] [--aggressive] [--limit N] [--min-free-gb N] [--script-path PATH] [default|all|rust|python|pixi|tox|venv|js]

report
    Show category totals and the largest matching cache/build directories.

clean
    Remove the matching directories. Requires --yes, or use --dry-run to preview.
    Remote targets support report and clean through SSH without needing this
    binary installed remotely.

auto-clean
    Check free space on $HOME; clean only if below --min-free-gb (default 10).
    Always implies --yes when firing. Intended for a systemd timer.

snapshots
    Report dated btrfs snapshots under /.snapshots and write a root-only
    cleanup script (keeps newest @ and @home, deletes older pairs, runs
    balance). The default path is under the user-scoped runtime directory.
    Override via --script-path. Use --aggressive to delete every dated
    @/@home snapshot. Run the script with `sudo bash <path>`.

default
    rust python tox

all
    default + pixi + venv + js"
}

fn parse_args() -> Result<Options, String> {
    parse_args_from(env::args().skip(1))
}

fn parse_args_from<I, S>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut mode = Mode::Report;
    let mut mode_seen = false;
    let mut dry_run = false;
    let mut yes = false;
    let mut limit = 20usize;
    let mut min_free_gb: u64 = 10;
    let mut script_path = default_snapshot_script_path();
    let mut aggressive = false;
    let mut target = Target::Local;
    let mut target_config_path = None;
    let mut category_tokens = Vec::new();

    let mut args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "report" if !mode_seen => {
                mode = Mode::Report;
                mode_seen = true;
            }
            "clean" if !mode_seen => {
                mode = Mode::Clean;
                mode_seen = true;
            }
            "auto-clean" if !mode_seen => {
                mode = Mode::AutoClean;
                mode_seen = true;
            }
            "snapshots" if !mode_seen => {
                mode = Mode::Snapshots;
                mode_seen = true;
            }
            "--dry-run" => dry_run = true,
            "--yes" => yes = true,
            "--aggressive" => aggressive = true,
            "--target" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--target requires a value".to_string())?;
                target = parse_target(&value)?;
            }
            "--target-config" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--target-config requires a value".to_string())?;
                target_config_path = Some(PathBuf::from(value));
            }
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
            other if other.starts_with('-') => return Err(format!("unknown option: {other}")),
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

    let options = Options {
        mode,
        categories,
        dry_run,
        yes,
        limit,
        min_free_gb,
        script_path,
        aggressive,
        target,
        target_config_path,
    };
    validate_target_mode(&options)?;
    Ok(options)
}

fn parse_target(value: &str) -> Result<Target, String> {
    if value.trim().is_empty() {
        return Err("--target requires a non-empty value".to_string());
    }
    if value == "local" {
        Ok(Target::Local)
    } else {
        Ok(Target::Remote(value.to_string()))
    }
}

fn validate_target_mode(options: &Options) -> Result<(), String> {
    if matches!(options.target, Target::Remote(_))
        && matches!(options.mode, Mode::AutoClean | Mode::Snapshots)
    {
        return Err(
            "remote targets support report and clean; auto-clean and snapshots are local-only"
                .to_string(),
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct RemoteProfilesFile {
    #[serde(default)]
    targets: BTreeMap<String, RemoteProfileToml>,
}

#[derive(Debug, Deserialize)]
struct RemoteProfileToml {
    host: String,
    home: PathBuf,
    #[serde(default = "default_remote_runner")]
    runner: RemoteRunner,
    #[serde(default = "default_remote_min_free_gb")]
    min_free_gb: u64,
    #[serde(default)]
    snapshots: bool,
}

fn default_remote_runner() -> RemoteRunner {
    RemoteRunner::Ssh
}

fn default_remote_min_free_gb() -> u64 {
    10
}

fn builtin_remote_profiles() -> BTreeMap<String, RemoteProfile> {
    let mut profiles = BTreeMap::new();
    let cosmolab = RemoteProfile {
        name: "cosmolab".to_string(),
        host: "rg.cosmolab".to_string(),
        home: PathBuf::from("/home/goswami"),
        runner: RemoteRunner::Ssh,
        min_free_gb: 200,
        snapshots: false,
    };
    profiles.insert(cosmolab.name.clone(), cosmolab.clone());
    profiles.insert(
        "rg.cosmolab".to_string(),
        RemoteProfile {
            name: "rg.cosmolab".to_string(),
            ..cosmolab
        },
    );
    profiles
}

fn remote_profiles_from_toml(input: &str) -> Result<BTreeMap<String, RemoteProfile>, String> {
    let parsed: RemoteProfilesFile =
        toml::from_str(input).map_err(|err| format!("parse target config: {err}"))?;
    let mut profiles = BTreeMap::new();

    for (name, profile) in parsed.targets {
        if name.trim().is_empty() {
            return Err("target profile name cannot be empty".to_string());
        }
        if profile.host.trim().is_empty() {
            return Err(format!("target {name} has an empty host"));
        }
        if !profile.home.is_absolute() {
            return Err(format!(
                "target {name} home must be absolute: {}",
                profile.home.display()
            ));
        }
        profiles.insert(
            name.clone(),
            RemoteProfile {
                name,
                host: profile.host,
                home: profile.home,
                runner: profile.runner,
                min_free_gb: profile.min_free_gb,
                snapshots: profile.snapshots,
            },
        );
    }

    Ok(profiles)
}

fn default_target_config_path() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from) {
        if config_home.is_absolute() {
            return Some(config_home.join("rg-space-sweep/targets.toml"));
        }
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/rg-space-sweep/targets.toml"))
}

fn load_remote_profiles(
    config_path: Option<&Path>,
) -> Result<BTreeMap<String, RemoteProfile>, String> {
    let mut profiles = builtin_remote_profiles();
    let (path, explicit) = match config_path {
        Some(path) => (Some(path.to_path_buf()), true),
        None => (default_target_config_path(), false),
    };

    let Some(path) = path else {
        return Ok(profiles);
    };
    if !path.exists() {
        if explicit {
            return Err(format!("target config does not exist: {}", path.display()));
        }
        return Ok(profiles);
    }

    let input =
        fs::read_to_string(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
    for (name, profile) in remote_profiles_from_toml(&input)? {
        profiles.insert(name, profile);
    }
    Ok(profiles)
}

fn remote_profile_for(
    target_name: &str,
    config_path: Option<&Path>,
) -> Result<RemoteProfile, String> {
    load_remote_profiles(config_path)?
        .remove(target_name)
        .ok_or_else(|| format!("unknown remote target: {target_name}"))
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

fn snapshot_prefix_for_path(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_str()?;
    if name.starts_with("@home.") {
        Some("@home")
    } else if name.starts_with("@.") {
        Some("@")
    } else {
        None
    }
}

fn snapshot_delete_plan(snaps: &[PathBuf]) -> Vec<SnapshotDeletePlan> {
    ["@", "@home"]
        .into_iter()
        .map(|prefix| {
            let found = snaps
                .iter()
                .filter(|path| snapshot_prefix_for_path(path) == Some(prefix))
                .count();
            SnapshotDeletePlan {
                prefix,
                found,
                standard_deletes: found.saturating_sub(1),
            }
        })
        .collect()
}

fn standard_snapshot_delete_total(snaps: &[PathBuf]) -> usize {
    snapshot_delete_plan(snaps)
        .iter()
        .map(|plan| plan.standard_deletes)
        .sum()
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
    let plan = snapshot_delete_plan(&snaps);
    let standard_deletes = standard_snapshot_delete_total(&snaps);
    println!();
    println!("standard deletion plan:");
    for item in &plan {
        println!(
            "  {}: {} found, {} deleted by standard mode, {} kept",
            item.prefix,
            item.found,
            item.standard_deletes,
            item.found.saturating_sub(item.standard_deletes)
        );
    }
    if options.aggressive {
        println!();
        println!(
            "aggressive mode will delete all {} dated snapshot(s).",
            snaps.len()
        );
    } else if standard_deletes == 0 {
        println!();
        println!("standard mode will delete 0 snapshots for this set.");
        println!("generate an aggressive script instead:");
        println!(
            "  rg-space-sweep snapshots --aggressive --script-path {}",
            options.script_path.display()
        );
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
        "aggressive (removes every dated @/@home snapshot)"
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
        println!("if space is still tight after the first pass, generate an aggressive script:");
        println!(
            "  rg-space-sweep snapshots --aggressive --script-path {}",
            options.script_path.display()
        );
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn push_shell_var(script: &mut String, name: &str, value: &str) {
    script.push_str(name);
    script.push('=');
    script.push_str(&shell_quote(value));
    script.push('\n');
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Report => "report",
        Mode::Clean => "clean",
        Mode::AutoClean => "auto-clean",
        Mode::Snapshots => "snapshots",
    }
}

fn remote_script_for(profile: &RemoteProfile, options: &Options) -> Result<String, String> {
    if matches!(options.mode, Mode::AutoClean | Mode::Snapshots) {
        return Err("remote script supports report and clean only".to_string());
    }

    let categories = options
        .categories
        .iter()
        .map(|category| category.label())
        .collect::<Vec<_>>()
        .join(" ");
    let home = profile.home.to_string_lossy();
    let mut script = String::new();

    script.push_str("#!/usr/bin/env bash\nset -euo pipefail\n\n");
    push_shell_var(&mut script, "HOME_DIR", home.as_ref());
    push_shell_var(&mut script, "MODE", mode_label(options.mode));
    push_shell_var(&mut script, "CATEGORIES", &categories);
    push_shell_var(&mut script, "LIMIT", &options.limit.to_string());
    push_shell_var(
        &mut script,
        "DRY_RUN",
        if options.dry_run { "1" } else { "0" },
    );
    if options.yes {
        push_shell_var(&mut script, "YES", "1");
    }
    push_shell_var(&mut script, "MIN_FREE_GB", &profile.min_free_gb.to_string());

    script.push_str(
        r#"
CANDIDATES="$(mktemp)"
SIZES="$(mktemp)"
SORTED="$(mktemp)"
trap 'rm -f "$CANDIDATES" "$SIZES" "$SORTED"' EXIT

format_bytes() {
  awk -v bytes="$1" 'BEGIN { printf "%6.1fG", bytes / 1024 / 1024 / 1024 }'
}

display_path() {
  local path="$1"
  if [[ "$path" == "$HOME_DIR" ]]; then
    printf '~\n'
  elif [[ "$path" == "$HOME_DIR/"* ]]; then
    printf '~/%s\n' "${path#"$HOME_DIR"/}"
  else
    printf '%s\n' "$path"
  fi
}

add_candidate() {
  local category="$1"
  local path="$2"
  [[ -d "$path" ]] || return 0
  printf '%s\t%s\n' "$category" "$path" >> "$CANDIDATES"
}

find_dirs() {
  find "$HOME_DIR" -xdev \
    \( -path "$HOME_DIR/.cache" \
    -o -path "$HOME_DIR/.cargo" \
    -o -path "$HOME_DIR/.local/share/containers" \
    -o -path "$HOME_DIR/.local/share/Trash" \
    -o -name .git \
    -o -name .direnv \
    -o -name .pixi \
    -o -name .nox \
    -o -name __pycache__ \) -prune \
    -o -type d "$@" -prune -print 2>/dev/null || true
}

find_named() {
  find_dirs -name "$1"
}

find_path() {
  find_dirs -path "$1"
}

find_python_caches() {
  find "$HOME_DIR" -xdev \
    \( -path "$HOME_DIR/.cache" \
    -o -path "$HOME_DIR/.cargo" \
    -o -path "$HOME_DIR/.local/share/containers" \
    -o -path "$HOME_DIR/.local/share/Trash" \
    -o -name .git \
    -o -name .direnv \
    -o -name .pixi \
    -o -name .nox \
    -o -name __pycache__ \
    -o -name target \
    -o -name node_modules \
    -o -name .venv \
    -o -name .tox \) -prune \
    -o -type d \( -name .pytest_cache \
    -o -name .mypy_cache \
    -o -name .ruff_cache \
    -o -name .hypothesis \) -prune -print 2>/dev/null || true
}

has_cargo_parent() {
  local path="$1"
  local parent
  parent="$(dirname "$path")"
  [[ -f "$parent/Cargo.toml" || -f "$parent/Cargo.lock" ]]
}

looks_like_rust_target() {
  local path="$1"
  [[ -d "$path/debug" || -d "$path/release" || -d "$path/.fingerprint" || -e "$path/.rustc_info.json" ]]
}

collect_category() {
  local category="$1"
  local path
  case "$category" in
    rust)
      while IFS= read -r path; do
        if has_cargo_parent "$path" && looks_like_rust_target "$path"; then
          add_candidate rust "$path"
        fi
      done < <(find_named target)
      add_candidate rust "$HOME_DIR/.cargo/registry/cache"
      add_candidate rust "$HOME_DIR/.cargo/git/db"
      ;;
    python)
      while IFS= read -r path; do
        add_candidate python "$path"
      done < <(find_python_caches)
      add_candidate python "$HOME_DIR/.cache/pip"
      add_candidate python "$HOME_DIR/.cache/uv"
      add_candidate python "$HOME_DIR/.cache/pre-commit"
      add_candidate python "$HOME_DIR/.local/share/hatch"
      ;;
    pixi)
      add_candidate pixi "$HOME_DIR/.cache/rattler/cache"
      ;;
    tox)
      while IFS= read -r path; do
        add_candidate tox "$path"
      done < <(find_named .tox)
      ;;
    venv)
      while IFS= read -r path; do
        add_candidate venv "$path"
      done < <(find_named .venv)
      ;;
    js)
      while IFS= read -r path; do
        add_candidate js "$path"
      done < <(find_path "*/node_modules/.cache")
      add_candidate js "$HOME_DIR/.npm"
      ;;
    *)
      echo "unknown category in remote script: $category" >&2
      exit 64
      ;;
  esac
}

collect_candidates() {
  local category
  for category in $CATEGORIES; do
    collect_category "$category"
  done
  sort -u "$CANDIDATES" -o "$CANDIDATES"
}

size_candidates() {
  local category path size
  : > "$SIZES"
  while IFS=$'\t' read -r category path; do
    [[ -n "${category:-}" && -n "${path:-}" ]] || continue
    size="$(du -s --block-size=1 "$path" 2>/dev/null | awk '{print $1}')" || continue
    [[ -n "$size" ]] || continue
    printf '%s\t%s\t%s\n' "$size" "$category" "$path" >> "$SIZES"
  done < "$CANDIDATES"
  sort -rn "$SIZES" > "$SORTED"
}

print_report() {
  local category total count shown size label path display total_all count_all
  echo "Category totals"
  for category in $CATEGORIES; do
    read -r total count < <(awk -F '\t' -v cat="$category" '$2 == cat { total += $1; count += 1 } END { printf "%s %s\n", total + 0, count + 0 }' "$SORTED")
    if [[ "$count" != "0" ]]; then
      label="$(format_bytes "$total")"
      printf '%s  %-6s (%2d paths)\n' "$label" "$category" "$count"
    fi
  done

  echo
  echo "Top paths"
  shown=0
  while IFS=$'\t' read -r size category path; do
    [[ -n "${size:-}" ]] || continue
    label="$(format_bytes "$size")"
    display="$(display_path "$path")"
    printf '%s  %-6s  %s\n' "$label" "$category" "$display"
    shown=$((shown + 1))
    [[ "$shown" -ge "$LIMIT" ]] && break
  done < "$SORTED"

  read -r total_all count_all < <(awk -F '\t' '{ total += $1; count += 1 } END { printf "%s %s\n", total + 0, count + 0 }' "$SORTED")
  echo
  printf 'Grand total: %s across %d matched paths\n' "$(format_bytes "$total_all")" "$count_all"
}

safe_to_remove() {
  local category="$1"
  local path="$2"
  local name parent
  [[ "$path" == "$HOME_DIR/"* ]] || return 1
  [[ "$path" != "$HOME_DIR" ]] || return 1
  case "$path" in
    "$HOME_DIR/.local/bin"|"$HOME_DIR/.local/bin/"*|"$HOME_DIR/.cargo/bin"|"$HOME_DIR/.cargo/bin/"*)
      return 1
      ;;
    "$HOME_DIR/.cargo/registry/cache"|"$HOME_DIR/.cargo/git/db"|"$HOME_DIR/.cache/pip"|"$HOME_DIR/.cache/uv"|"$HOME_DIR/.cache/rattler/cache"|"$HOME_DIR/.cache/pre-commit"|"$HOME_DIR/.local/share/hatch"|"$HOME_DIR/.npm")
      return 0
      ;;
  esac

  name="${path##*/}"
  case "$category" in
    rust)
      [[ "$name" == "target" ]] && has_cargo_parent "$path" && looks_like_rust_target "$path"
      ;;
    python)
      [[ "$name" == ".pytest_cache" || "$name" == ".mypy_cache" || "$name" == ".ruff_cache" || "$name" == ".hypothesis" ]]
      ;;
    pixi)
      [[ "$path" == "$HOME_DIR/.cache/rattler/cache" ]]
      ;;
    tox)
      [[ "$name" == ".tox" ]]
      ;;
    venv)
      [[ "$name" == ".venv" ]]
      ;;
    js)
      parent="$(basename "$(dirname "$path")")"
      [[ "$name" == ".cache" && "$parent" == "node_modules" ]]
      ;;
    *)
      return 1
      ;;
  esac
}

clean_paths() {
  local size category path label display
  if [[ "${YES:-0}" != "1" ]]; then
    echo "refusing to clean without --yes; use --dry-run to preview first" >&2
    exit 64
  fi

  while IFS=$'\t' read -r size category path; do
    [[ -n "${path:-}" ]] || continue
    if ! safe_to_remove "$category" "$path"; then
      echo "refusing to remove unexpected path: $path" >&2
      exit 65
    fi
  done < "$SORTED"

  while IFS=$'\t' read -r size category path; do
    [[ -n "${path:-}" ]] || continue
    label="$(format_bytes "$size")"
    display="$(display_path "$path")"
    printf 'removing %s  %-6s  %s\n' "$label" "$category" "$display"
    rm -rf -- "$path"
  done < "$SORTED"
}

collect_candidates
size_candidates

case "$MODE" in
  report)
    print_report
    ;;
  clean)
    if [[ "$DRY_RUN" == "1" ]]; then
      print_report
      echo
      echo "Dry run"
      while IFS=$'\t' read -r size category path; do
        [[ -n "${path:-}" ]] || continue
        printf 'would remove %s  %-6s  %s\n' "$(format_bytes "$size")" "$category" "$(display_path "$path")"
      done < "$SORTED"
    else
      clean_paths
    fi
    ;;
  *)
    echo "unsupported remote mode: $MODE" >&2
    exit 64
    ;;
esac
"#,
    );

    Ok(script)
}

fn remote_invocation_for(
    profile: &RemoteProfile,
    options: &Options,
) -> Result<RemoteInvocation, String> {
    validate_target_mode(options)?;
    match profile.runner {
        RemoteRunner::Ssh => Ok(RemoteInvocation {
            program: "ssh".to_string(),
            args: vec![
                "-o".to_string(),
                "BatchMode=yes".to_string(),
                "-o".to_string(),
                "ConnectTimeout=10".to_string(),
                profile.host.clone(),
                "bash".to_string(),
                "-s".to_string(),
            ],
            stdin: remote_script_for(profile, options)?,
        }),
        RemoteRunner::Gsocket => Err(
            "gsocket remote runner is not supported by rg-space-sweep; use runner = \"ssh\""
                .to_string(),
        ),
    }
}

fn run_remote_invocation(invocation: &RemoteInvocation) -> Result<(), String> {
    let mut child = Command::new(&invocation.program)
        .args(&invocation.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| format!("failed to run {}: {err}", invocation.program))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "failed to open remote command stdin".to_string())?;
        stdin
            .write_all(invocation.stdin.as_bytes())
            .map_err(|err| format!("write remote script: {err}"))?;
    }

    let status = child
        .wait()
        .map_err(|err| format!("wait for remote command: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("remote command exited with status {status}"))
    }
}

fn run_remote_mode(target_name: &str, options: &Options) -> Result<(), String> {
    let profile = remote_profile_for(target_name, options.target_config_path.as_deref())?;
    let invocation = remote_invocation_for(&profile, options)?;
    run_remote_invocation(&invocation)
}

fn mode_requires_candidate_scan(mode: Mode) -> bool {
    !matches!(mode, Mode::Snapshots)
}

fn run_local_mode(options: &Options) -> Result<(), String> {
    if !mode_requires_candidate_scan(options.mode) {
        return write_snapshot_script(options);
    }

    home_dir()
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
                        println!(
                            "if standard mode reports 0 deletes, run: rg-space-sweep snapshots --aggressive"
                        );
                    }
                }
                Mode::Snapshots => {
                    write_snapshot_script(options)?;
                }
            }
            Ok(())
        })
}

fn run_options(options: &Options) -> Result<(), String> {
    validate_target_mode(options)?;
    match &options.target {
        Target::Local => run_local_mode(options),
        Target::Remote(target_name) => run_remote_mode(target_name, options),
    }
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

    if let Err(err) = run_options(&options) {
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

    #[test]
    fn standard_snapshot_plan_keeps_single_root_and_home_snapshots() {
        let snaps = vec![
            PathBuf::from("/.snapshots/@.20260429T0000"),
            PathBuf::from("/.snapshots/@home.20260429T0000"),
        ];

        assert_eq!(standard_snapshot_delete_total(&snaps), 0);
        assert_eq!(
            snapshot_delete_plan(&snaps),
            vec![
                SnapshotDeletePlan {
                    prefix: "@",
                    found: 1,
                    standard_deletes: 0,
                },
                SnapshotDeletePlan {
                    prefix: "@home",
                    found: 1,
                    standard_deletes: 0,
                },
            ]
        );
    }

    #[test]
    fn standard_snapshot_plan_deletes_older_root_and_home_pairs() {
        let snaps = vec![
            PathBuf::from("/.snapshots/@.20260428T0000"),
            PathBuf::from("/.snapshots/@.20260429T0000"),
            PathBuf::from("/.snapshots/@home.20260428T0000"),
            PathBuf::from("/.snapshots/@home.20260429T0000"),
        ];

        assert_eq!(standard_snapshot_delete_total(&snaps), 2);
    }

    #[test]
    fn aggressive_snapshot_script_reports_btrfs_state_before_deleting() {
        assert!(usage().contains("--aggressive"));
        assert!(BTRFS_CLEANUP_SCRIPT_AGGRESSIVE.contains("=== snapshot deletion plan ==="));
        assert!(BTRFS_CLEANUP_SCRIPT_AGGRESSIVE.contains("btrfs subvolume list -s /"));
        assert!(BTRFS_CLEANUP_SCRIPT_AGGRESSIVE.contains("btrfs filesystem usage -T /"));
    }

    #[test]
    fn snapshots_mode_does_not_need_cache_candidate_scan() {
        assert!(!mode_requires_candidate_scan(Mode::Snapshots));
        assert!(mode_requires_candidate_scan(Mode::Report));
        assert!(mode_requires_candidate_scan(Mode::Clean));
        assert!(mode_requires_candidate_scan(Mode::AutoClean));
    }

    #[test]
    fn parse_args_from_defaults_to_local_target() {
        let options = parse_args_from(["report"]).expect("parse args");

        assert_eq!(options.target, Target::Local);
    }

    #[test]
    fn parse_args_from_accepts_remote_target_before_mode() {
        let options =
            parse_args_from(["--target", "cosmolab", "report", "all"]).expect("parse args");

        assert_eq!(options.target, Target::Remote("cosmolab".to_string()));
        assert_eq!(
            options.categories,
            vec![
                Category::Rust,
                Category::Python,
                Category::Pixi,
                Category::Tox,
                Category::Venv,
                Category::Js,
            ]
        );
    }

    #[test]
    fn parse_args_from_accepts_target_config_path() {
        let options = parse_args_from([
            "report",
            "--target",
            "labbox",
            "--target-config",
            "/tmp/targets.toml",
        ])
        .expect("parse args");

        assert_eq!(options.target, Target::Remote("labbox".to_string()));
        assert_eq!(
            options.target_config_path,
            Some(PathBuf::from("/tmp/targets.toml"))
        );
    }

    #[test]
    fn remote_target_rejects_snapshots_and_auto_clean() {
        let snapshots = Options {
            mode: Mode::Snapshots,
            categories: vec![Category::Rust],
            dry_run: false,
            yes: false,
            limit: 20,
            min_free_gb: 10,
            script_path: PathBuf::from("/tmp/script.sh"),
            aggressive: false,
            target: Target::Remote("cosmolab".to_string()),
            target_config_path: None,
        };
        let auto_clean = Options {
            mode: Mode::AutoClean,
            ..snapshots.clone()
        };

        assert!(validate_target_mode(&snapshots).is_err());
        assert!(validate_target_mode(&auto_clean).is_err());
    }

    #[test]
    fn remote_profiles_load_from_toml_and_keep_cosmolab_builtin() {
        let profiles = remote_profiles_from_toml(
            r#"
            [targets.labbox]
            host = "labbox.example.org"
            home = "/home/labuser"
            runner = "ssh"
            min_free_gb = 75
            snapshots = false
            "#,
        )
        .expect("parse profiles");

        assert_eq!(
            profiles.get("labbox"),
            Some(&RemoteProfile {
                name: "labbox".to_string(),
                host: "labbox.example.org".to_string(),
                home: PathBuf::from("/home/labuser"),
                runner: RemoteRunner::Ssh,
                min_free_gb: 75,
                snapshots: false,
            })
        );
        assert_eq!(builtin_remote_profiles()["cosmolab"].host, "rg.cosmolab");
    }

    #[test]
    fn remote_invocation_uses_ssh_and_generated_script() {
        let profile = builtin_remote_profiles()["cosmolab"].clone();
        let options = parse_args_from(["report", "--target", "cosmolab", "--limit", "8", "all"])
            .expect("parse args");

        let invocation = remote_invocation_for(&profile, &options).expect("remote invocation");

        assert_eq!(invocation.program, "ssh");
        assert_eq!(invocation.args[0], "-o");
        assert!(invocation.args.contains(&"BatchMode=yes".to_string()));
        assert!(invocation.args.contains(&"rg.cosmolab".to_string()));
        assert!(invocation.stdin.contains("HOME_DIR='/home/goswami'"));
        assert!(invocation.stdin.contains("MODE='report'"));
        assert!(invocation.stdin.contains("LIMIT='8'"));
        assert!(!invocation.stdin.contains("rg-space-sweep"));
    }

    #[test]
    fn remote_clean_invocation_preserves_yes_guard() {
        let profile = builtin_remote_profiles()["cosmolab"].clone();
        let dry_run = parse_args_from(["clean", "--target", "cosmolab", "--dry-run", "python"])
            .expect("parse dry-run");
        let clean = parse_args_from(["clean", "--target", "cosmolab", "--yes", "python"])
            .expect("parse clean");

        let dry_invocation = remote_invocation_for(&profile, &dry_run).expect("remote dry-run");
        assert!(dry_invocation.stdin.contains("MODE='clean'"));
        assert!(dry_invocation.stdin.contains("DRY_RUN='1'"));
        assert!(!dry_invocation.stdin.contains("YES='1'"));

        let invocation = remote_invocation_for(&profile, &clean).expect("remote invocation");
        assert!(invocation.stdin.contains("MODE='clean'"));
        assert!(invocation.stdin.contains("YES='1'"));
        assert!(invocation.stdin.contains("rm -rf --"));
    }
}
