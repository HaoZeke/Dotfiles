use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{self, Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const TIMED_DEFAULT: &str = "2h";
const COFFEE_ICON: &str = "\u{f0f4}";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Off,
    Idle,
    Presentation,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Idle => "idle",
            Self::Presentation => "presentation",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Idle => "AWAKE",
            Self::Presentation => "PRESENT",
        }
    }
}

impl TryFrom<&str> for Mode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "off" => Ok(Self::Off),
            "idle" => Ok(Self::Idle),
            "presentation" => Ok(Self::Presentation),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct State {
    mode: Mode,
    expires_at: Option<u64>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            expires_at: None,
        }
    }
}

impl State {
    fn status_text(self) -> String {
        match remaining_text(self) {
            Some(remaining) => format!("{} {}", self.mode.label().to_lowercase(), remaining),
            None => self.mode.label().to_lowercase(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Health {
    Ok,
    Unknown,
    Error,
}

#[derive(Clone, Copy, Debug)]
enum Action {
    Dim,
    DimResume,
    Lock,
    DpmsOff,
    DpmsOn,
    Hibernate,
}

impl TryFrom<&str> for Action {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dim" => Ok(Self::Dim),
            "dim-resume" => Ok(Self::DimResume),
            "lock" => Ok(Self::Lock),
            "dpms-off" => Ok(Self::DpmsOff),
            "dpms-on" => Ok(Self::DpmsOn),
            "hibernate" => Ok(Self::Hibernate),
            _ => Err(()),
        }
    }
}

fn state_file() -> PathBuf {
    let state_home = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    state_home.join("rg-caffeine-idle")
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn write_state(path: &PathBuf, state: State) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let expires_at = state.expires_at.unwrap_or(0);
    fs::write(path, format!("mode={}\nexpires_at={}\n", state.mode.as_str(), expires_at))
}

fn load_state(path: &PathBuf) -> io::Result<State> {
    let mut state = State::default();

    match fs::read_to_string(path) {
        Ok(content) => {
            for line in content.lines() {
                let mut parts = line.splitn(2, '=');
                let key = parts.next().unwrap_or_default().trim();
                let value = parts.next().unwrap_or_default().trim();
                match key {
                    "mode" => {
                        if let Ok(mode) = Mode::try_from(value) {
                            state.mode = mode;
                        }
                    }
                    "expires_at" => {
                        if let Ok(epoch) = value.parse::<u64>() {
                            state.expires_at = if epoch == 0 { None } else { Some(epoch) };
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            write_state(path, state)?;
            return Ok(state);
        }
        Err(err) => return Err(err),
    }

    if state.mode == Mode::Off {
        state.expires_at = None;
    }

    if let Some(expires_at) = state.expires_at {
        if now_epoch() >= expires_at {
            state = State::default();
            write_state(path, state)?;
        }
    }

    Ok(state)
}

fn parse_duration(raw: Option<&str>) -> Result<Option<u64>, String> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.is_empty() {
        return Ok(None);
    }

    if let Ok(minutes) = raw.parse::<u64>() {
        return Ok(Some(minutes * 60));
    }

    let (number, unit) = raw.split_at(raw.len().saturating_sub(1));
    let value = number
        .parse::<u64>()
        .map_err(|_| format!("invalid duration: {raw}"))?;
    let seconds = match unit {
        "s" => value,
        "m" => value * 60,
        "h" => value * 3600,
        "d" => value * 86_400,
        _ => return Err(format!("invalid duration: {raw}")),
    };
    Ok(Some(seconds))
}

fn set_mode(path: &PathBuf, mode: Mode, duration: Option<&str>) -> Result<State, String> {
    let expires_at = if mode == Mode::Off {
        None
    } else {
        parse_duration(duration)?
            .map(|seconds| now_epoch().saturating_add(seconds))
    };

    let state = State { mode, expires_at };
    write_state(path, state).map_err(|err| err.to_string())?;
    Ok(state)
}

fn remaining_text(state: State) -> Option<String> {
    let expires_at = state.expires_at?;
    if expires_at <= now_epoch() {
        return None;
    }

    let seconds_left = expires_at - now_epoch();
    let minutes_left = (seconds_left + 59) / 60;
    if minutes_left < 60 {
        return Some(format!("{minutes_left}m"));
    }

    let hours_left = minutes_left / 60;
    let rem_minutes = minutes_left % 60;
    if hours_left < 24 {
        return Some(if rem_minutes == 0 {
            format!("{hours_left}h")
        } else {
            format!("{hours_left}h{rem_minutes:02}m")
        });
    }

    let days_left = hours_left / 24;
    let rem_hours = hours_left % 24;
    Some(if rem_hours == 0 {
        format!("{days_left}d")
    } else {
        format!("{days_left}d{rem_hours:02}h")
    })
}

fn action_allowed(state: State, action: Action) -> bool {
    match (state.mode, action) {
        (Mode::Off, _) => true,
        (Mode::Idle, Action::Hibernate) => false,
        (Mode::Idle, _) => true,
        (Mode::Presentation, Action::Dim) => false,
        (Mode::Presentation, Action::Lock) => false,
        (Mode::Presentation, Action::DpmsOff) => false,
        (Mode::Presentation, Action::Hibernate) => false,
        (Mode::Presentation, _) => true,
    }
}

fn notify(summary: &str) {
    if env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        return;
    }
    let _ = Command::new("notify-send")
        .arg("Idle policy")
        .arg(summary)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn health(state: State) -> Health {
    let output = match Command::new("systemctl")
        .args(["--user", "show", "-p", "ActiveState", "-p", "ExecStart", "swayidle.service"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return Health::Unknown,
    };

    if !output.status.success() {
        return Health::Unknown;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    if !text.contains("ActiveState=active") {
        return Health::Error;
    }
    if !text.contains("rg-idle-hibernate") {
        return Health::Error;
    }
    if state.mode == Mode::Presentation
        && (!text.contains("rg-caffeine run lock")
            || !text.contains("rg-caffeine run dpms-off")
            || !text.contains("rg-caffeine run dim"))
    {
        return Health::Error;
    }

    Health::Ok
}

fn status_json(state: State) -> String {
    match health(state) {
        Health::Error => {
            format!(
                "{{\"state\":\"Critical\",\"text\":\"{} ERR\",\"short_text\":\"{}!\"}}",
                COFFEE_ICON, COFFEE_ICON
            )
        }
        Health::Unknown if state.mode == Mode::Off => {
            format!(
                "{{\"state\":\"Info\",\"text\":\"{} OFF\",\"short_text\":\"{}\"}}",
                COFFEE_ICON, COFFEE_ICON
            )
        }
        _ => {
            let timer = remaining_text(state)
                .map(|remaining| format!(" {remaining}"))
                .unwrap_or_default();
            match state.mode {
                Mode::Off => {
                    format!(
                        "{{\"state\":\"Idle\",\"text\":\"{} OFF\",\"short_text\":\"{}\"}}",
                        COFFEE_ICON, COFFEE_ICON
                    )
                }
                Mode::Idle => format!(
                    "{{\"state\":\"Good\",\"text\":\"{} {}{}\",\"short_text\":\"{}\"}}",
                    COFFEE_ICON,
                    state.mode.label(),
                    timer,
                    COFFEE_ICON
                ),
                Mode::Presentation => format!(
                    "{{\"state\":\"Warning\",\"text\":\"{} {}{}\",\"short_text\":\"{}!\"}}",
                    COFFEE_ICON,
                    state.mode.label(),
                    timer,
                    COFFEE_ICON
                ),
            }
        }
    }
}

fn run_action(args: &[String], state: State) -> Result<i32, String> {
    if args.len() < 2 {
        return Err("usage: rg-caffeine run <action> <cmd...>".to_string());
    }
    let action = Action::try_from(args[0].as_str())
        .map_err(|_| format!("invalid action: {}", args[0]))?;
    if !action_allowed(state, action) {
        return Ok(0);
    }

    let mut cmd = Command::new(&args[1]);
    cmd.args(&args[2..]);
    let status = cmd.status().map_err(|err| err.to_string())?;
    Ok(exit_code(status))
}

fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

fn print_usage() {
    eprintln!(
        "usage: rg-caffeine [toggle|presentation-toggle|on [duration]|timed [duration]|presentation [duration]|off|enabled|mode|action-allowed <action>|run <action> <cmd...>|status|status-json|health]"
    );
}

fn main() {
    let path = state_file();
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("toggle");

    let current_state = match load_state(&path) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    let result = match command {
        "toggle" => {
            let new_state = if current_state.mode == Mode::Idle {
                set_mode(&path, Mode::Off, None)
            } else {
                set_mode(&path, Mode::Idle, args.get(1).map(String::as_str))
            };
            match new_state {
                Ok(state) => {
                    let message = match state.mode {
                        Mode::Off => "idle policy restored".to_string(),
                        Mode::Idle => remaining_text(state)
                            .map(|remaining| format!("awake mode for {remaining}"))
                            .unwrap_or_else(|| "awake mode enabled".to_string()),
                        Mode::Presentation => unreachable!(),
                    };
                    notify(&message);
                    Ok(0)
                }
                Err(err) => Err(err),
            }
        }
        "presentation-toggle" => {
            let new_state = if current_state.mode == Mode::Presentation {
                set_mode(&path, Mode::Off, None)
            } else {
                set_mode(&path, Mode::Presentation, args.get(1).map(String::as_str))
            };
            match new_state {
                Ok(state) => {
                    let message = match state.mode {
                        Mode::Off => "idle policy restored".to_string(),
                        Mode::Presentation => remaining_text(state)
                            .map(|remaining| format!("presentation mode for {remaining}"))
                            .unwrap_or_else(|| "presentation mode enabled".to_string()),
                        Mode::Idle => unreachable!(),
                    };
                    notify(&message);
                    Ok(0)
                }
                Err(err) => Err(err),
            }
        }
        "on" => match set_mode(&path, Mode::Idle, args.get(1).map(String::as_str)) {
            Ok(state) => {
                let message = remaining_text(state)
                    .map(|remaining| format!("awake mode for {remaining}"))
                    .unwrap_or_else(|| "awake mode enabled".to_string());
                notify(&message);
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "timed" => match set_mode(
            &path,
            Mode::Idle,
            Some(args.get(1).map(String::as_str).unwrap_or(TIMED_DEFAULT)),
        ) {
            Ok(state) => {
                let remaining = remaining_text(state).unwrap_or_else(|| TIMED_DEFAULT.to_string());
                notify(&format!("awake mode for {remaining}"));
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "presentation" => match set_mode(&path, Mode::Presentation, args.get(1).map(String::as_str))
        {
            Ok(state) => {
                let message = remaining_text(state)
                    .map(|remaining| format!("presentation mode for {remaining}"))
                    .unwrap_or_else(|| "presentation mode enabled".to_string());
                notify(&message);
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "off" => match set_mode(&path, Mode::Off, None) {
            Ok(_) => {
                notify("idle policy restored");
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "enabled" => Ok(if current_state.mode == Mode::Off { 1 } else { 0 }),
        "mode" => {
            println!("{}", current_state.mode.as_str());
            Ok(0)
        }
        "action-allowed" => match args.get(1) {
            Some(action) => match Action::try_from(action.as_str()) {
                Ok(action) => Ok(if action_allowed(current_state, action) { 0 } else { 1 }),
                Err(_) => Err(format!("invalid action: {action}")),
            },
            None => Err("usage: rg-caffeine action-allowed <action>".to_string()),
        },
        "run" => run_action(&args[1..], current_state),
        "status" => {
            println!("{}", current_state.status_text());
            Ok(0)
        }
        "status-json" => {
            println!("{}", status_json(current_state));
            Ok(0)
        }
        "health" => {
            let text = match health(current_state) {
                Health::Ok => "ok",
                Health::Unknown => "unknown",
                Health::Error => "error",
            };
            println!("{text}");
            Ok(0)
        }
        _ => {
            print_usage();
            Ok(2)
        }
    };

    match result {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}
