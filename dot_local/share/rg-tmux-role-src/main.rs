use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{self, Command, ExitStatus, Stdio};

const TMUX_BIN: &str = "/usr/bin/tmux";

#[derive(Clone, Copy, Debug)]
enum Role {
    Main,
    Float,
}

impl Role {
    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "main" => Ok(Self::Main),
            "float" => Ok(Self::Float),
            other => Err(format!("unknown tmux role: {other}")),
        }
    }

    fn session(self) -> &'static str {
        match self {
            Self::Main => "mS",
            Self::Float => "subFloat",
        }
    }
}

fn run_tmux<I, S>(args: I) -> Result<ExitStatus, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(TMUX_BIN)
        .args(args)
        .status()
        .map_err(|err| format!("failed to run tmux: {err}"))
}

fn run_tmux_checked<I, S>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = run_tmux(args)?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux exited with status {status}"))
    }
}

fn ensure_session(role: Role) -> Result<(), String> {
    let session = role.session();
    let status = run_tmux(["has-session", "-t", session])?;
    if status.success() {
        return Ok(());
    }
    run_tmux_checked(["new-session", "-d", "-s", session])
}

fn attach_session(role: Role) -> Result<(), String> {
    ensure_session(role)?;
    let status = Command::new(TMUX_BIN)
        .args(["attach-session", "-t", role.session()])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| format!("failed to attach tmux session: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux exited with status {status}"))
    }
}

fn home_dir() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn resurrect_script(name: &str) -> Result<PathBuf, String> {
    Ok(home_dir()?
        .join(".tmux")
        .join("plugins")
        .join("tmux-resurrect")
        .join("scripts")
        .join(name))
}

fn run_resurrect(name: &str) -> Result<(), String> {
    let script = resurrect_script(name)?;
    if !script.is_file() {
        return Err(format!("missing tmux-resurrect script: {}", script.display()));
    }
    let status = Command::new(TMUX_BIN)
        .args(["run-shell", script.to_string_lossy().as_ref()])
        .status()
        .map_err(|err| format!("failed to run tmux resurrect hook: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux exited with status {status}"))
    }
}

fn run_optional_resurrect(name: &str) -> Result<(), String> {
    let script = resurrect_script(name)?;
    if !script.is_file() {
        return Ok(());
    }
    let status = Command::new(TMUX_BIN)
        .args(["run-shell", script.to_string_lossy().as_ref()])
        .status()
        .map_err(|err| format!("failed to run tmux resurrect hook: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux exited with status {status}"))
    }
}

fn choose_session() -> Result<(), String> {
    let status = Command::new(TMUX_BIN)
        .args(["choose-tree", "-Zs"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| format!("failed to open tmux chooser: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux exited with status {status}"))
    }
}

fn attach_chooser() -> Result<(), String> {
    ensure_session(Role::Main)?;
    let status = Command::new(TMUX_BIN)
        .args([
            "attach-session",
            "-t",
            Role::Main.session(),
            ";",
            "choose-tree",
            "-Zs",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| format!("failed to open tmux chooser client: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tmux exited with status {status}"))
    }
}

fn save_state() -> Result<(), String> {
    run_resurrect("save.sh")
}

fn restore_state() -> Result<(), String> {
    run_tmux_checked(["start-server"])?;
    run_optional_resurrect("restore.sh")?;
    ensure_session(Role::Main)?;
    ensure_session(Role::Float)
}

fn usage() -> &'static str {
    "usage: rg-tmux-role {ensure|client|chooser|client-chooser|save|restore|session} [role]"
}

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next();
    let result = match command.as_deref() {
        Some("ensure") => args
            .next()
            .ok_or_else(|| usage().to_string())
            .and_then(|role| Role::from_str(&role))
            .and_then(ensure_session),
        Some("client") => args
            .next()
            .ok_or_else(|| usage().to_string())
            .and_then(|role| Role::from_str(&role))
            .and_then(attach_session),
        Some("chooser") => choose_session(),
        Some("client-chooser") => attach_chooser(),
        Some("save") => save_state(),
        Some("restore") => restore_state(),
        Some("session") => args
            .next()
            .ok_or_else(|| usage().to_string())
            .and_then(|role| Role::from_str(&role))
            .map(|role| {
                println!("{}", role.session());
            }),
        _ => Err(usage().to_string()),
    };

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(64);
    }
}
