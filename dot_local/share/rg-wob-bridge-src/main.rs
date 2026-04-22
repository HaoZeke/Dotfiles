use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::Duration;

const MKFIFO_BIN: &str = "/usr/bin/mkfifo";
const WOB_BIN: &str = "/usr/bin/wob";

fn runtime_dir() -> Result<PathBuf, String> {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "XDG_RUNTIME_DIR is not set".to_string())
}

fn fifo_path() -> Result<PathBuf, String> {
    Ok(runtime_dir()?.join("wob.sock"))
}

fn recreate_fifo(path: &Path) -> Result<(), String> {
    if path.exists() {
        let metadata =
            fs::symlink_metadata(path).map_err(|err| format!("failed to stat {}: {err}", path.display()))?;
        if metadata.file_type().is_dir() {
            return Err(format!("refusing to replace directory {}", path.display()));
        }
        fs::remove_file(path).map_err(|err| format!("failed to remove {}: {err}", path.display()))?;
    }

    let status = Command::new(MKFIFO_BIN)
        .arg(path)
        .status()
        .map_err(|err| format!("failed to run mkfifo: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("mkfifo exited with status {status}"))
    }
}

fn run() -> Result<(), String> {
    let fifo = fifo_path()?;
    recreate_fifo(&fifo)?;

    let mut wob = Command::new(WOB_BIN)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to start wob: {err}"))?;

    let _guard = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&fifo)
        .map_err(|err| format!("failed to open guard handle for {}: {err}", fifo.display()))?;
    let reader_file = OpenOptions::new()
        .read(true)
        .open(&fifo)
        .map_err(|err| format!("failed to open reader for {}: {err}", fifo.display()))?;
    let mut reader = BufReader::new(reader_file);
    let mut wob_stdin = wob
        .stdin
        .take()
        .ok_or_else(|| "wob stdin unavailable".to_string())?;

    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|err| format!("failed reading {}: {err}", fifo.display()))?;
        if bytes == 0 {
            if let Some(status) = wob
                .try_wait()
                .map_err(|err| format!("failed to poll wob: {err}"))?
            {
                return Err(format!("wob exited with status {status}"));
            }
            thread::sleep(Duration::from_millis(50));
            continue;
        }

        wob_stdin
            .write_all(line.as_bytes())
            .map_err(|err| format!("failed writing to wob stdin: {err}"))?;
        wob_stdin
            .flush()
            .map_err(|err| format!("failed flushing wob stdin: {err}"))?;
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}
