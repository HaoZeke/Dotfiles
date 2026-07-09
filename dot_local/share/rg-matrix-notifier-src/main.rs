use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_SYNC_LIMIT: u64 = 20;
const DEFAULT_BODY_MAX_CHARS: usize = 160;
const DEFAULT_HOMESERVER: &str = "https://matrix.surf.nl";
const DEFAULT_USER_ID: &str = "@rohit.goswami:surf.nl";
const DEFAULT_PASS_NAME: &str = "matrix/rg-matrix-notifier/access-token";

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
    open_command: Option<Vec<String>>,
    timeout_ms: Option<u64>,
}

impl Default for SinkConfig {
    fn default() -> Self {
        Self {
            backend: Some("notify-send".to_string()),
            command: None,
            urgency: Some("normal".to_string()),
            open_command: Some(vec!["rg-fractal-toggle".to_string(), "show".to_string()]),
            timeout_ms: Some(8000),
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

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn percent_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut idx = 0;
    while idx < bytes.len() {
        match bytes[idx] {
            b'%' if idx + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[idx + 1..idx + 3])
                    .map_err(|err| format!("invalid percent escape: {err}"))?;
                let value = u8::from_str_radix(hex, 16)
                    .map_err(|err| format!("invalid percent escape %{hex}: {err}"))?;
                out.push(value);
                idx += 3;
            }
            b'+' => {
                out.push(b' ');
                idx += 1;
            }
            byte => {
                out.push(byte);
                idx += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|err| format!("invalid UTF-8 in decoded string: {err}"))
}

fn query_param(path: &str, name: &str) -> Result<Option<String>, String> {
    let Some((_, query)) = path.split_once('?') else {
        return Ok(None);
    };
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(key)? == name {
            return percent_decode(value).map(Some);
        }
    }
    Ok(None)
}

fn login_token_from_request_line(line: &str) -> Result<Option<String>, String> {
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();
    if method != "GET" {
        return Ok(None);
    }
    query_param(path, "loginToken")
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
            open_command: self.open_command.clone(),
            timeout_ms: self.timeout_ms,
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
    let timeout = sink.timeout_ms.unwrap_or(8000).to_string();
    let open_command = sink
        .open_command
        .clone()
        .unwrap_or_else(|| vec!["rg-fractal-toggle".to_string(), "show".to_string()]);
    let body = format!("{}: {}", event.sender, event.body);
    let output = Command::new(&command[0])
        .args(&command[1..])
        .args([
            "-a",
            "matrix",
            "-u",
            urgency,
            "-t",
            &timeout,
            "-A",
            "default=Open",
            &event.room_name,
            &body,
        ])
        .output()
        .map_err(|err| format!("failed to run notify-send sink {}: {err}", command[0]))?;
    if !output.status.success() {
        return Err(format!(
            "notify-send sink exited with status {}",
            output.status
        ));
    }
    let selected = String::from_utf8_lossy(&output.stdout);
    if selected.trim() == "default" {
        run_open_command(&open_command)?;
    }
    Ok(())
}

fn run_open_command(command: &[String]) -> Result<(), String> {
    if command.is_empty() {
        return Ok(());
    }
    let status = Command::new(&command[0])
        .args(&command[1..])
        .status()
        .map_err(|err| format!("failed to run open command {}: {err}", command[0]))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open command exited with status {status}"))
    }
}

fn process_sync(sync: &Value, config: &Config, state_dir: &Path) -> Result<usize, String> {
    fs::create_dir_all(state_dir)
        .map_err(|err| format!("failed to create {}: {err}", state_dir.display()))?;
    let seen_path = state_dir.join("seen.json");
    let rooms_path = state_dir.join("rooms.json");
    let status_path = state_dir.join("status.json");
    let since_path = state_dir.join("since.json");
    let initial_sync = !since_path.exists();
    let mut seen: BTreeSet<String> = read_json(&seen_path, BTreeSet::new());
    let mut rooms_state: HashMap<String, String> = read_json(&rooms_path, HashMap::new());
    let mut emitted = 0usize;

    for event in normalize_events(sync, config, &mut rooms_state) {
        if seen.contains(&event.event_id) {
            continue;
        }
        if !initial_sync {
            emit_event(&event, config)?;
            emitted += 1;
        }
        seen.insert(event.event_id.clone());
    }

    if let Some(next_batch) = sync.get("next_batch").and_then(Value::as_str) {
        write_json(
            &since_path,
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

fn write_http_response(stream: &mut TcpStream, title: &str, body: &str) -> Result<(), String> {
    let page = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>{title}</title><body><h1>{title}</h1><p>{body}</p></body>"
    );
    write!(
        stream,
        "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        page.len(),
        page
    )
    .map_err(|err| format!("failed to write callback response: {err}"))
}

fn wait_for_login_token(listener: TcpListener) -> Result<String, String> {
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("failed to configure callback listener: {err}"))?;
    let deadline = Instant::now() + Duration::from_secs(300);
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0_u8; 4096];
                let len = stream
                    .read(&mut buf)
                    .map_err(|err| format!("failed to read callback request: {err}"))?;
                let request = String::from_utf8_lossy(&buf[..len]);
                let line = request.lines().next().unwrap_or_default();
                if let Some(token) = login_token_from_request_line(line)? {
                    write_http_response(
                        &mut stream,
                        "Matrix notifier login complete",
                        "The notifier received the login token. This browser tab can be closed.",
                    )?;
                    return Ok(token);
                }
                write_http_response(
                    &mut stream,
                    "Matrix notifier login failed",
                    "The callback did not include a Matrix login token.",
                )?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(err) => return Err(format!("failed to accept callback: {err}")),
        }
    }
    Err("timed out waiting for Matrix SSO callback".to_string())
}

fn open_browser(url: &str) -> Result<(), String> {
    let opener = env::var("BROWSER").unwrap_or_else(|_| "xdg-open".to_string());
    let status = Command::new(&opener)
        .arg(url)
        .status()
        .map_err(|err| format!("failed to run browser opener {opener}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "browser opener exited with status {status}; open manually: {url}"
        ))
    }
}

fn exchange_login_token(homeserver: &str, login_token: &str) -> Result<Value, String> {
    let body = serde_json::json!({
        "type": "m.login.token",
        "token": login_token,
        "initial_device_display_name": "rg-matrix-notifier"
    });
    let text = ureq::post(&format!("{homeserver}/_matrix/client/v3/login"))
        .content_type("application/json")
        .send(body.to_string())
        .map_err(|err| format!("failed to exchange Matrix login token: {err}"))?
        .body_mut()
        .read_to_string()
        .map_err(|err| format!("failed to read Matrix login response: {err}"))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid Matrix login response: {err}"))
}

fn store_pass_secret(pass_name: &str, secret: &str) -> Result<(), String> {
    let mut child = Command::new("pass")
        .args(["insert", "--force", "--multiline", pass_name])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to run pass insert: {err}"))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(secret.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .map_err(|err| format!("failed to write token to pass: {err}"))?;
    }
    let status = child
        .wait()
        .map_err(|err| format!("failed waiting on pass insert: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("pass insert exited with status {status}"))
    }
}

fn config_text(homeserver: &str, user_id: &str, pass_name: &str) -> String {
    format!(
        r#"user_id = "{user_id}"
homeserver = "{homeserver}"
body_max_chars = 160
timeout_ms = 30000
retry_seconds = 30
sync_limit = 20

[auth]
access_token_command = ["pass", "show", "{pass_name}"]

[sink]
backend = "notify-send"
urgency = "normal"
timeout_ms = 8000
open_command = ["rg-fractal-toggle", "show"]
"#
    )
}

fn write_config(
    path: &Path,
    homeserver: &str,
    user_id: &str,
    pass_name: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, config_text(homeserver, user_id, pass_name))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn setup_sso(paths: &Paths, args: &[String]) -> Result<(), String> {
    let mut homeserver = DEFAULT_HOMESERVER.to_string();
    let mut user_id = DEFAULT_USER_ID.to_string();
    let mut pass_name = DEFAULT_PASS_NAME.to_string();
    let mut no_open = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--homeserver" => {
                homeserver = iter
                    .next()
                    .ok_or_else(|| "--homeserver requires a value".to_string())?
                    .trim_end_matches('/')
                    .to_string();
            }
            "--user-id" => {
                user_id = iter
                    .next()
                    .ok_or_else(|| "--user-id requires a value".to_string())?
                    .to_string();
            }
            "--pass-name" => {
                pass_name = iter
                    .next()
                    .ok_or_else(|| "--pass-name requires a value".to_string())?
                    .to_string();
            }
            "--no-open" => no_open = true,
            other => return Err(format!("unknown setup-sso argument: {other}")),
        }
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|err| format!("failed to bind SSO callback listener: {err}"))?;
    let callback = format!(
        "http://127.0.0.1:{}/callback",
        listener
            .local_addr()
            .map_err(|err| format!("failed to read callback listener address: {err}"))?
            .port()
    );
    let login_url = format!(
        "{homeserver}/_matrix/client/v3/login/sso/redirect?redirectUrl={}",
        percent_encode(&callback)
    );
    println!("Matrix SSO URL: {login_url}");
    if !no_open {
        open_browser(&login_url)?;
    }
    let login_token = wait_for_login_token(listener)?;
    let response = exchange_login_token(&homeserver, &login_token)?;
    let access_token = response
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "Matrix login response did not include access_token".to_string())?;
    let returned_user_id = response
        .get("user_id")
        .and_then(Value::as_str)
        .unwrap_or(&user_id);
    store_pass_secret(&pass_name, access_token)?;
    write_config(&paths.config, &homeserver, returned_user_id, &pass_name)?;
    write_json(
        &paths.state_dir.join("status.json"),
        &serde_json::json!({ "state": "Warning", "text": "Mx auth saved", "short_text": "Mx!" }),
    )?;
    println!("Matrix notifier auth stored in pass entry: {pass_name}");
    println!(
        "Matrix notifier config written to: {}",
        paths.config.display()
    );
    Ok(())
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
    if let Err(err) = access_token(config) {
        write_json(
            &state_dir.join("status.json"),
            &serde_json::json!({ "state": "Warning", "text": "Mx setup", "short_text": "Mx!" }),
        )?;
        eprintln!("rg-matrix-notifier: Matrix access token is not configured: {err}");
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
        "usage: rg-matrix-notifier [--config PATH] [--state-dir PATH] <status-json|process-sync PATH|run|setup-sso>"
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
        "setup-sso" => setup_sso(&paths, &args),
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
                open_command: None,
                timeout_ms: None,
            }),
            ..Config::default()
        }
    }

    fn test_state_dir(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!(
            "rg-matrix-notifier-test-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn sample_sync(event_id: &str) -> Value {
        serde_json::json!({
            "next_batch": format!("batch-{event_id}"),
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
                                    "event_id": event_id,
                                    "sender": "@alice:example.org",
                                    "origin_server_ts": 2,
                                    "content": { "body": "hello from a backend agnostic notifier" }
                                }
                            ]
                        }
                    }
                }
            }
        })
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

    #[test]
    fn extracts_sso_login_token_from_callback() {
        let token = login_token_from_request_line(
            "GET /callback?loginToken=abc%2B123&state=ignored HTTP/1.1",
        )
        .unwrap();
        assert_eq!(token.as_deref(), Some("abc+123"));
    }

    #[test]
    fn writes_non_secret_config_text() {
        let text = config_text(
            "https://matrix.surf.nl",
            "@rohit.goswami:surf.nl",
            "matrix/rg-matrix-notifier/access-token",
        );
        assert!(text.contains("homeserver = \"https://matrix.surf.nl\""));
        assert!(text.contains("user_id = \"@rohit.goswami:surf.nl\""));
        assert!(text.contains("access_token_command"));
        assert!(!text.contains("access_token ="));
    }

    #[test]
    fn first_sync_seeds_without_emitting_backlog() {
        let dir = test_state_dir("initial");
        let emitted = process_sync(&sample_sync("$first"), &config(), &dir).unwrap();
        assert_eq!(emitted, 0);
        let seen: BTreeSet<String> = read_json(&dir.join("seen.json"), BTreeSet::new());
        assert!(seen.contains("$first"));
        assert!(dir.join("since.json").exists());

        let emitted = process_sync(&sample_sync("$second"), &config(), &dir).unwrap();
        assert_eq!(emitted, 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_notify_sink_has_open_action() {
        let sink = SinkConfig::default();
        assert_eq!(
            sink.open_command,
            Some(vec!["rg-fractal-toggle".to_string(), "show".to_string()])
        );
        assert_eq!(sink.timeout_ms, Some(8000));
    }
}
