use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

const FOOTCLIENT_BIN: &str = "/usr/bin/footclient";
const ROFI_BIN: &str = "/usr/bin/rofi";
const SWAYMSG_BIN: &str = "/usr/bin/swaymsg";
const HERDR_BIN: &str = "/usr/bin/herdr";

#[derive(Clone, Copy)]
enum Action {
    Workspace(&'static str),
    WorkspaceMenu,
    TmuxChooser,
    TmuxSave,
    TmuxRestore,
    HerdrOpen,
    HerdrSessionPicker,
    AwakeToggle,
    BoostToggle,
    FocusToggle,
    AwakeTimed,
    ThemeToggle,
    ThemePick,
    ThemeApply,
    Lock,
    PowerMenu,
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

fn path_is_usable(path: &PathBuf) -> bool {
    path.is_file()
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
    })
}

fn herdr_bin() -> Option<PathBuf> {
    let fixed = PathBuf::from(HERDR_BIN);
    if path_is_usable(&fixed) {
        return Some(fixed);
    }
    which_on_path("herdr")
}

fn theme_bin() -> Option<PathBuf> {
    local_bin("rg-theme")
        .ok()
        .filter(path_is_usable)
        .or_else(|| which_on_path("rg-theme"))
}

fn theme_status_suffix() -> String {
    let Some(bin) = theme_bin() else {
        return String::new();
    };
    let output = Command::new(bin).arg("status").output().ok();
    let Some(output) = output else {
        return String::new();
    };
    if !output.status.success() {
        return String::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("  [{trimmed}]")
    }
}

fn entries() -> Vec<Entry> {
    let mut out = vec![
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
            label: "Web workspace            [$mod+g w]",
            action: Action::Workspace("2:\u{f269} web"),
        },
        Entry {
            label: "Code workspace           [$mod+g c]",
            action: Action::Workspace("3:\u{f121} code"),
        },
        Entry {
            label: "Media workspace          [$mod+g m]",
            action: Action::Workspace("8:\u{f001} media"),
        },
        Entry {
            label: "Sys workspace            [$mod+g s]",
            action: Action::Workspace("10:\u{f013} sys"),
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
    ];

    if herdr_bin().is_some() {
        out.push(Entry {
            label: "Herdr open / attach",
            action: Action::HerdrOpen,
        });
        out.push(Entry {
            label: "Herdr session picker",
            action: Action::HerdrSessionPicker,
        });
    }

    out.extend([
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
    ]);

    if theme_bin().is_some() {
        out.extend([
            Entry {
                label: "Theme toggle light/dark  [$mod+Shift+F1]",
                action: Action::ThemeToggle,
            },
            Entry {
                label: "Theme palette picker     [$mod+Shift+F2]",
                action: Action::ThemePick,
            },
            Entry {
                label: "Theme re-apply",
                action: Action::ThemeApply,
            },
        ]);
    }

    out.extend([
        Entry {
            label: "Lock screen              [$mod+Escape]",
            action: Action::Lock,
        },
        Entry {
            label: "Power / profiles menu    [$mod+Shift+Escape]",
            action: Action::PowerMenu,
        },
    ]);

    out
}

/// Static labels used by `--print-menu` / selftests (no runtime probing).
fn print_menu_labels() -> Vec<&'static str> {
    vec![
        "Read workspace           [$mod+g r]",
        "Notes workspace          [$mod+g n]",
        "Files workspace          [$mod+g f]",
        "Web workspace            [$mod+g w]",
        "Code workspace           [$mod+g c]",
        "Media workspace          [$mod+g m]",
        "Sys workspace            [$mod+g s]",
        "Workspace chooser        [$mod+/]",
        "Tmux chooser             [$mod+F1]",
        "Tmux save                [$mod+F5]",
        "Tmux restore             [$mod+F6]",
        "Herdr open / attach",
        "Herdr session picker",
        "Awake toggle             [$mod+Shift+F3]",
        "Boost toggle             [$mod+Ctrl+F3]",
        "Focus toggle             [$mod+Ctrl+Shift+F3]",
        "Awake for 2h             [$mod+Alt+Shift+F3]",
        "Theme toggle light/dark  [$mod+Shift+F1]",
        "Theme palette picker     [$mod+Shift+F2]",
        "Theme re-apply",
        "Lock screen              [$mod+Escape]",
        "Power / profiles menu    [$mod+Shift+Escape]",
    ]
}

fn choose_index(prompt: &str, labels: &[String], lines: usize) -> Result<Option<usize>, String> {
    let input = labels.join("\n");
    let theme_str = format!("window {{ width: 50%; }} listview {{ lines: {lines}; }}");
    let mut rofi = Command::new(ROFI_BIN)
        .args([
            "-dmenu",
            "-i",
            "-p",
            prompt,
            "-format",
            "i",
            "-theme-str",
            &theme_str,
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

fn foot_run(app_id: &str, title: &str, program: &PathBuf, args: &[&str]) -> Result<(), String> {
    let socket = runtime_dir()?.join("foot-server.sock");
    let mut cmd_args: Vec<String> = vec![
        "--server-socket".into(),
        socket.to_string_lossy().into_owned(),
        "--app-id".into(),
        app_id.into(),
        "-T".into(),
        title.into(),
        "-e".into(),
        program.to_string_lossy().into_owned(),
    ];
    for arg in args {
        cmd_args.push((*arg).to_string());
    }
    let arg_refs: Vec<&str> = cmd_args.iter().map(String::as_str).collect();
    run_command_path(PathBuf::from(FOOTCLIENT_BIN), &arg_refs)
}

fn list_herdr_sessions() -> Result<Vec<String>, String> {
    let bin = herdr_bin().ok_or_else(|| "herdr not found".to_string())?;
    let output = Command::new(&bin)
        .args(["session", "list", "--json"])
        .output()
        .map_err(|err| format!("herdr session list failed: {err}"))?;
    if !output.status.success() {
        // Fallback: plain table
        let plain = Command::new(&bin)
            .args(["session", "list"])
            .output()
            .map_err(|err| format!("herdr session list failed: {err}"))?;
        if !plain.status.success() {
            return Err(format!(
                "herdr session list exited with status {}",
                plain.status
            ));
        }
        let text = String::from_utf8_lossy(&plain.stdout);
        let mut names = Vec::new();
        for line in text.lines().skip(1) {
            let name = line.split_whitespace().next().unwrap_or("");
            if !name.is_empty() && name != "name" {
                names.push(name.to_string());
            }
        }
        return Ok(names);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    // Minimal JSON parse without a dependency: find "name":"..."
    let mut names = Vec::new();
    let bytes = text.as_bytes();
    let key = b"\"name\"";
    let mut i = 0;
    while i + key.len() < bytes.len() {
        if &bytes[i..i + key.len()] == key {
            i += key.len();
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b':' || bytes[i] == b'\t') {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'"' {
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                if i <= bytes.len() {
                    let name = text[start..i].to_string();
                    if !name.is_empty() {
                        names.push(name);
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}

fn herdr_session_picker() -> Result<(), String> {
    let sessions = list_herdr_sessions()?;
    if sessions.is_empty() {
        // No named sessions yet: just open the default.
        return herdr_open(None);
    }

    let mut labels: Vec<String> = sessions
        .iter()
        .map(|name| {
            if name == "default" {
                format!("{name}  (default)")
            } else {
                name.clone()
            }
        })
        .collect();
    labels.push("New / type session name…".into());

    let index = match choose_index("herdr session", &labels, labels.len().clamp(4, 16))? {
        Some(i) => i,
        None => return Ok(()),
    };

    if index == labels.len() - 1 {
        // Free-form name via rofi.
        let output = Command::new(ROFI_BIN)
            .args(["-dmenu", "-i", "-p", "herdr session name"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .output()
            .map_err(|err| format!("failed to start rofi: {err}"))?;
        if !output.status.success() {
            return Ok(());
        }
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            return Ok(());
        }
        return herdr_open(Some(&name));
    }

    let name = sessions
        .get(index)
        .ok_or_else(|| format!("selection out of range: {index}"))?;
    herdr_open(Some(name))
}

fn herdr_open(session: Option<&str>) -> Result<(), String> {
    let bin = herdr_bin().ok_or_else(|| "herdr not found".to_string())?;
    match session {
        None | Some("default") => foot_run("herdr", "herdr", &bin, &[]),
        Some(name) => foot_run("herdr", &format!("herdr:{name}"), &bin, &["--session", name]),
    }
}

fn run_action(action: Action) -> Result<(), String> {
    match action {
        Action::Workspace(name) => run_command(SWAYMSG_BIN, &["workspace", "number", name]),
        Action::WorkspaceMenu => run_command_path(local_bin("rg-workspace-menu")?, &[]),
        Action::TmuxChooser => {
            let helper = local_bin("rg-tmux-role")?;
            foot_run(
                "tmuxChooser",
                "tmux-chooser",
                &helper,
                &["client-chooser"],
            )
        }
        Action::TmuxSave => run_command_path(local_bin("rg-tmux-role")?, &["save"]),
        Action::TmuxRestore => run_command_path(local_bin("rg-tmux-role")?, &["restore"]),
        Action::HerdrOpen => herdr_open(None),
        Action::HerdrSessionPicker => herdr_session_picker(),
        Action::AwakeToggle => run_command_path(local_bin("rg-caffeine")?, &["toggle"]),
        Action::BoostToggle => {
            run_command_path(local_bin("rg-caffeine")?, &["performance-toggle"])
        }
        Action::FocusToggle => run_command_path(local_bin("rg-caffeine")?, &["focus-toggle"]),
        Action::AwakeTimed => run_command_path(local_bin("rg-caffeine")?, &["timed", "2h"]),
        Action::ThemeToggle => {
            let bin = theme_bin().ok_or_else(|| "rg-theme not found".to_string())?;
            run_command_path(bin, &["toggle"])
        }
        Action::ThemePick => {
            let bin = theme_bin().ok_or_else(|| "rg-theme not found".to_string())?;
            run_command_path(bin, &["pick"])
        }
        Action::ThemeApply => {
            let bin = theme_bin().ok_or_else(|| "rg-theme not found".to_string())?;
            run_command_path(bin, &["apply"])
        }
        Action::Lock => run_command_path(local_bin("rg-lockscreen")?, &["--daemonize"]),
        Action::PowerMenu => run_command_path(local_bin("rg-power-menu")?, &[]),
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

fn usage() {
    eprintln!(
        "usage: rg-desktop-help [--print-menu]\n\n\
         Keyboard-first desktop control palette (rofi).\n\
         --print-menu   print static entry labels (for selftests)"
    );
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("-h") | Some("--help") => {
            usage();
            return;
        }
        Some("--print-menu") => {
            for label in print_menu_labels() {
                println!("{label}");
            }
            return;
        }
        Some(other) => {
            eprintln!("unknown argument: {other}");
            usage();
            process::exit(2);
        }
        None => {}
    }

    let prompt_suffix = theme_status_suffix();
    let entries = entries();
    let labels: Vec<String> = entries.iter().map(|e| e.label.to_string()).collect();
    let prompt = if prompt_suffix.is_empty() {
        "Desktop".to_string()
    } else {
        format!("Desktop{prompt_suffix}")
    };

    let result = choose_index(&prompt, &labels, labels.len().clamp(8, 22)).and_then(|selection| {
        match selection {
            Some(index) => entries
                .get(index)
                .ok_or_else(|| format!("selection out of range: {index}"))
                .and_then(|entry| run_action(entry.action)),
            None => Ok(()),
        }
    });

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(1);
    }
}
