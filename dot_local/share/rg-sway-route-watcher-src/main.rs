use serde::Deserialize;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

const SWAYMSG_BIN: &str = "/usr/bin/swaymsg";
const NOTIFY_SEND_BIN: &str = "/usr/bin/notify-send";

#[derive(Debug, Deserialize)]
struct WindowEvent {
    change: Option<String>,
    container: Option<EventContainer>,
}

#[derive(Debug, Deserialize)]
struct EventContainer {
    id: i64,
    app_id: Option<String>,
    window_properties: Option<WindowProperties>,
}

#[derive(Debug, Deserialize)]
struct WindowProperties {
    class: Option<String>,
}

fn command_stdout(args: &[&str]) -> Result<String, String> {
    let output = Command::new(SWAYMSG_BIN)
        .args(args)
        .output()
        .map_err(|err| format!("failed to run swaymsg {:?}: {err}", args))?;
    if !output.status.success() {
        return Err(format!(
            "swaymsg {:?} exited with status {}",
            args, output.status
        ));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("invalid UTF-8 from swaymsg: {err}"))
}

fn focused_workspace() -> Result<String, String> {
    let text = command_stdout(&["-t", "get_workspaces"])?;
    let workspaces: Value =
        serde_json::from_str(&text).map_err(|err| format!("invalid workspace json: {err}"))?;
    workspaces
        .as_array()
        .and_then(|items| {
            items.iter().find_map(|item| {
                let focused = item.get("focused").and_then(Value::as_bool).unwrap_or(false);
                if focused {
                    item.get("name").and_then(Value::as_str).map(str::to_string)
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| "no focused workspace found".to_string())
}

fn find_workspace_name(node: &Value, target_id: i64, current_workspace: Option<&str>) -> Option<String> {
    let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();
    let workspace_here = if node_type == "workspace" {
        node.get("name").and_then(Value::as_str)
    } else {
        current_workspace
    };

    if node.get("id").and_then(Value::as_i64) == Some(target_id) {
        return workspace_here.map(str::to_string);
    }

    for key in ["nodes", "floating_nodes"] {
        if let Some(children) = node.get(key).and_then(Value::as_array) {
            for child in children {
                if let Some(name) = find_workspace_name(child, target_id, workspace_here) {
                    return Some(name);
                }
            }
        }
    }

    None
}

fn workspace_for_container(target_id: i64) -> Result<Option<String>, String> {
    let text = command_stdout(&["-t", "get_tree"])?;
    let tree: Value =
        serde_json::from_str(&text).map_err(|err| format!("invalid tree json: {err}"))?;
    Ok(find_workspace_name(&tree, target_id, None))
}

fn app_label(app: &str) -> Option<&'static str> {
    match app {
        "mS" | "subFloat" | "tmuxChooser" => None,
        "org.pwmt.zathura" | "Zathura" | "org.gnome.Evince" | "viewnior" | "Viewnior" => {
            Some("PDF")
        }
        "firefox" | "Firefox" | "librewolf" | "LibreWolf" => Some("Web"),
        "obsidian" | "Obsidian" => Some("Notes"),
        "thunar" | "Thunar" => Some("Files"),
        "code" | "Code" | "codium" | "VSCodium" | "emacs" | "Emacs" => Some("Code"),
        "discord" | "Signal" | "signal" | "TelegramDesktop" | "org.telegram.desktop" => {
            Some("Chat")
        }
        _ => None,
    }
}

fn notify_route(label: &str, workspace: &str) -> Result<(), String> {
    let status = Command::new(NOTIFY_SEND_BIN)
        .args([
            "-a",
            "sway",
            "-u",
            "low",
            "-t",
            "1600",
            "-h",
            "string:x-dunst-stack-tag:rg-route",
            "Routed",
            &format!("{label} -> {workspace}"),
        ])
        .status()
        .map_err(|err| format!("failed to run notify-send: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("notify-send exited with status {status}"))
    }
}

fn handle_event(line: &str) -> Result<(), String> {
    let event: WindowEvent =
        serde_json::from_str(line).map_err(|err| format!("invalid window event json: {err}"))?;
    if event.change.as_deref() != Some("new") {
        return Ok(());
    }
    let Some(container) = event.container else {
        return Ok(());
    };

    let app = container
        .app_id
        .as_deref()
        .or_else(|| {
            container
                .window_properties
                .as_ref()
                .and_then(|props| props.class.as_deref())
        })
        .unwrap_or("app");

    let Some(label) = app_label(app) else {
        return Ok(());
    };

    let current = focused_workspace()?;
    let Some(target) = workspace_for_container(container.id)? else {
        return Ok(());
    };
    if current == target {
        return Ok(());
    }

    notify_route(label, &target)
}

fn ensure_binary(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(format!("missing required binary: {path}"))
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    ensure_binary(SWAYMSG_BIN)?;
    ensure_binary(NOTIFY_SEND_BIN)?;

    let mut child = Command::new(SWAYMSG_BIN)
        .args(["-m", "-t", "subscribe", r#"["window"]"#])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to subscribe to sway window events: {err}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "sway subscription stdout unavailable".to_string())?;
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(text) => {
                if let Err(err) = handle_event(&text) {
                    eprintln!("{err}");
                }
            }
            Err(err) => return Err(format!("failed to read sway event stream: {err}")),
        }
    }

    let status = child
        .wait()
        .map_err(|err| format!("failed waiting on sway subscription: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("sway subscription exited with status {status}"))
    }
}
