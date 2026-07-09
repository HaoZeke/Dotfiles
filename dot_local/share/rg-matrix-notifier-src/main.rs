use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_SYNC_LIMIT: u64 = 20;
const DEFAULT_BODY_MAX_CHARS: usize = 160;

#[derive(Debug, Default, Deserialize)]
struct Config {
    user_id: Option<String>,
    homeserver: Option<String>,
    timeout_ms: Option<u64>,
    retry_seconds: Option<u64>,
    sync_limit: Option<u64>,
    body_max_chars: Option<usize>,
    auth: Option<AuthConfig>,
    sink: Option<SinkConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct AuthConfig {
    homeserver: Option<String>,
    access_token_env: Option<String>,
    access_token_file: Option<String>,
    access_token_command: Option<Vec<String>>,
    access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SinkConfig {
    backend: Option<String>,
    command: Option<Vec<String>>,
    urgency: Option<String>,
}

impl Default for SinkConfig {
    fn default() -> Self {
        Self {
            backend: Some("notify-send".to_string()),
            command: None,
            urgency: Some("normal".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NormalizedEvent {
    backend: &'static str,
    event_id: String,
    event_type: String,
    room_id: String,
    room_name: String,
    sender: String,
    body: String,
    origin_server_ts: Option<i64>,
}

#[derive(Debug)]
struct Paths {
    config: PathBuf,
    state_dir: PathBuf,
}

fn xdg_config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
}

fn xdg_state_home() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/state"))
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn default_config_path() -> PathBuf {
    xdg_config_home().join("rg-matrix-notifier/config.toml")
}

fn default_state_dir() -> PathBuf {
    xdg_state_home().join("rg-matrix-notifier")
}

fn read_config(path: &Path) -> Result<Config, String> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    toml::from_str(&text).map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn compact_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|err| format!("failed to encode json: {err}"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path, fallback: T) -> T {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or(fallback),
        Err(_) => fallback,
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let tmp = path.with_extension("tmp");
    let text = compact_json(value)?;
    fs::write(&tmp, format!("{text}\n"))
        .map_err(|err| format!("failed to write {}: {err}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|err| format!("failed to replace {}: {err}", path.display()))
}

fn truncate_text(text: &str, limit: usize) -> String {
    let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if limit == 0 || clean.chars().count() <= limit {
        return clean;
    }
    clean.chars().take(limit).collect()
}

fn room_name(room_id: &str, room: &Value, rooms_state: &mut HashMap<String, String>) -> String {
    for section in ["state", "timeline"] {
        if let Some(events) = room
            .get(section)
            .and_then(|v| v.get("events"))
            .and_then(Value::as_array)
        {
            for event in events {
                if event.get("type").and_then(Value::as_str) == Some("m.room.name") {
                    if let Some(name) = event
                        .get("content")
                        .and_then(|v| v.get("name"))
                        .and_then(Value::as_str)
                        .filter(|name| !name.is_empty())
                    {
                        rooms_state.insert(room_id.to_string(), name.to_string());
                        return name.to_string();
                    }
                }
            }
        }
    }
    rooms_state
        .get(room_id)
        .cloned()
        .unwrap_or_else(|| room_id.to_string())
}

fn event_body(event: &Value, max_chars: usize) -> Option<String> {
    match event.get("type").and_then(Value::as_str) {
        Some("m.room.encrypted") => Some("Encrypted Matrix message".to_string()),
        Some("m.room.message") => event
            .get("content")
            .and_then(|v| v.get("body"))
            .and_then(Value::as_str)
            .filter(|body| !body.is_empty())
            .map(|body| truncate_text(body, max_chars)),
        _ => None,
    }
}

fn normalize_events(
    sync: &Value,
    config: &Config,
    rooms_state: &mut HashMap<String, String>,
) -> Vec<NormalizedEvent> {
    let max_chars = config.body_max_chars.unwrap_or(DEFAULT_BODY_MAX_CHARS);
    let Some(joined) = sync
        .get("rooms")
        .and_then(|v| v.get("join"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (room_id, room) in joined {
        let label = room_name(room_id, room, rooms_state);
        let Some(events) = room
            .get("timeline")
            .and_then(|v| v.get("events"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for event in events {
            let Some(event_id) = event.get("event_id").and_then(Value::as_str) else {
                continue;
            };
            let Some(sender) = event.get("sender").and_then(Value::as_str) else {
                continue;
            };
            if config.user_id.as_deref() == Some(sender) {
                continue;
            }
            let Some(body) = event_body(event, max_chars) else {
                continue;
            };
            out.push(NormalizedEvent {
                backend: "matrix",
                event_id: event_id.to_string(),
                event_type: event
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                room_id: room_id.to_string(),
                room_name: label.clone(),
                sender: sender.to_string(),
                body,
                origin_server_ts: event.get("origin_server_ts").and_then(Value::as_i64),
            });
        }
    }
    out
}

fn sink_config(config: &Config) -> SinkConfig {
    config.sink.as_ref().cloned().unwrap_or_default()
}

impl Clone for SinkConfig {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            command: self.command.clone(),
            urgency: self.urgency.clone(),
        }
    }
}

fn emit_event(event: &NormalizedEvent, config: &Config) -> Result<(), String> {
    let sink = sink_config(config);
    match sink.backend.as_deref().unwrap_or("notify-send") {
        "stdout" => {
            println!("{}", compact_json(event)?);
            Ok(())
        }
        "command" => {
            let command = sink
                .command
                .ok_or_else(|| "command sink requires sink.command".to_string())?;
            run_command_sink(event, &command)
        }
        "notify-send" => run_notify_send(event, &sink),
        other => Err(format!("unknown sink backend: {other}")),
    }
}

fn run_command_sink(event: &NormalizedEvent, command: &[String]) -> Result<(), String> {
    if command.is_empty() {
        return Err("empty sink.command".to_string());
    }
    let payload = compact_json(event)?;
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .env("RG_MATRIX_EVENT_JSON", &payload)
        .env("RG_MATRIX_EVENT_ID", &event.event_id)
        .env("RG_MATRIX_ROOM_ID", &event.room_id)
        .env("RG_MATRIX_ROOM_NAME", &event.room_name)
        .env("RG_MATRIX_SENDER", &event.sender)
        .env("RG_MATRIX_BODY", &event.body)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to run sink command {}: {err}", command[0]))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(payload.as_bytes())
            .map_err(|err| format!("failed to write event to sink: {err}"))?;
    }
    let status = child
        .wait()
        .map_err(|err| format!("failed waiting on sink command: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("sink command exited with status {status}"))
    }
}

fn run_notify_send(event: &NormalizedEvent, sink: &SinkConfig) -> Result<(), String> {
    let command = sink
        .command
        .clone()
        .unwrap_or_else(|| vec!["notify-send".to_string()]);
    if command.is_empty() {
        return Err("empty notify-send command".to_string());
    }
    let urgency = sink.urgency.as_deref().unwrap_or("normal");
    let body = format!("{}: {}", event.sender, event.body);
    let status = Command::new(&command[0])
        .args(&command[1..])
        .args(["-a", "matrix", "-u", urgency, &event.room_name, &body])
        .status()
        .map_err(|err| format!("failed to run notify-send sink {}: {err}", command[0]))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("notify-send sink exited with status {status}"))
    }
}

fn process_sync(sync: &Value, config: &Config, state_dir: &Path) -> Result<usize, String> {
    fs::create_dir_all(state_dir)
        .map_err(|err| format!("failed to create {}: {err}", state_dir.display()))?;
    let seen_path = state_dir.join("seen.json");
    let rooms_path = state_dir.join("rooms.json");
    let status_path = state_dir.join("status.json");
    let mut seen: BTreeSet<String> = read_json(&seen_path, BTreeSet::new());
    let mut rooms_state: HashMap<String, String> = read_json(&rooms_path, HashMap::new());
    let mut emitted = 0usize;

    for event in normalize_events(sync, config, &mut rooms_state) {
        if seen.contains(&event.event_id) {
            continue;
        }
        emit_event(&event, config)?;
        seen.insert(event.event_id.clone());
        emitted += 1;
    }

    if let Some(next_batch) = sync.get("next_batch").and_then(Value::as_str) {
        write_json(
            &state_dir.join("since.json"),
            &serde_json::json!({ "next_batch": next_batch }),
        )?;
    }
    let retained = seen.into_iter().rev().take(1000).collect::<BTreeSet<_>>();
    write_json(&seen_path, &retained)?;
    write_json(&rooms_path, &rooms_state)?;
    write_json(
        &status_path,
        &serde_json::json!({ "state": "Good", "text": "Mx", "short_text": "Mx", "emitted": emitted }),
    )?;
    Ok(emitted)
}

fn access_token(config: &Config) -> Result<String, String> {
    let Some(auth) = config.auth.as_ref() else {
        return Err("missing [auth] access token provider".to_string());
    };
    if let Some(name) = auth.access_token_env.as_deref() {
        if let Ok(value) = env::var(name) {
            if !value.trim().is_empty() {
                return Ok(value.trim().to_string());
            }
        }
    }
    if let Some(path) = auth.access_token_file.as_deref() {
        let text = fs::read_to_string(expand_home(path))
            .map_err(|err| format!("failed to read access token file: {err}"))?;
        if !text.trim().is_empty() {
            return Ok(text.trim().to_string());
        }
    }
    if let Some(command) = auth.access_token_command.as_ref() {
        if command.is_empty() {
            return Err("empty auth.access_token_command".to_string());
        }
        let output = Command::new(&command[0])
            .args(&command[1..])
            .output()
            .map_err(|err| format!("failed to run access token command {}: {err}", command[0]))?;
        if !output.status.success() {
            return Err(format!(
                "access token command exited with status {}",
                output.status
            ));
        }
        let token = String::from_utf8(output.stdout)
            .map_err(|err| format!("access token command emitted invalid UTF-8: {err}"))?;
        if !token.trim().is_empty() {
            return Ok(token.trim().to_string());
        }
    }
    if let Some(token) = auth.access_token.as_deref() {
        if !token.trim().is_empty() {
            return Ok(token.trim().to_string());
        }
    }
    Err("missing access token".to_string())
}

fn homeserver(config: &Config) -> Result<String, String> {
    config
        .homeserver
        .as_deref()
        .or_else(|| {
            config
                .auth
                .as_ref()
                .and_then(|auth| auth.homeserver.as_deref())
        })
        .map(|server| server.trim_end_matches('/').to_string())
        .filter(|server| !server.is_empty())
        .ok_or_else(|| "missing Matrix homeserver".to_string())
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(path)
    }
}

fn sync_request(config: &Config, state_dir: &Path) -> Result<Value, String> {
    let server = homeserver(config)?;
    let token = access_token(config)?;
    let since: Value = read_json(&state_dir.join("since.json"), Value::Null);
    let filter = serde_json::json!({
        "room": {
            "timeline": { "limit": config.sync_limit.unwrap_or(DEFAULT_SYNC_LIMIT) }
        }
    });
    let timeout = config.timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS).to_string();
    let mut request = ureq::get(&format!("{server}/_matrix/client/v3/sync"))
        .query("timeout", &timeout)
        .query("filter", &filter.to_string())
        .header("Authorization", &format!("Bearer {token}"));
    if let Some(next_batch) = since.get("next_batch").and_then(Value::as_str) {
        request = request.query("since", next_batch);
    }
    let text = request
        .call()
        .map_err(|err| format!("Matrix sync failed: {err}"))?
        .body_mut()
        .read_to_string()
        .map_err(|err| format!("failed to read Matrix sync response: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid Matrix sync JSON: {err}"))
}

fn run_loop(config: &Config, state_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(state_dir)
        .map_err(|err| format!("failed to create {}: {err}", state_dir.display()))?;
    if homeserver(config).is_err() || config.auth.is_none() {
        write_json(
            &state_dir.join("status.json"),
            &serde_json::json!({ "state": "Warning", "text": "Mx setup", "short_text": "Mx!" }),
        )?;
        eprintln!("rg-matrix-notifier: Matrix homeserver/auth is not configured");
        return Ok(());
    }
    let retry = Duration::from_secs(config.retry_seconds.unwrap_or(30));
    loop {
        match sync_request(config, state_dir)
            .and_then(|sync| process_sync(&sync, config, state_dir))
        {
            Ok(_) => {}
            Err(err) => {
                write_json(
                    &state_dir.join("status.json"),
                    &serde_json::json!({ "state": "Warning", "text": "Mx auth/net", "short_text": "Mx!" }),
                )?;
                eprintln!("rg-matrix-notifier: {err}");
                thread::sleep(retry);
            }
        }
    }
}

fn status_json(state_dir: &Path) -> Result<(), String> {
    let status: Value = read_json(&state_dir.join("status.json"), Value::Null);
    if status.is_object() {
        println!("{}", compact_json(&status)?);
    } else {
        println!(
            "{}",
            compact_json(&serde_json::json!({
                "state": "Warning",
                "text": "Mx setup",
                "short_text": "Mx!"
            }))?
        );
    }
    Ok(())
}

fn parse_args() -> Result<(Paths, String, Vec<String>), String> {
    let mut config = default_config_path();
    let mut state_dir = default_state_dir();
    let mut rest = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--config requires a path".to_string())?;
                config = PathBuf::from(value);
            }
            "--state-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--state-dir requires a path".to_string())?;
                state_dir = PathBuf::from(value);
            }
            "-h" | "--help" => {
                return Ok((Paths { config, state_dir }, "help".to_string(), Vec::new()));
            }
            _ => {
                rest.push(arg);
                rest.extend(args);
                break;
            }
        }
    }
    let command = rest
        .first()
        .cloned()
        .unwrap_or_else(|| "status-json".to_string());
    let command_args = rest.into_iter().skip(1).collect();
    Ok((Paths { config, state_dir }, command, command_args))
}

fn print_usage() {
    eprintln!(
        "usage: rg-matrix-notifier [--config PATH] [--state-dir PATH] <status-json|process-sync PATH|run>"
    );
}

fn run() -> Result<(), String> {
    let (paths, command, args) = parse_args()?;
    if command == "help" {
        print_usage();
        return Ok(());
    }
    let config = read_config(&paths.config)?;
    match command.as_str() {
        "status-json" => status_json(&paths.state_dir),
        "process-sync" => {
            let path = args
                .first()
                .ok_or_else(|| "process-sync requires a sync JSON path".to_string())?;
            let mut text = String::new();
            fs::File::open(path)
                .map_err(|err| format!("failed to open {path}: {err}"))?
                .read_to_string(&mut text)
                .map_err(|err| format!("failed to read {path}: {err}"))?;
            let sync: Value =
                serde_json::from_str(&text).map_err(|err| format!("invalid sync JSON: {err}"))?;
            process_sync(&sync, &config, &paths.state_dir).map(|_| ())
        }
        "run" => run_loop(&config, &paths.state_dir),
        other => Err(format!("unknown command: {other}")),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            user_id: Some("@me:example.org".to_string()),
            body_max_chars: Some(18),
            sink: Some(SinkConfig {
                backend: Some("stdout".to_string()),
                command: None,
                urgency: None,
            }),
            ..Config::default()
        }
    }

    #[test]
    fn normalizes_backend_agnostic_events() {
        let sync: Value = serde_json::json!({
            "rooms": {
                "join": {
                    "!room:example.org": {
                        "state": {
                            "events": [
                                { "type": "m.room.name", "content": { "name": "Ops Room" } }
                            ]
                        },
                        "timeline": {
                            "events": [
                                {
                                    "type": "m.room.message",
                                    "event_id": "$own",
                                    "sender": "@me:example.org",
                                    "content": { "body": "self message" }
                                },
                                {
                                    "type": "m.room.message",
                                    "event_id": "$other",
                                    "sender": "@alice:example.org",
                                    "origin_server_ts": 2,
                                    "content": { "body": "hello from a backend agnostic notifier" }
                                },
                                {
                                    "type": "m.room.encrypted",
                                    "event_id": "$secret",
                                    "sender": "@bob:example.org",
                                    "origin_server_ts": 3,
                                    "content": {}
                                }
                            ]
                        }
                    }
                }
            }
        });
        let mut rooms = HashMap::new();
        let events = normalize_events(&sync, &config(), &mut rooms);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].backend, "matrix");
        assert_eq!(events[0].event_id, "$other");
        assert_eq!(events[0].room_name, "Ops Room");
        assert_eq!(events[0].body, "hello from a backe");
        assert_eq!(events[1].event_id, "$secret");
        assert_eq!(events[1].body, "Encrypted Matrix message");
    }

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate_text("héllo world", 5), "héllo");
    }
}
