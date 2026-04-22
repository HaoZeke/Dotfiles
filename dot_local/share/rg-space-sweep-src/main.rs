use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};

const FIND_BIN: &str = "/usr/bin/find";
const DU_BIN: &str = "/usr/bin/du";

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
            "default" => Ok(vec![Self::Rust, Self::Python, Self::Pixi, Self::Tox]),
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
}

#[derive(Debug)]
struct Options {
    mode: Mode,
    categories: Vec<Category>,
    dry_run: bool,
    yes: bool,
    limit: usize,
}

#[derive(Clone, Debug)]
struct Entry {
    category: Category,
    path: PathBuf,
    size: u64,
}

fn usage() -> &'static str {
    "\
usage: rg-space-sweep [report|clean] [--dry-run] [--yes] [--limit N] [default|all|rust|python|pixi|tox|venv|js]

report
    Show category totals and the largest matching cache/build directories.

clean
    Remove the matching directories. Requires --yes, or use --dry-run to preview.

default
    rust python pixi tox

all
    default + venv + js"
}

fn parse_args() -> Result<Options, String> {
    let mut mode = Mode::Report;
    let mut dry_run = false;
    let mut yes = false;
    let mut limit = 20usize;
    let mut category_tokens = Vec::new();

    let mut args = env::args().skip(1);
    if let Some(first) = args.next() {
        match first.as_str() {
            "report" => mode = Mode::Report,
            "clean" => mode = Mode::Clean,
            "-h" | "--help" | "help" => return Err(usage().to_string()),
            other => category_tokens.push(other.to_string()),
        }
    }

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--yes" => yes = true,
            "--limit" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--limit requires a value".to_string())?;
                limit = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --limit value: {value}"))?;
            }
            "-h" | "--help" | "help" => return Err(usage().to_string()),
            other => category_tokens.push(other.to_string()),
        }
    }

    if category_tokens.is_empty() {
        category_tokens.push("default".to_string());
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
    })
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
        Category::Pixi => find_named_dirs(home, ".pixi")?,
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

fn collect_entries(home: &Path, categories: &[Category]) -> Result<Vec<Entry>, String> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();

    for &category in categories {
        for path in emit_category_paths(home, category)? {
            if !path.exists() || !seen.insert(path.clone()) {
                continue;
            }
            let size = path_size_bytes(&path)?;
            entries.push(Entry {
                category,
                path,
                size,
            });
        }
    }

    entries.sort_by(|left, right| right.size.cmp(&left.size).then_with(|| left.path.cmp(&right.path)));
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

fn safe_to_remove(home: &Path, entry: &Entry) -> bool {
    if !entry.path.starts_with(home) || entry.path == home {
        return false;
    }

    let exact = [
        home.join(".cargo/registry/cache"),
        home.join(".cargo/git/db"),
        home.join(".cache/pip"),
        home.join(".cache/uv"),
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
        Category::Rust => name == "target" && has_cargo_parent(&entry.path) && looks_like_rust_target(&entry.path),
        Category::Python => matches!(name, ".pytest_cache" | ".mypy_cache" | ".ruff_cache" | ".hypothesis"),
        Category::Pixi => name == ".pixi",
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

fn clean_entries(home: &Path, entries: &[Entry], dry_run: bool, yes: bool) -> Result<(), String> {
    if !dry_run && !yes {
        return Err("refusing to clean without --yes".to_string());
    }

    for entry in entries {
        if !safe_to_remove(home, entry) {
            return Err(format!("refusing to remove unexpected path: {}", entry.path.display()));
        }
    }

    if dry_run {
        println!("Dry run");
        for entry in entries {
            println!(
                "would remove {}  {:<6}  {}",
                format_bytes(entry.size),
                entry.category.label(),
                display_path(home, &entry.path)
            );
        }
        return Ok(());
    }

    for entry in entries {
        println!(
            "removing {}  {:<6}  {}",
            format_bytes(entry.size),
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
        .and_then(|home| collect_entries(&home, &options.categories).map(|entries| (home, entries)))
        .and_then(|(home, entries)| {
            match options.mode {
                Mode::Report => print_report(&home, &entries, options.limit),
                Mode::Clean => {
                    print_report(&home, &entries, options.limit);
                    println!();
                    clean_entries(&home, &entries, options.dry_run, options.yes)?;
                }
            }
            Ok(())
        });

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(1);
    }
}
