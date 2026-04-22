use std::env;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

const FOOTCLIENT_BIN: &str = "/usr/bin/footclient";
const ROFI_BIN: &str = "/usr/bin/rofi";
const SWAYMSG_BIN: &str = "/usr/bin/swaymsg";

#[derive(Clone, Copy)]
enum Action {
    Workspace(&'static str),
    WorkspaceMenu,
    TmuxChooser,
    TmuxSave,
    TmuxRestore,
    AwakeToggle,
    BoostToggle,
    FocusToggle,
    AwakeTimed,
    Lock,
}

struct Entry {
    label: &'static str,
    action: Action,
}

fn home_dir() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn runtime_dir() -> Result<PathBuf, String> {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "XDG_RUNTIME_DIR is not set".to_string())
}

fn local_bin(name: &str) -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".local/bin").join(name))
}

fn entries() -> Vec<Entry> {
    vec![
        Entry {
            label: "Read workspace           [$mod+g r]",
            action: Action::Workspace("7:\u{f02d} read"),
        },
        Entry {
            label: "Notes workspace          [$mod+g n]",
            action: Action::Workspace("4:\u{f044} note"),
        },
        Entry {
            label: "Files workspace          [$mod+g f]",
            action: Action::Workspace("6:\u{f07b} file"),
        },
        Entry {
            label: "Workspace chooser        [$mod+/]",
            action: Action::WorkspaceMenu,
        },
        Entry {
            label: "Tmux chooser             [$mod+F1]",
            action: Action::TmuxChooser,
        },
        Entry {
            label: "Tmux save                [$mod+F5]",
            action: Action::TmuxSave,
        },
        Entry {
            label: "Tmux restore             [$mod+F6]",
            action: Action::TmuxRestore,
        },
        Entry {
            label: "Awake toggle             [$mod+Shift+F3]",
            action: Action::AwakeToggle,
        },
        Entry {
            label: "Boost toggle             [$mod+Ctrl+F3]",
            action: Action::BoostToggle,
        },
        Entry {
            label: "Focus toggle             [$mod+Ctrl+Shift+F3]",
            action: Action::FocusToggle,
        },
        Entry {
            label: "Awake for 2h             [$mod+Alt+Shift+F3]",
            action: Action::AwakeTimed,
        },
        Entry {
            label: "Lock screen              [$mod+Escape]",
            action: Action::Lock,
        },
    ]
}

fn choose_entry(entries: &[Entry]) -> Result<Option<usize>, String> {
    let input = entries
        .iter()
        .map(|entry| entry.label)
        .collect::<Vec<_>>()
        .join("\n");
    let mut rofi = Command::new(ROFI_BIN)
        .args([
            "-dmenu",
            "-i",
            "-p",
            "Desktop",
            "-format",
            "i",
            "-theme-str",
            "window { width: 46%; } listview { lines: 12; }",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to start rofi: {err}"))?;

    {
        let stdin = rofi
            .stdin
            .as_mut()
            .ok_or_else(|| "rofi stdin unavailable".to_string())?;
        use std::io::Write;
        stdin
            .write_all(input.as_bytes())
            .map_err(|err| format!("failed writing menu to rofi: {err}"))?;
    }

    let output = rofi
        .wait_with_output()
        .map_err(|err| format!("failed waiting for rofi: {err}"))?;
    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8(output.stdout).map_err(|err| format!("invalid UTF-8 from rofi: {err}"))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let index = trimmed
        .parse::<usize>()
        .map_err(|err| format!("invalid rofi selection index: {err}"))?;
    Ok(Some(index))
}

fn run_action(action: Action) -> Result<(), String> {
    match action {
        Action::Workspace(name) => {
            run_command(SWAYMSG_BIN, &["workspace", "number", name])
        }
        Action::WorkspaceMenu => run_command_path(local_bin("rg-workspace-menu")?, &[]),
        Action::TmuxChooser => {
            let socket = runtime_dir()?.join("foot-server.sock");
            let helper = local_bin("rg-tmux-role")?;
            run_command_path(
                PathBuf::from(FOOTCLIENT_BIN),
                &[
                    "--server-socket",
                    socket.to_string_lossy().as_ref(),
                    "--app-id",
                    "tmuxChooser",
                    "-T",
                    "tmux-chooser",
                    "-e",
                    helper.to_string_lossy().as_ref(),
                    "client-chooser",
                ],
            )
        }
        Action::TmuxSave => run_command_path(local_bin("rg-tmux-role")?, &["save"]),
        Action::TmuxRestore => run_command_path(local_bin("rg-tmux-role")?, &["restore"]),
        Action::AwakeToggle => run_command_path(local_bin("rg-caffeine")?, &["toggle"]),
        Action::BoostToggle => run_command_path(local_bin("rg-caffeine")?, &["performance-toggle"]),
        Action::FocusToggle => run_command_path(local_bin("rg-caffeine")?, &["focus-toggle"]),
        Action::AwakeTimed => run_command_path(local_bin("rg-caffeine")?, &["timed", "2h"]),
        Action::Lock => run_command_path(local_bin("rg-lockscreen")?, &["--daemonize"]),
    }
}

fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    run_command_path(PathBuf::from(program), args)
}

fn run_command_path(program: PathBuf, args: &[&str]) -> Result<(), String> {
    let status = Command::new(&program)
        .args(args)
        .status()
        .map_err(|err| format!("failed to run {}: {err}", program.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with status {status}", program.display()))
    }
}

fn main() {
    let entries = entries();
    let result = choose_entry(&entries)
        .and_then(|selection| match selection {
            Some(index) => entries
                .get(index)
                .ok_or_else(|| format!("selection out of range: {index}"))
                .and_then(|entry| run_action(entry.action)),
            None => Ok(()),
        });

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(1);
    }
}
