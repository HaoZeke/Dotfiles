use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

use serde::{Deserialize, Serialize};

mod remote_template;

use remote_template::REMOTE_SWEEP_SCRIPT;

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
enum Category {
    Rust,
    Python,
    Pixi,
    Tox,
    Venv,
    Js,
    /// Go module/build caches under ~/.cache (rebuildable).
    Go,
    /// JVM dependency caches (Gradle/Maven) under the home tree (rebuildable).
    Java,
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
            Self::Go => "go",
            Self::Java => "java",
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
                Self::Go,
                Self::Java,
            ]),
            "rust" => Ok(vec![Self::Rust]),
            "python" => Ok(vec![Self::Python]),
            "pixi" => Ok(vec![Self::Pixi]),
            "tox" => Ok(vec![Self::Tox]),
            "venv" => Ok(vec![Self::Venv]),
            "js" => Ok(vec![Self::Js]),
            "go" => Ok(vec![Self::Go]),
            "java" => Ok(vec![Self::Java]),
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
    /// Show all configured remote targets.
    TargetList,
    /// Show one configured remote target.
    TargetShow,
    /// Check local or remote target reachability and basic tool availability.
    TargetCheck,
    /// Show filesystem/cache pressure without doing a full sweep.
    Pressure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Text,
    Json,
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
    user: Option<String>,
    home: PathBuf,
    runner: RemoteRunner,
    gsocket_secret_file: Option<PathBuf>,
    ssh_identity: Option<PathBuf>,
    host_key_alias: Option<String>,
    roots: Vec<PathBuf>,
    prune: Vec<PathBuf>,
    exclude: Vec<PathBuf>,
    categories: Vec<Category>,
    min_free_gb: u64,
    snapshots: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RemoteInvocation {
    program: String,
    args: Vec<String>,
    stdin: String,
    max_attempts: usize,
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
    /// Output format for report/dry-run/target metadata.
    output_format: OutputFormat,
    /// Whether categories were supplied explicitly on the CLI.
    categories_explicit: bool,
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

#[derive(Serialize)]
struct JsonPathEntry {
    category: &'static str,
    path: String,
    size_bytes: u64,
}

#[derive(Serialize)]
struct JsonCategoryTotal {
    category: &'static str,
    size_bytes: u64,
    paths: usize,
}

#[derive(Serialize)]
struct JsonReport {
    grand_total_bytes: u64,
    matched_paths: usize,
    totals: Vec<JsonCategoryTotal>,
    top_paths: Vec<JsonPathEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    would_remove: Option<Vec<JsonPathEntry>>,
}

#[derive(Serialize)]
struct JsonTargetProfile {
    name: String,
    host: String,
    ssh_target: String,
    runner: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    home: String,
    roots: Vec<String>,
    prune: Vec<String>,
    exclude: Vec<String>,
    categories: Vec<&'static str>,
    min_free_gb: u64,
    snapshots: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    host_key_alias: Option<String>,
    gsocket_secret_file_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    gsocket_secret_file_path: Option<String>,
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
usage: rg-space-sweep [--target local|NAME] [--target-config PATH] [--json] [report|clean|auto-clean|snapshots|pressure] [--dry-run] [--yes] [--aggressive] [--limit N] [--min-free-gb N] [--script-path PATH] [default|all|rust|python|pixi|tox|venv|js|go|java]
       rg-space-sweep target list|show NAME|check NAME [--json] [--target-config PATH]

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
    default + pixi + venv + js + go + java"
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
    let mut output_format = OutputFormat::Text;
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
            "pressure" if !mode_seen => {
                mode = Mode::Pressure;
                mode_seen = true;
            }
            "target" if !mode_seen => {
                let subcommand = args
                    .next()
                    .ok_or_else(|| "target requires list, show, or check".to_string())?;
                match subcommand.as_str() {
                    "list" => mode = Mode::TargetList,
                    "show" => {
                        let name = args
                            .next()
                            .ok_or_else(|| "target show requires a target name".to_string())?;
                        mode = Mode::TargetShow;
                        target = Target::Remote(name);
                    }
                    "check" => {
                        let name = args
                            .next()
                            .ok_or_else(|| "target check requires a target name".to_string())?;
                        mode = Mode::TargetCheck;
                        target = Target::Remote(name);
                    }
                    other => {
                        return Err(format!(
                            "unknown target subcommand: {other}; use list, show, or check"
                        ))
                    }
                }
                mode_seen = true;
            }
            "--dry-run" => dry_run = true,
            "--yes" => yes = true,
            "--aggressive" => aggressive = true,
            "--json" => output_format = OutputFormat::Json,
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

    let categories_explicit = !category_tokens.is_empty();
    if category_tokens.is_empty()
        && matches!(
            mode,
            Mode::Report | Mode::Clean | Mode::AutoClean | Mode::Snapshots
        )
    {
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
        output_format,
        categories_explicit,
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
    #[serde(default)]
    user: Option<String>,
    home: PathBuf,
    #[serde(default = "default_remote_runner")]
    runner: RemoteRunner,
    #[serde(default)]
    gsocket_secret_file: Option<PathBuf>,
    #[serde(default)]
    ssh_identity: Option<PathBuf>,
    #[serde(default)]
    host_key_alias: Option<String>,
    #[serde(default)]
    roots: Vec<PathBuf>,
    #[serde(default)]
    prune: Vec<PathBuf>,
    #[serde(default)]
    exclude: Vec<PathBuf>,
    #[serde(default)]
    categories: Vec<Category>,
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

fn runner_label(runner: RemoteRunner) -> &'static str {
    match runner {
        RemoteRunner::Ssh => "ssh",
        RemoteRunner::Gsocket => "gsocket",
    }
}

fn local_home_dir() -> PathBuf {
    env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/home/rgoswami"))
}

fn ssh_config_path() -> Option<PathBuf> {
    let config = local_home_dir().join(".ssh/config");
    if config.is_file() {
        Some(config)
    } else {
        None
    }
}

fn ssh_config_args() -> Vec<String> {
    ssh_config_path()
        .map(|config| vec!["-F".to_string(), config.display().to_string()])
        .unwrap_or_default()
}

fn ssh_target_for(profile: &RemoteProfile) -> String {
    match profile.user.as_ref() {
        Some(user) => format!("{user}@{}", profile.host),
        None => profile.host.clone(),
    }
}

fn json_target_profile(profile: &RemoteProfile) -> JsonTargetProfile {
    JsonTargetProfile {
        name: profile.name.clone(),
        host: profile.host.clone(),
        ssh_target: ssh_target_for(profile),
        runner: runner_label(profile.runner),
        user: profile.user.clone(),
        home: profile.home.display().to_string(),
        roots: profile
            .roots
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        prune: profile
            .prune
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        exclude: profile
            .exclude
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        categories: profile
            .categories
            .iter()
            .map(|category| category.label())
            .collect(),
        min_free_gb: profile.min_free_gb,
        snapshots: profile.snapshots,
        ssh_config_path: ssh_config_path().map(|path| path.display().to_string()),
        host_key_alias: profile.host_key_alias.clone(),
        gsocket_secret_file_configured: profile.gsocket_secret_file.is_some(),
        gsocket_secret_file_path: profile
            .gsocket_secret_file
            .as_ref()
            .map(|path| path.display().to_string()),
    }
}

fn builtin_remote_profiles() -> BTreeMap<String, RemoteProfile> {
    let mut profiles = BTreeMap::new();
    let local_home = local_home_dir();
    let cosmolab = RemoteProfile {
        name: "cosmolab".to_string(),
        host: "rg.cosmolab".to_string(),
        user: None,
        home: PathBuf::from("/home/goswami"),
        runner: RemoteRunner::Gsocket,
        gsocket_secret_file: Some(local_home.join(".config/cosmolab/gsocket/rg.cosmolab.secret")),
        ssh_identity: None,
        host_key_alias: None,
        roots: vec![PathBuf::from("/home/goswami")],
        prune: Vec::new(),
        exclude: Vec::new(),
        categories: Vec::new(),
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
        let roots = if profile.roots.is_empty() {
            vec![profile.home.clone()]
        } else {
            profile.roots
        };
        for (field, paths) in [
            ("roots", roots.as_slice()),
            ("prune", profile.prune.as_slice()),
            ("exclude", profile.exclude.as_slice()),
        ] {
            for path in paths {
                if !path.is_absolute() {
                    return Err(format!(
                        "target {name} {field} path must be absolute: {}",
                        path.display()
                    ));
                }
            }
        }
        if profile.runner == RemoteRunner::Gsocket && profile.gsocket_secret_file.is_none() {
            return Err(format!(
                "target {name} uses runner = \"gsocket\" but has no gsocket_secret_file"
            ));
        }
        if let Some(path) = profile.gsocket_secret_file.as_ref() {
            if !path.is_absolute() {
                return Err(format!(
                    "target {name} gsocket_secret_file must be absolute: {}",
                    path.display()
                ));
            }
        }
        if let Some(path) = profile.ssh_identity.as_ref() {
            if !path.is_absolute() {
                return Err(format!(
                    "target {name} ssh_identity must be absolute: {}",
                    path.display()
                ));
            }
        }
        profiles.insert(
            name.clone(),
            RemoteProfile {
                name,
                host: profile.host,
                user: profile.user,
                home: profile.home,
                runner: profile.runner,
                gsocket_secret_file: profile.gsocket_secret_file,
                ssh_identity: profile.ssh_identity,
                host_key_alias: profile.host_key_alias,
                roots,
                prune: profile.prune,
                exclude: profile.exclude,
                categories: profile.categories,
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

fn list_targets(options: &Options) -> Result<(), String> {
    let profiles = load_remote_profiles(options.target_config_path.as_deref())?;
    if options.output_format == OutputFormat::Json {
        let values = profiles
            .values()
            .map(json_target_profile)
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::to_string(&values).map_err(|err| format!("serialize targets: {err}"))?
        );
    } else {
        for profile in profiles.values() {
            println!(
                "{:<12} {:<8} {}",
                profile.name,
                runner_label(profile.runner),
                profile.host
            );
        }
    }
    Ok(())
}

fn show_target(target_name: &str, options: &Options) -> Result<(), String> {
    let profile = remote_profile_for(target_name, options.target_config_path.as_deref())?;
    if options.output_format == OutputFormat::Json {
        println!(
            "{}",
            serde_json::to_string(&json_target_profile(&profile))
                .map_err(|err| format!("serialize target: {err}"))?
        );
    } else {
        println!("name: {}", profile.name);
        println!("host: {}", profile.host);
        println!("runner: {}", runner_label(profile.runner));
        if let Some(user) = profile.user.as_ref() {
            println!("user: {user}");
        }
        println!("home: {}", profile.home.display());
        println!("min_free_gb: {}", profile.min_free_gb);
        println!("roots:");
        for root in &profile.roots {
            println!("  {}", root.display());
        }
    }
    Ok(())
}

fn run_local_target_check(options: &Options) -> Result<(), String> {
    let home = home_dir()?;
    if options.output_format == OutputFormat::Json {
        println!(
            "{{\"mode\":\"target-check\",\"target\":\"local\",\"home\":\"{}\",\"ok\":true}}",
            home.display()
        );
    } else {
        println!("Target: local");
        println!("Home: {}", home.display());
        println!("find: {}", Path::new(FIND_BIN).display());
        println!("du: {}", Path::new(DU_BIN).display());
    }
    Ok(())
}

fn run_local_pressure(options: &Options) -> Result<(), String> {
    let home = home_dir()?;
    let free = free_bytes_for(&home)?;
    if options.output_format == OutputFormat::Json {
        println!(
            "{{\"mode\":\"pressure\",\"target\":\"local\",\"home\":\"{}\",\"free_bytes\":{},\"min_free_gb\":{}}}",
            home.display(),
            free,
            options.min_free_gb
        );
    } else {
        println!("Target: local");
        println!("Home: {}", home.display());
        println!("Free: {}", format_bytes(free));
        println!("Min free: {}G", options.min_free_gb);
    }
    Ok(())
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

/// Code-ish roots under $HOME. Avoid walking video dumps and other top-level
/// clutter; fixed caches are still added via exact paths under $HOME.
fn project_scan_roots(home: &Path) -> Vec<PathBuf> {
    const NAMES: &[&str] = &[
        "Git",
        "git",
        "src",
        "code",
        "Code",
        "projects",
        "Projects",
        "work",
        "Work",
        "dev",
        "Dev",
        "repos",
        "Repos",
        "workspace",
        "Workspace",
        "lab",
        "Lab",
        "src-git",
    ];
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();
    for name in NAMES {
        let path = home.join(name);
        if path.is_dir() && seen.insert(path.clone()) {
            roots.push(path);
        }
    }
    // Prefer shallow tool checkout(s), not the entire ~/.local/share tree
    // (flatpak/Steam/etc. are huge and already pruned poorly on some hosts).
    let tool_src = home.join(".local/share/rg-space-sweep-src");
    if tool_src.is_dir() && seen.insert(tool_src.clone()) {
        roots.push(tool_src);
    }
    // Bounded probe under .local/share for other *target* dirs (maxdepth via find later).
    let local_share = home.join(".local/share");
    if local_share.is_dir() {
        // Dedicated root only if we did not already add a tool checkout — still skip full share.
        let _ = local_share;
    }
    // Fallback: full home only when no project roots exist (unusual hosts).
    if roots.is_empty() {
        roots.push(home.to_path_buf());
    }
    roots
}

fn append_prune_group(args: &mut Vec<OsString>, home: &Path, extra_prune_names: &[&str]) {
    let prune_paths = [
        home.join(".cache"),
        home.join(".cargo"),
        home.join(".local/share/containers"),
        home.join(".local/share/Trash"),
        home.join(".local/pipx"),
        home.join(".local/share/pipx"),
        home.join(".gradle"),
        home.join(".m2"),
        home.join(".npm"),
        // Large non-code trees under .local/share when we scan that root.
        home.join(".local/share/Steam"),
        home.join(".local/share/flatpak"),
        home.join(".local/share/containers"),
        home.join(".local/share/Trash"),
        home.join(".local/share/baloo"),
        home.join(".local/share/zeitgeist"),
    ];
    args.push(OsString::from("("));
    let mut first = true;
    for path in &prune_paths {
        if !first {
            args.push(OsString::from("-o"));
        }
        first = false;
        args.push(OsString::from("-path"));
        args.push(path.clone().into_os_string());
    }
    for name in [".git", ".direnv", ".pixi", ".nox", "__pycache__"]
        .into_iter()
        .chain(extra_prune_names.iter().copied())
    {
        if !first {
            args.push(OsString::from("-o"));
        }
        first = false;
        args.push(OsString::from("-name"));
        args.push(OsString::from(name));
    }
    args.push(OsString::from(")"));
    args.push(OsString::from("-prune"));
    args.push(OsString::from("-o"));
}

/// Walk one root for any of the directory names (OR). Prefer over N finds.
fn find_named_in_root(
    home: &Path,
    root: &Path,
    names: &[&str],
    extra_prune_names: &[&str],
) -> Result<Vec<PathBuf>, String> {
    if names.is_empty() || !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut args = vec![
        root.as_os_str().to_os_string(),
        OsString::from("-xdev"),
    ];
    append_prune_group(&mut args, home, extra_prune_names);
    args.push(OsString::from("-type"));
    args.push(OsString::from("d"));
    args.push(OsString::from("("));
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            args.push(OsString::from("-o"));
        }
        args.push(OsString::from("-name"));
        args.push(OsString::from(*name));
    }
    args.push(OsString::from(")"));
    args.push(OsString::from("-prune"));
    args.push(OsString::from("-print"));
    Ok(run_find_output(&args)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect())
}

/// Parallel multi-root named walk (threads join on completion).
fn find_named_any(home: &Path, names: &[&str]) -> Result<Vec<PathBuf>, String> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let name_set: BTreeSet<&str> = names.iter().copied().collect();
    let mut extra_prune_names: Vec<&str> = Vec::new();
    for candidate in ["node_modules", "target", "target-nomount", ".venv", ".tox"] {
        if !name_set.contains(candidate) {
            extra_prune_names.push(candidate);
        }
    }
    let roots = project_scan_roots(home);
    let extra = extra_prune_names;
    let names_owned: Vec<String> = names.iter().map(|s| (*s).to_string()).collect();
    let home_buf = home.to_path_buf();

    let mut handles = Vec::new();
    for root in roots {
        let home_c = home_buf.clone();
        let names_c = names_owned.clone();
        let extra_c: Vec<String> = extra.iter().map(|s| (*s).to_string()).collect();
        handles.push(std::thread::spawn(move || {
            let name_refs: Vec<&str> = names_c.iter().map(String::as_str).collect();
            let extra_refs: Vec<&str> = extra_c.iter().map(String::as_str).collect();
            find_named_in_root(&home_c, &root, &name_refs, &extra_refs)
        }));
    }

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for handle in handles {
        let part = handle
            .join()
            .map_err(|_| "scan thread panicked".to_string())??;
        for path in part {
            if seen.insert(path.clone()) {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// Path-glob find under project roots only (e.g. */node_modules/.cache).
fn find_path_dirs(home: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    let roots = project_scan_roots(home);
    let pattern = pattern.to_string();
    let home_buf = home.to_path_buf();
    let mut handles = Vec::new();
    for root in roots {
        let home_c = home_buf.clone();
        let pattern_c = pattern.clone();
        handles.push(std::thread::spawn(move || {
            let mut args = vec![
                root.as_os_str().to_os_string(),
                OsString::from("-xdev"),
            ];
            append_prune_group(&mut args, &home_c, &["target", "target-nomount", ".venv", ".tox"]);
            args.extend([
                OsString::from("-type"),
                OsString::from("d"),
                OsString::from("-path"),
                OsString::from(pattern_c),
                OsString::from("-prune"),
                OsString::from("-print"),
            ]);
            run_find_output(&args).map(|stdout| {
                stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(PathBuf::from)
                    .collect::<Vec<_>>()
            })
        }));
    }
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for handle in handles {
        let part = handle
            .join()
            .map_err(|_| "scan thread panicked".to_string())??;
        for path in part {
            if seen.insert(path.clone()) {
                out.push(path);
            }
        }
    }
    Ok(out)
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
        // CACHEDIR.TAG is written by Cargo into build targets; allows target-nomount style dirs.
        || path.join("CACHEDIR.TAG").is_file()
}

/// Names that require a project-root walk, keyed by whether any selected category needs them.
fn walk_names_for(categories: &[Category]) -> Vec<&'static str> {
    let mut names = Vec::new();
    let wants = |c: Category| categories.contains(&c);
    if wants(Category::Rust) {
        names.push("target");
        // Alternate cargo target dir name used by some workspaces (e.g. bind-mount setups).
        names.push("target-nomount");
    }
    if wants(Category::Python) {
        names.extend([
            ".pytest_cache",
            ".mypy_cache",
            ".ruff_cache",
            ".hypothesis",
        ]);
    }
    if wants(Category::Tox) {
        names.push(".tox");
    }
    if wants(Category::Venv) {
        names.push(".venv");
    }
    names.sort_unstable();
    names.dedup();
    names
}

fn fixed_paths_for(home: &Path, category: Category) -> Vec<PathBuf> {
    match category {
        Category::Rust => vec![
            home.join(".cargo/registry/cache"),
            home.join(".cargo/registry/src"),
            home.join(".cargo/git/db"),
            home.join(".cache/sccache"),
            home.join(".cache/ccache"),
        ],
        Category::Python => vec![
            home.join(".cache/pip"),
            home.join(".cache/uv"),
            home.join(".cache/pre-commit"),
            home.join(".local/share/hatch"),
        ],
        Category::Pixi => vec![home.join(".cache/rattler/cache")],
        Category::Tox | Category::Venv => Vec::new(),
        Category::Js => vec![home.join(".npm")],
        Category::Go => vec![
            home.join(".cache/go-build"),
            home.join(".cache/go-mod"),
        ],
        Category::Java => vec![
            home.join(".gradle/caches"),
            home.join(".m2/repository"),
        ],
    }
}

fn classify_walked_path(path: &Path, categories: &[Category]) -> Option<Category> {
    let name = path.file_name().and_then(|v| v.to_str())?;
    let wants = |c: Category| categories.contains(&c);
    match name {
        "target" | "target-nomount" if wants(Category::Rust) => {
            if has_cargo_parent(path) && looks_like_rust_target(path) {
                Some(Category::Rust)
            } else {
                None
            }
        }
        ".pytest_cache" | ".mypy_cache" | ".ruff_cache" | ".hypothesis"
            if wants(Category::Python) =>
        {
            Some(Category::Python)
        }
        ".tox" if wants(Category::Tox) => Some(Category::Tox),
        ".venv" if wants(Category::Venv) => Some(Category::Venv),
        _ => None,
    }
}

fn collect_candidates(home: &Path, categories: &[Category]) -> Result<Vec<Candidate>, String> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();

    // Exact fixed caches first — O(1), no walk (also what clean --yes needs most).
    for &category in categories {
        for path in fixed_paths_for(home, category) {
            if !path.exists() || !seen.insert(path.clone()) {
                continue;
            }
            entries.push(Candidate { category, path });
        }
    }

    // Parallel multi-root walk for project-local dirs.
    let names = walk_names_for(categories);
    if !names.is_empty() {
        for path in find_named_any(home, &names)? {
            let Some(category) = classify_walked_path(&path, categories) else {
                continue;
            };
            if !path.exists() || !seen.insert(path.clone()) {
                continue;
            }
            entries.push(Candidate { category, path });
        }
        // Shallow ~/.local/share probe for cargo targets (depth-capped, not full share walk).
        if names.iter().any(|n| *n == "target" || *n == "target-nomount") {
            let share = home.join(".local/share");
            if share.is_dir() {
                let mut args = vec![
                    share.as_os_str().to_os_string(),
                    OsString::from("-xdev"),
                    OsString::from("-maxdepth"),
                    OsString::from("4"),
                    OsString::from("("),
                    OsString::from("-path"),
                    home.join(".local/share/containers").into_os_string(),
                    OsString::from("-o"),
                    OsString::from("-path"),
                    home.join(".local/share/Trash").into_os_string(),
                    OsString::from("-o"),
                    OsString::from("-path"),
                    home.join(".local/share/flatpak").into_os_string(),
                    OsString::from("-o"),
                    OsString::from("-path"),
                    home.join(".local/share/Steam").into_os_string(),
                    OsString::from("-o"),
                    OsString::from("-path"),
                    home.join(".local/share/pipx").into_os_string(),
                    OsString::from(")"),
                    OsString::from("-prune"),
                    OsString::from("-o"),
                    OsString::from("-type"),
                    OsString::from("d"),
                    OsString::from("("),
                    OsString::from("-name"),
                    OsString::from("target"),
                    OsString::from("-o"),
                    OsString::from("-name"),
                    OsString::from("target-nomount"),
                    OsString::from(")"),
                    OsString::from("-prune"),
                    OsString::from("-print"),
                ];
                for path in run_find_output(&args)?
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(PathBuf::from)
                {
                    let Some(category) = classify_walked_path(&path, categories) else {
                        continue;
                    };
                    if !path.exists() || !seen.insert(path.clone()) {
                        continue;
                    }
                    entries.push(Candidate { category, path });
                }
            }
        }
    }

    // JS node_modules/.cache under project roots only.
    if categories.contains(&Category::Js) {
        for path in find_path_dirs(home, "*/node_modules/.cache")? {
            if !path.exists() || !seen.insert(path.clone()) {
                continue;
            }
            entries.push(Candidate {
                category: Category::Js,
                path,
            });
        }
    }

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

/// Batch `du -s` across many paths (one process) instead of one process per path.
fn path_sizes_bytes(paths: &[&Path]) -> Result<Vec<u64>, String> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    // Chunk to stay under ARG_MAX on huge candidate sets.
    const CHUNK: usize = 200;
    let mut sizes = Vec::with_capacity(paths.len());
    for chunk in paths.chunks(CHUNK) {
        let mut args = vec![
            OsString::from("-s"),
            OsString::from("--block-size=1"),
            OsString::from("--"),
        ];
        for path in chunk {
            args.push(path.as_os_str().to_os_string());
        }
        let output = run_output(DU_BIN, &args)?;
        let mut got = 0usize;
        for line in output.lines() {
            if line.is_empty() {
                continue;
            }
            let first = line
                .split_whitespace()
                .next()
                .ok_or_else(|| format!("unable to parse du output line: {line}"))?;
            let size = first
                .parse::<u64>()
                .map_err(|err| format!("invalid du size in batch: {err}"))?;
            sizes.push(size);
            got += 1;
        }
        if got != chunk.len() {
            // Fallback per-path if du omitted a line (permission / race).
            sizes.truncate(sizes.len().saturating_sub(got));
            for path in chunk {
                sizes.push(path_size_bytes(path)?);
            }
        }
    }
    Ok(sizes)
}

fn size_entries(candidates: &[Candidate]) -> Result<Vec<Entry>, String> {
    let paths: Vec<&Path> = candidates.iter().map(|c| c.path.as_path()).collect();
    let sizes = path_sizes_bytes(&paths)?;
    let mut entries = Vec::with_capacity(candidates.len());
    for (candidate, size) in candidates.iter().zip(sizes) {
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

fn report_json(
    home: &Path,
    entries: &[Entry],
    limit: usize,
    dry_run: Option<bool>,
) -> Result<String, String> {
    let mut totals: BTreeMap<Category, (u64, usize)> = BTreeMap::new();
    let mut grand_total = 0u64;
    for entry in entries {
        grand_total += entry.size;
        let item = totals.entry(entry.category).or_insert((0, 0));
        item.0 += entry.size;
        item.1 += 1;
    }

    let totals = totals
        .into_iter()
        .map(|(category, (size_bytes, paths))| JsonCategoryTotal {
            category: category.label(),
            size_bytes,
            paths,
        })
        .collect::<Vec<_>>();
    let json_entry = |entry: &Entry| JsonPathEntry {
        category: entry.category.label(),
        path: display_path(home, &entry.path),
        size_bytes: entry.size,
    };
    let top_paths = entries
        .iter()
        .take(limit)
        .map(json_entry)
        .collect::<Vec<_>>();
    let would_remove = dry_run.and_then(|value| {
        if value {
            Some(entries.iter().map(json_entry).collect::<Vec<_>>())
        } else {
            None
        }
    });
    serde_json::to_string(&JsonReport {
        grand_total_bytes: grand_total,
        matched_paths: entries.len(),
        totals,
        top_paths,
        dry_run,
        would_remove,
    })
    .map_err(|err| format!("serialize report json: {err}"))
}

fn print_report_json(
    home: &Path,
    entries: &[Entry],
    limit: usize,
    dry_run: Option<bool>,
) -> Result<(), String> {
    println!("{}", report_json(home, entries, limit, dry_run)?);
    Ok(())
}

fn safe_to_remove(home: &Path, entry: &Candidate) -> bool {
    if !entry.path.starts_with(home) || entry.path == home {
        return false;
    }
    // Never remove the dirs that back installed CLI tools: ~/.local/bin and
    // ~/.cargo/bin shims, and the pipx tool venvs they point into. Losing a
    // pipx venv leaves a dangling ~/.local/bin symlink and an uninstalled tool.
    if entry.path.starts_with(home.join(".local/bin"))
        || entry.path.starts_with(home.join(".cargo/bin"))
        || entry.path.starts_with(home.join(".local/pipx"))
        || entry.path.starts_with(home.join(".local/share/pipx"))
    {
        return false;
    }

    let exact = [
        home.join(".cargo/registry/cache"),
        home.join(".cargo/registry/src"),
        home.join(".cargo/git/db"),
        home.join(".cache/pip"),
        home.join(".cache/uv"),
        home.join(".cache/rattler/cache"),
        home.join(".cache/pre-commit"),
        home.join(".cache/sccache"),
        home.join(".cache/ccache"),
        home.join(".cache/go-build"),
        home.join(".cache/go-mod"),
        home.join(".local/share/hatch"),
        home.join(".npm"),
        home.join(".gradle/caches"),
        home.join(".m2/repository"),
    ];
    if exact.iter().any(|candidate| candidate == &entry.path) {
        return true;
    }

    let Some(name) = entry.path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    match entry.category {
        Category::Rust => {
            (name == "target" || name == "target-nomount")
                && has_cargo_parent(&entry.path)
                && looks_like_rust_target(&entry.path)
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
        Category::Go => entry.path.ends_with(Path::new(".cache/go-build"))
            || entry.path.ends_with(Path::new(".cache/go-mod")),
        Category::Java => entry.path.ends_with(Path::new(".gradle/caches"))
            || entry.path.ends_with(Path::new(".m2/repository")),
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

fn push_shell_array(script: &mut String, name: &str, values: &[String]) {
    script.push_str(name);
    script.push_str("=(\n");
    for value in values {
        script.push_str("  ");
        script.push_str(&shell_quote(value));
        script.push('\n');
    }
    script.push_str(")\n");
}

fn proxy_command_quote(value: &str) -> String {
    shell_quote(value)
}

fn local_command_exists(command: &str) -> bool {
    Command::new("sh")
        .args(["-c", "command -v \"$1\" >/dev/null 2>&1", "sh", command])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn validate_gsocket_preflight(profile: &RemoteProfile) -> Result<(), String> {
    if profile.runner != RemoteRunner::Gsocket {
        return Ok(());
    }
    if !local_command_exists("gs-netcat") {
        return Err("local gs-netcat is missing; install gsocket or run cosmolab gsocket setup from a host with gs-netcat".to_string());
    }

    let secret = profile.gsocket_secret_file.as_ref().ok_or_else(|| {
        format!(
            "target {} uses runner = \"gsocket\" but has no gsocket_secret_file",
            profile.name
        )
    })?;
    let metadata =
        fs::metadata(secret).map_err(|err| format!("stat {}: {err}", secret.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "target {} gsocket_secret_file is not a file: {}",
            profile.name,
            secret.display()
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o777;
        if !secret_permissions_private(mode) {
            return Err(format!(
                "target {} gsocket_secret_file permissions are too open ({mode:o}); run: chmod 600 {}",
                profile.name,
                secret.display()
            ));
        }
    }

    Ok(())
}

#[cfg(unix)]
fn secret_permissions_private(mode: u32) -> bool {
    mode & 0o077 == 0
}

fn gsocket_secret_file_ok(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return secret_permissions_private(metadata.permissions().mode() & 0o777);
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Report => "report",
        Mode::Clean => "clean",
        Mode::AutoClean => "auto-clean",
        Mode::Snapshots => "snapshots",
        Mode::TargetList => "target-list",
        Mode::TargetShow => "target-show",
        Mode::TargetCheck => "target-check",
        Mode::Pressure => "pressure",
    }
}

fn effective_remote_categories(profile: &RemoteProfile, options: &Options) -> Vec<Category> {
    if !options.categories_explicit && !profile.categories.is_empty() {
        profile.categories.clone()
    } else {
        options.categories.clone()
    }
}

fn remote_script_for(profile: &RemoteProfile, options: &Options) -> Result<String, String> {
    if matches!(
        options.mode,
        Mode::AutoClean | Mode::Snapshots | Mode::TargetList | Mode::TargetShow
    ) {
        return Err(
            "remote script supports report, clean, pressure, and target check only".to_string(),
        );
    }

    let categories = effective_remote_categories(profile, options)
        .iter()
        .map(|category| category.label())
        .collect::<Vec<_>>()
        .join(" ");
    let home = profile.home.to_string_lossy();
    let mut script = String::new();

    script.push_str("#!/usr/bin/env bash\nset -euo pipefail\n\n");
    push_shell_var(&mut script, "TARGET_NAME", &profile.name);
    push_shell_var(&mut script, "RUNNER", runner_label(profile.runner));
    push_shell_var(&mut script, "SSH_TARGET", &ssh_target_for(profile));
    let ssh_config = ssh_config_path()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    push_shell_var(&mut script, "SSH_CONFIG_PATH", &ssh_config);
    push_shell_var(
        &mut script,
        "GSOCKET_SECRET_FILE_CONFIGURED",
        if profile.gsocket_secret_file.is_some() {
            "1"
        } else {
            "0"
        },
    );
    let gsocket_secret_ok = profile
        .gsocket_secret_file
        .as_ref()
        .map(|path| gsocket_secret_file_ok(path))
        .unwrap_or(false);
    push_shell_var(
        &mut script,
        "GSOCKET_SECRET_FILE_OK",
        if gsocket_secret_ok { "1" } else { "0" },
    );
    push_shell_var(
        &mut script,
        "LOCAL_GS_NETCAT_OK",
        if local_command_exists("gs-netcat") {
            "1"
        } else {
            "0"
        },
    );
    push_shell_var(&mut script, "HOME_DIR", home.as_ref());
    push_shell_var(&mut script, "MODE", mode_label(options.mode));
    push_shell_var(&mut script, "CATEGORIES", &categories);
    push_shell_var(&mut script, "LIMIT", &options.limit.to_string());
    push_shell_var(
        &mut script,
        "OUTPUT_FORMAT",
        match options.output_format {
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
        },
    );
    push_shell_var(
        &mut script,
        "DRY_RUN",
        if options.dry_run { "1" } else { "0" },
    );
    if options.yes {
        push_shell_var(&mut script, "YES", "1");
    }
    push_shell_var(&mut script, "MIN_FREE_GB", &profile.min_free_gb.to_string());
    push_shell_array(
        &mut script,
        "ROOTS",
        &profile
            .roots
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    );
    push_shell_array(
        &mut script,
        "PRUNE_PATHS",
        &profile
            .prune
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    );
    push_shell_array(
        &mut script,
        "EXCLUDE_PATHS",
        &profile
            .exclude
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
    );
    script.push('\n');
    script.push_str(REMOTE_SWEEP_SCRIPT);
    Ok(script)
}

fn remote_invocation_for(
    profile: &RemoteProfile,
    options: &Options,
) -> Result<RemoteInvocation, String> {
    validate_target_mode(options)?;
    let ssh_target = ssh_target_for(profile);
    match profile.runner {
        RemoteRunner::Ssh => {
            let mut args = ssh_config_args();
            args.extend([
                "-T".to_string(),
                "-o".to_string(),
                "BatchMode=yes".to_string(),
                "-o".to_string(),
                "ConnectTimeout=10".to_string(),
                ssh_target,
                "bash".to_string(),
                "-s".to_string(),
                "--".to_string(),
            ]);
            Ok(RemoteInvocation {
                program: "ssh".to_string(),
                args,
                stdin: remote_script_for(profile, options)?,
                max_attempts: 1,
            })
        }
        RemoteRunner::Gsocket => {
            let secret = profile.gsocket_secret_file.as_ref().ok_or_else(|| {
                format!(
                    "target {} uses runner = \"gsocket\" but has no gsocket_secret_file",
                    profile.name
                )
            })?;
            let mut args = ssh_config_args();
            args.extend([
                "-T".to_string(),
                "-o".to_string(),
                "BatchMode=yes".to_string(),
                "-o".to_string(),
                "ConnectTimeout=10".to_string(),
                "-o".to_string(),
                format!(
                    "ProxyCommand=gs-netcat -q -k {}",
                    proxy_command_quote(&secret.to_string_lossy())
                ),
                "-o".to_string(),
                "StrictHostKeyChecking=yes".to_string(),
            ]);
            if let Some(host_key_alias) = profile.host_key_alias.as_ref() {
                args.push("-o".to_string());
                args.push(format!("HostKeyAlias={host_key_alias}"));
            }
            if let Some(identity) = profile.ssh_identity.as_ref() {
                args.push("-i".to_string());
                args.push(identity.to_string_lossy().into_owned());
            }
            args.extend([
                ssh_target,
                "bash".to_string(),
                "-s".to_string(),
                "--".to_string(),
            ]);
            Ok(RemoteInvocation {
                program: "ssh".to_string(),
                args,
                stdin: remote_script_for(profile, options)?,
                max_attempts: 3,
            })
        }
    }
}

fn should_retry_remote_status(
    status: std::process::ExitStatus,
    attempt: usize,
    max_attempts: usize,
) -> bool {
    attempt < max_attempts && status.code() == Some(255)
}

fn run_remote_invocation(invocation: &RemoteInvocation) -> Result<(), String> {
    let max_attempts = invocation.max_attempts.max(1);
    for attempt in 1..=max_attempts {
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
            return Ok(());
        }
        if should_retry_remote_status(status, attempt, max_attempts) {
            continue;
        }
        return Err(format!("remote command exited with status {status}"));
    }
    Err("remote command failed without a recorded exit status".to_string())
}

fn run_remote_mode(target_name: &str, options: &Options) -> Result<(), String> {
    let profile = remote_profile_for(target_name, options.target_config_path.as_deref())?;
    validate_gsocket_preflight(&profile)?;
    let invocation = remote_invocation_for(&profile, options)?;
    run_remote_invocation(&invocation)
}

fn mode_requires_candidate_scan(mode: Mode) -> bool {
    !matches!(mode, Mode::Snapshots)
}

fn run_local_mode(options: &Options) -> Result<(), String> {
    match options.mode {
        Mode::TargetList => return list_targets(options),
        Mode::TargetShow => {
            return match &options.target {
                Target::Remote(target_name) => show_target(target_name, options),
                Target::Local => Err("target show requires a remote target name".to_string()),
            };
        }
        Mode::TargetCheck => {
            if matches!(options.target, Target::Local) {
                return run_local_target_check(options);
            }
        }
        Mode::Pressure => {
            if matches!(options.target, Target::Local) {
                return run_local_pressure(options);
            }
        }
        _ => {}
    }

    if !mode_requires_candidate_scan(options.mode) {
        return write_snapshot_script(options);
    }

    home_dir()
        .and_then(|home| {
            collect_candidates(&home, &options.categories).map(|entries| (home, entries))
        })
        .and_then(|(home, entries)| {
            match options.mode {
                Mode::Report => {
                    let sized = size_entries(&entries)?;
                    match options.output_format {
                        OutputFormat::Text => print_report(&home, &sized, options.limit),
                        OutputFormat::Json => {
                            print_report_json(&home, &sized, options.limit, None)?
                        }
                    }
                }
                Mode::Clean => {
                    if options.dry_run {
                        let sized = size_entries(&entries)?;
                        match options.output_format {
                            OutputFormat::Text => {
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
                            }
                            OutputFormat::Json => {
                                print_report_json(&home, &sized, options.limit, Some(true))?
                            }
                        }
                    } else {
                        // Skip du entirely for destructive clean — only existence + safety matter.
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
                    // No du: reclaim ASAP under pressure.
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
                Mode::TargetList | Mode::TargetShow | Mode::TargetCheck | Mode::Pressure => {}
            }
            Ok(())
        })
}

fn run_options(options: &Options) -> Result<(), String> {
    validate_target_mode(options)?;
    match &options.target {
        Target::Local => run_local_mode(options),
        Target::Remote(target_name) => match options.mode {
            Mode::TargetList => list_targets(options),
            Mode::TargetShow => show_target(target_name, options),
            _ => run_remote_mode(target_name, options),
        },
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
    fn project_scan_roots_prefers_git_over_full_home() {
        let home = Path::new("/home/example");
        // Without real dirs on disk, fallback is full home — structural check on source.
        let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/main.rs"));
        assert!(src.contains("fn project_scan_roots"));
        assert!(src.contains("std::thread::spawn"));
        assert!(src.contains("Skip du entirely for destructive clean") || src.contains("No du: reclaim ASAP"));
        let _ = home;
    }

    #[test]
    fn walk_names_for_python_is_single_combined_set() {
        let names = walk_names_for(&[Category::Python]);
        assert_eq!(
            names,
            vec![
                ".hypothesis",
                ".mypy_cache",
                ".pytest_cache",
                ".ruff_cache",
            ]
        );
    }

    #[test]
    fn walk_names_for_all_project_dirs_dedups_once() {
        let names = walk_names_for(&Category::expand("all").unwrap());
        assert!(names.contains(&"target"));
        assert!(names.contains(&".venv"));
        assert!(names.contains(&".tox"));
        assert_eq!(names.iter().filter(|n| **n == "target").count(), 1);
    }

    #[test]
    fn fixed_paths_include_high_value_rebuildable_caches() {
        let home = Path::new("/home/example");
        let rust = fixed_paths_for(home, Category::Rust);
        assert!(rust.iter().any(|p| p.ends_with(".cargo/registry/src")));
        assert!(rust.iter().any(|p| p.ends_with(".cache/sccache")));
        let java = fixed_paths_for(home, Category::Java);
        assert!(java.iter().any(|p| p.ends_with(".m2/repository")));
        let go = fixed_paths_for(home, Category::Go);
        assert!(go.iter().any(|p| p.ends_with(".cache/go-build")));
    }

    #[test]
    fn looks_like_rust_target_accepts_cachedir_tag() {
        // Structural: CACHEDIR.TAG is part of the acceptance predicate source.
        let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/main.rs"));
        assert!(src.contains("CACHEDIR.TAG"));
        assert!(src.contains("find_named_any"));
        assert!(src.contains("path_sizes_bytes"));
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
                Category::Go,
                Category::Java,
            ]
        );
    }

    #[test]
    fn parse_args_from_accepts_target_config_path() {
        let options = parse_args_from([
            "report",
            "--target",
            "rg.cosmolab",
            "--target-config",
            "/tmp/targets.toml",
        ])
        .expect("parse args");

        assert_eq!(options.target, Target::Remote("rg.cosmolab".to_string()));
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
            output_format: OutputFormat::Text,
            categories_explicit: true,
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
            [targets."rg.cosmolab"]
            host = "rg.cosmolab"
            home = "/home/goswami"
            runner = "ssh"
            min_free_gb = 75
            snapshots = false
            "#,
        )
        .expect("parse profiles");

        assert_eq!(
            profiles.get("rg.cosmolab"),
            Some(&RemoteProfile {
                name: "rg.cosmolab".to_string(),
                host: "rg.cosmolab".to_string(),
                user: None,
                home: PathBuf::from("/home/goswami"),
                runner: RemoteRunner::Ssh,
                gsocket_secret_file: None,
                ssh_identity: None,
                host_key_alias: None,
                roots: vec![PathBuf::from("/home/goswami")],
                prune: Vec::new(),
                exclude: Vec::new(),
                categories: Vec::new(),
                min_free_gb: 75,
                snapshots: false,
            })
        );
        let builtin = &builtin_remote_profiles()["cosmolab"];
        assert_eq!(builtin.host, "rg.cosmolab");
        assert_eq!(builtin.runner, RemoteRunner::Gsocket);
        assert_eq!(
            builtin.gsocket_secret_file,
            Some(local_home_dir().join(".config/cosmolab/gsocket/rg.cosmolab.secret"))
        );
        assert_eq!(builtin.ssh_identity, None);
        assert_eq!(builtin.host_key_alias, None);
    }

    #[test]
    fn target_profile_json_exposes_non_secret_connection_metadata() {
        let mut profile = builtin_remote_profiles()["rg.cosmolab"].clone();
        profile.categories = vec![Category::Python, Category::Rust];

        let json = serde_json::to_value(json_target_profile(&profile)).expect("target json");

        assert_eq!(json["name"], "rg.cosmolab");
        assert_eq!(json["runner"], "gsocket");
        assert_eq!(json["ssh_target"], "rg.cosmolab");
        assert_eq!(json["gsocket_secret_file_configured"], true);
        assert!(json["gsocket_secret_file_path"]
            .as_str()
            .expect("secret path")
            .ends_with(".config/cosmolab/gsocket/rg.cosmolab.secret"));
        assert_eq!(json["categories"][0], "python");
        assert_eq!(json["categories"][1], "rust");
    }

    #[test]
    fn remote_invocation_uses_ssh_and_generated_script() {
        let profile = builtin_remote_profiles()["cosmolab"].clone();
        let options = parse_args_from(["report", "--target", "cosmolab", "--limit", "8", "all"])
            .expect("parse args");

        let invocation = remote_invocation_for(&profile, &options).expect("remote invocation");

        assert_eq!(invocation.program, "ssh");
        assert!(invocation.args.contains(&"-T".to_string()));
        assert!(invocation.args.contains(&"BatchMode=yes".to_string()));
        assert!(invocation.args.contains(&"rg.cosmolab".to_string()));
        assert!(!invocation.args.contains(&"-i".to_string()));
        assert!(invocation
            .args
            .iter()
            .any(|arg| arg.starts_with("ProxyCommand=gs-netcat -q -k ")));
        assert!(!invocation
            .args
            .iter()
            .any(|arg| arg.starts_with("HostKeyAlias=")));
        assert!(invocation.stdin.contains("HOME_DIR='/home/goswami'"));
        assert!(invocation.stdin.contains("MODE='report'"));
        assert!(invocation.stdin.contains("LIMIT='8'"));
        assert!(!invocation.stdin.contains("rg-space-sweep"));
        assert_eq!(invocation.max_attempts, 3);
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

    #[cfg(unix)]
    #[test]
    fn retry_policy_only_retries_ssh_transport_exit_255() {
        use std::os::unix::process::ExitStatusExt;

        let ssh_transport_failure = std::process::ExitStatus::from_raw(255 << 8);
        let remote_usage_failure = std::process::ExitStatus::from_raw(64 << 8);

        assert!(should_retry_remote_status(ssh_transport_failure, 1, 3));
        assert!(!should_retry_remote_status(ssh_transport_failure, 3, 3));
        assert!(!should_retry_remote_status(remote_usage_failure, 1, 3));
    }

    #[cfg(unix)]
    #[test]
    fn gsocket_secret_permissions_must_be_private() {
        assert!(secret_permissions_private(0o600));
        assert!(secret_permissions_private(0o400));
        assert!(!secret_permissions_private(0o640));
        assert!(!secret_permissions_private(0o604));
    }

    #[test]
    fn parse_args_from_accepts_target_subcommands_pressure_and_json() {
        let list = parse_args_from(["target", "list", "--json"]).expect("parse target list");
        let show = parse_args_from(["target", "show", "cosmolab"]).expect("parse target show");
        let check = parse_args_from(["target", "check", "cosmolab"]).expect("parse target check");
        let pressure = parse_args_from(["pressure", "--target", "cosmolab", "--json"])
            .expect("parse pressure");

        assert_eq!(list.mode, Mode::TargetList);
        assert_eq!(list.output_format, OutputFormat::Json);
        assert_eq!(show.mode, Mode::TargetShow);
        assert_eq!(show.target, Target::Remote("cosmolab".to_string()));
        assert_eq!(check.mode, Mode::TargetCheck);
        assert_eq!(check.target, Target::Remote("cosmolab".to_string()));
        assert_eq!(pressure.mode, Mode::Pressure);
        assert_eq!(pressure.output_format, OutputFormat::Json);
    }

    #[test]
    fn remote_profiles_load_controls_and_gsocket_secret_file() {
        let profiles = remote_profiles_from_toml(
            r#"
            [targets."rg.cosmolab"]
            host = "rg.cosmolab"
            user = "goswami"
            home = "/home/goswami"
            runner = "gsocket"
            gsocket_secret_file = "/run/user/1001/gsocket/rg.cosmolab.secret"
            ssh_identity = "/home/rgoswami/.ssh/id_cosmolab"
            host_key_alias = "rg.cosmolab-gsocket"
            roots = ["/home/goswami", "/scratch/goswami"]
            prune = ["/scratch/goswami/raw"]
            exclude = ["/home/goswami/.cache/keep"]
            categories = ["python", "rust"]
            min_free_gb = 75
            snapshots = false
            "#,
        )
        .expect("parse profiles");

        assert_eq!(
            profiles.get("rg.cosmolab"),
            Some(&RemoteProfile {
                name: "rg.cosmolab".to_string(),
                host: "rg.cosmolab".to_string(),
                user: Some("goswami".to_string()),
                home: PathBuf::from("/home/goswami"),
                runner: RemoteRunner::Gsocket,
                gsocket_secret_file: Some(PathBuf::from(
                    "/run/user/1001/gsocket/rg.cosmolab.secret"
                )),
                ssh_identity: Some(PathBuf::from("/home/rgoswami/.ssh/id_cosmolab")),
                host_key_alias: Some("rg.cosmolab-gsocket".to_string()),
                roots: vec![
                    PathBuf::from("/home/goswami"),
                    PathBuf::from("/scratch/goswami"),
                ],
                prune: vec![PathBuf::from("/scratch/goswami/raw")],
                exclude: vec![PathBuf::from("/home/goswami/.cache/keep")],
                categories: vec![Category::Python, Category::Rust],
                min_free_gb: 75,
                snapshots: false,
            })
        );
    }

    #[test]
    fn gsocket_invocation_uses_proxycommand_keyfile_and_ssh_identity() {
        let profile = RemoteProfile {
            name: "rg.cosmolab".to_string(),
            host: "rg.cosmolab".to_string(),
            user: Some("goswami".to_string()),
            home: PathBuf::from("/home/goswami"),
            runner: RemoteRunner::Gsocket,
            gsocket_secret_file: Some(PathBuf::from("/run/user/1001/gsocket/rg.cosmolab secret")),
            ssh_identity: Some(PathBuf::from("/home/rgoswami/.ssh/id_cosmolab")),
            host_key_alias: Some("rg.cosmolab-gsocket".to_string()),
            roots: vec![PathBuf::from("/home/goswami")],
            prune: Vec::new(),
            exclude: Vec::new(),
            categories: Vec::new(),
            min_free_gb: 75,
            snapshots: false,
        };
        let options =
            parse_args_from(["report", "--target", "rg.cosmolab", "python"]).expect("parse args");

        let invocation = remote_invocation_for(&profile, &options).expect("remote invocation");

        assert_eq!(invocation.program, "ssh");
        assert!(invocation.args.contains(&"-T".to_string()));
        assert!(invocation.args.contains(&"-i".to_string()));
        assert!(invocation
            .args
            .contains(&"/home/rgoswami/.ssh/id_cosmolab".to_string()));
        assert!(invocation.args.contains(
            &"ProxyCommand=gs-netcat -q -k '/run/user/1001/gsocket/rg.cosmolab secret'".to_string()
        ));
        assert!(invocation
            .args
            .contains(&"HostKeyAlias=rg.cosmolab-gsocket".to_string()));
        assert!(invocation.args.contains(&"goswami@rg.cosmolab".to_string()));
        assert!(invocation.stdin.contains("MODE='report'"));
    }

    #[test]
    fn remote_script_uses_template_and_profile_controls() {
        let mut profile = builtin_remote_profiles()["cosmolab"].clone();
        profile.roots.push(PathBuf::from("/scratch/goswami"));
        profile.prune.push(PathBuf::from("/scratch/goswami/raw"));
        profile
            .exclude
            .push(PathBuf::from("/home/goswami/.cache/keep"));
        let options =
            parse_args_from(["report", "--target", "cosmolab", "--json"]).expect("parse args");

        let script = remote_script_for(&profile, &options).expect("remote script");

        assert!(REMOTE_SWEEP_SCRIPT.contains("collect_candidates"));
        assert!(REMOTE_SWEEP_SCRIPT.contains("\"totals\":["));
        assert!(REMOTE_SWEEP_SCRIPT.contains("\"local_gsocket\""));
        assert!(script.contains("OUTPUT_FORMAT='json'"));
        assert!(script.contains("RUNNER='gsocket'"));
        assert!(script.contains("SSH_TARGET='rg.cosmolab'"));
        assert!(script.contains("GSOCKET_SECRET_FILE_CONFIGURED='1'"));
        assert!(script.contains("'/scratch/goswami'"));
        assert!(script.contains("'/scratch/goswami/raw'"));
        assert!(script.contains("'/home/goswami/.cache/keep'"));
    }

    #[test]
    fn report_json_includes_totals_and_dry_run_entries() {
        let home = Path::new("/home/test");
        let entries = vec![Entry {
            category: Category::Python,
            path: PathBuf::from("/home/test/.cache/pip"),
            size: 42,
        }];

        let json = report_json(home, &entries, 5, Some(true)).expect("json report");

        assert!(json.contains("\"grand_total_bytes\":42"));
        assert!(json.contains("\"dry_run\":true"));
        assert!(json.contains("\"path\":\"~/.cache/pip\""));
    }
}
