use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{self, Command, ExitStatus, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const TIMED_DEFAULT: &str = "2h";
const AUTO_ICON: &str = "\u{f236}";
const COFFEE_ICON: &str = "\u{f0f4}";
const BOOST_ICON: &str = "\u{f0e7}";
const FOCUS_ICON: &str = "\u{f140}";
const PRESENT_ICON: &str = "\u{f108}";
const MIXED_ICON: &str = "\u{f0ec}";
const INHIBITOR_SERVICE: &str = "rg-caffeine-idle-inhibit.service";
const PERFORMANCE_SERVICE: &str = "rg-caffeine-performance.service";
const SUDO_BIN: &str = "/usr/bin/sudo";
const TLP_BIN: &str = "/usr/bin/tlp";
const TLP_STAT_BIN: &str = "/usr/bin/tlp-stat";

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

    fn status_label(self) -> &'static str {
        match self {
            Self::Off => "auto",
            Self::Idle => "awake",
            Self::Presentation => "present",
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
            Some(remaining) => format!("{} {}", self.mode.status_label(), remaining),
            None => self.mode.status_label().to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PerformanceMode {
    Off,
    Performance,
}

impl PerformanceMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Performance => "performance",
        }
    }
}

impl TryFrom<&str> for PerformanceMode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "off" => Ok(Self::Off),
            "performance" => Ok(Self::Performance),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PerformanceBackend {
    Tlp,
    RgPower,
    PowerProfilesCtl,
    Unavailable,
}

impl PerformanceBackend {
    fn as_str(self) -> &'static str {
        match self {
            Self::Tlp => "tlp",
            Self::RgPower => "rgpower",
            Self::PowerProfilesCtl => "powerprofilesctl",
            Self::Unavailable => "unavailable",
        }
    }
}

impl TryFrom<&str> for PerformanceBackend {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "tlp" => Ok(Self::Tlp),
            "rgpower" => Ok(Self::RgPower),
            "powerprofilesctl" => Ok(Self::PowerProfilesCtl),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PerformanceState {
    mode: PerformanceMode,
    backend: PerformanceBackend,
    restore_profile: Option<String>,
}

impl PerformanceState {
    fn status_text(&self) -> &'static str {
        match self.mode {
            PerformanceMode::Off => "boost off",
            PerformanceMode::Performance => match self.backend {
                PerformanceBackend::Unavailable => "boost noop",
                _ => "boost on",
            },
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

fn state_home() -> PathBuf {
    let state_home = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    state_home
}

fn state_file() -> PathBuf {
    state_home().join("rg-caffeine-idle")
}

fn performance_state_file() -> PathBuf {
    state_home().join("rg-caffeine-performance")
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn default_performance_state() -> PerformanceState {
    PerformanceState {
        mode: PerformanceMode::Off,
        backend: detect_performance_backend(),
        restore_profile: None,
    }
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

fn write_performance_state(path: &PathBuf, state: &PerformanceState) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let restore_profile = state.restore_profile.clone().unwrap_or_default();
    fs::write(
        path,
        format!(
            "mode={}\nbackend={}\nrestore_profile={}\n",
            state.mode.as_str(),
            state.backend.as_str(),
            restore_profile
        ),
    )
}

fn load_performance_state(path: &PathBuf) -> io::Result<PerformanceState> {
    let mut state = default_performance_state();

    match fs::read_to_string(path) {
        Ok(content) => {
            for line in content.lines() {
                let mut parts = line.splitn(2, '=');
                let key = parts.next().unwrap_or_default().trim();
                let value = parts.next().unwrap_or_default().trim();
                match key {
                    "mode" => {
                        if let Ok(mode) = PerformanceMode::try_from(value) {
                            state.mode = mode;
                        }
                    }
                    "backend" => {
                        if let Ok(backend) = PerformanceBackend::try_from(value) {
                            state.backend = backend;
                        }
                    }
                    "restore_profile" => {
                        state.restore_profile = if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        };
                    }
                    _ => {}
                }
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            write_performance_state(path, &state)?;
            return Ok(state);
        }
        Err(err) => return Err(err),
    }

    if state.mode == PerformanceMode::Off {
        state.restore_profile = None;
        state.backend = detect_performance_backend();
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

fn path_is_executable(path: &PathBuf) -> bool {
    fs::metadata(path).map(|meta| meta.is_file()).unwrap_or(false)
}

fn command_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|dir| dir.join(name))
        .find(path_is_executable)
}

fn rgpower_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/bin/rgpower"));
    match home {
        Some(path) if path_is_executable(&path) => Some(path),
        _ => command_path("rgpower"),
    }
}

fn tlp_path() -> Option<PathBuf> {
    let system = PathBuf::from(TLP_BIN);
    if path_is_executable(&system) {
        Some(system)
    } else {
        command_path("tlp")
    }
}

fn detect_performance_backend() -> PerformanceBackend {
    if tlp_path().is_some() {
        PerformanceBackend::Tlp
    } else if rgpower_path().is_some() {
        PerformanceBackend::RgPower
    } else if command_path("powerprofilesctl").is_some() {
        PerformanceBackend::PowerProfilesCtl
    } else {
        PerformanceBackend::Unavailable
    }
}

fn run_command(program: &PathBuf, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} {} exited with {}",
            program.display(),
            args.join(" "),
            exit_code(status)
        ))
    }
}

fn run_named_command(name: &str, args: &[&str]) -> Result<(), String> {
    let program = command_path(name).ok_or_else(|| format!("{name} not found"))?;
    run_command(&program, args)
}

fn run_privileged_absolute(program: &str, args: &[&str]) -> Result<(), String> {
    let sudo = PathBuf::from(SUDO_BIN);
    if !path_is_executable(&sudo) {
        return Err("sudo not found".to_string());
    }

    let status = Command::new(&sudo)
        .arg("-n")
        .arg(program)
        .args(args)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "sudo -n {} {} exited with {}",
            program,
            args.join(" "),
            exit_code(status)
        ))
    }
}

fn powerprofilesctl_current_profile() -> Result<Option<String>, String> {
    let program = command_path("powerprofilesctl").ok_or_else(|| "powerprofilesctl not found".to_string())?;
    let output = Command::new(program)
        .arg("get")
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err("powerprofilesctl get failed".to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

fn tlp_current_profile() -> Result<Option<String>, String> {
    let output = Command::new(SUDO_BIN)
        .args(["-n", TLP_STAT_BIN, "-s"])
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err("tlp-stat -s failed".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if let Some((_, profile)) = line.split_once("Power profile =") {
            let profile = profile.trim();
            if !profile.is_empty() {
                return Ok(Some(profile.to_string()));
            }
        }
    }

    Ok(None)
}

fn sync_idle_inhibitor(mode: Mode) -> Result<(), String> {
    let action = if mode == Mode::Off { "stop" } else { "start" };
    let status = Command::new("systemctl")
        .args(["--user", action, INHIBITOR_SERVICE])
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed to {action} {INHIBITOR_SERVICE}"))
    }
}

fn apply_idle_state(path: &PathBuf, current: State, state: State) -> Result<State, String> {
    write_state(path, state).map_err(|err| err.to_string())?;
    if let Err(err) = sync_idle_inhibitor(state.mode) {
        let _ = write_state(path, current);
        return Err(err);
    }
    Ok(state)
}

fn set_mode(path: &PathBuf, current: State, mode: Mode, duration: Option<&str>) -> Result<State, String> {
    let expires_at = if mode == Mode::Off {
        None
    } else {
        parse_duration(duration)?
            .map(|seconds| now_epoch().saturating_add(seconds))
    };

    apply_idle_state(path, current, State { mode, expires_at })
}

fn sync_performance_service(mode: PerformanceMode) -> Result<(), String> {
    let action = if mode == PerformanceMode::Off { "stop" } else { "start" };
    let status = Command::new("systemctl")
        .args(["--user", action, PERFORMANCE_SERVICE])
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed to {action} {PERFORMANCE_SERVICE}"))
    }
}

fn performance_apply(path: &PathBuf) -> Result<PerformanceState, String> {
    let mut state = load_performance_state(path).map_err(|err| err.to_string())?;
    state.mode = PerformanceMode::Performance;
    state.backend = detect_performance_backend();

    match state.backend {
        PerformanceBackend::Tlp => {
            state.restore_profile = tlp_current_profile()?;
            run_privileged_absolute(TLP_BIN, &["ac"])?;
        }
        PerformanceBackend::RgPower => {
            let program = rgpower_path().ok_or_else(|| "rgpower not found".to_string())?;
            run_command(&program, &["perf"])?;
            state.restore_profile = Some("balanced".to_string());
        }
        PerformanceBackend::PowerProfilesCtl => {
            let current_profile = powerprofilesctl_current_profile()?;
            if current_profile.as_deref() != Some("performance") {
                state.restore_profile = current_profile;
            } else {
                state.restore_profile = None;
            }
            run_named_command("powerprofilesctl", &["set", "performance"])?;
        }
        PerformanceBackend::Unavailable => {
            state.restore_profile = None;
        }
    }

    write_performance_state(path, &state).map_err(|err| err.to_string())?;
    Ok(state)
}

fn performance_release(path: &PathBuf) -> Result<PerformanceState, String> {
    let mut state = load_performance_state(path).map_err(|err| err.to_string())?;

    match state.backend {
        PerformanceBackend::Tlp => {
            run_privileged_absolute(TLP_BIN, &["start"])?;
        }
        PerformanceBackend::RgPower => {
            if let Some(program) = rgpower_path() {
                run_command(&program, &["balanced"])?;
            }
        }
        PerformanceBackend::PowerProfilesCtl => {
            let profile = state.restore_profile.as_deref().unwrap_or("balanced");
            run_named_command("powerprofilesctl", &["set", profile])?;
        }
        PerformanceBackend::Unavailable => {}
    }

    state.mode = PerformanceMode::Off;
    state.restore_profile = None;
    state.backend = detect_performance_backend();
    write_performance_state(path, &state).map_err(|err| err.to_string())?;
    Ok(state)
}

fn set_performance_mode(
    path: &PathBuf,
    current: &PerformanceState,
    mode: PerformanceMode,
) -> Result<PerformanceState, String> {
    match mode {
        PerformanceMode::Off => {
            sync_performance_service(mode)?;
            let state = default_performance_state();
            write_performance_state(path, &state).map_err(|err| err.to_string())?;
            Ok(state)
        }
        PerformanceMode::Performance => {
            let state = PerformanceState {
                mode,
                backend: detect_performance_backend(),
                restore_profile: current.restore_profile.clone(),
            };
            write_performance_state(path, &state).map_err(|err| err.to_string())?;
            if let Err(err) = sync_performance_service(mode) {
                let _ = write_performance_state(path, current);
                return Err(err);
            }
            load_performance_state(path).map_err(|err| err.to_string())
        }
    }
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

fn health_text(health: Health) -> &'static str {
    match health {
        Health::Ok => "ok",
        Health::Unknown => "unknown",
        Health::Error => "error",
    }
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

fn service_active(name: &str) -> Result<bool, ()> {
    let output = Command::new("systemctl")
        .args(["--user", "show", "-p", "ActiveState", name])
        .output()
        .map_err(|_| ())?;
    if !output.status.success() {
        return Err(());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.contains("ActiveState=active"))
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
    match service_active(INHIBITOR_SERVICE) {
        Ok(true) if state.mode == Mode::Off => return Health::Error,
        Ok(false) if state.mode != Mode::Off => return Health::Error,
        Err(_) if state.mode != Mode::Off => return Health::Unknown,
        _ => {}
    }

    Health::Ok
}

fn performance_health(state: &PerformanceState) -> Health {
    match service_active(PERFORMANCE_SERVICE) {
        Ok(true) if state.mode == PerformanceMode::Off => Health::Error,
        Ok(false) if state.mode == PerformanceMode::Performance => Health::Error,
        Err(_) if state.mode == PerformanceMode::Performance => Health::Unknown,
        _ => Health::Ok,
    }
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
                "{{\"state\":\"Info\",\"text\":\"{} AUTO\",\"short_text\":\"{}\"}}",
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
                        "{{\"state\":\"Idle\",\"text\":\"{} AUTO\",\"short_text\":\"{}\"}}",
                        COFFEE_ICON, COFFEE_ICON
                    )
                }
                Mode::Idle => format!(
                    "{{\"state\":\"Good\",\"text\":\"{} AWAKE{}\",\"short_text\":\"{}\"}}",
                    COFFEE_ICON,
                    timer,
                    COFFEE_ICON
                ),
                Mode::Presentation => format!(
                    "{{\"state\":\"Warning\",\"text\":\"{} PRESENT{}\",\"short_text\":\"{}!\"}}",
                    COFFEE_ICON,
                    timer,
                    COFFEE_ICON
                ),
            }
        }
    }
}

fn performance_status_json(state: &PerformanceState, health: Health) -> String {
    let enabled = state.mode == PerformanceMode::Performance;
    match health {
        Health::Error => format!(
            "{{\"state\":\"Critical\",\"text\":\"BOOST ERR\",\"short_text\":\"BOOST!\",\"mode\":\"{}\",\"backend\":\"{}\",\"enabled\":{enabled}}}",
            state.mode.as_str(),
            state.backend.as_str(),
        ),
        _ if enabled && state.backend == PerformanceBackend::Unavailable => format!(
            "{{\"state\":\"Warning\",\"text\":\"BOOST NOOP\",\"short_text\":\"BOOST?\",\"mode\":\"{}\",\"backend\":\"{}\",\"enabled\":{enabled}}}",
            state.mode.as_str(),
            state.backend.as_str(),
        ),
        _ if enabled => format!(
            "{{\"state\":\"Good\",\"text\":\"BOOST ON\",\"short_text\":\"BOOST\",\"mode\":\"{}\",\"backend\":\"{}\",\"enabled\":{enabled}}}",
            state.mode.as_str(),
            state.backend.as_str(),
        ),
        Health::Unknown => format!(
            "{{\"state\":\"Info\",\"text\":\"BOOST OFF\",\"short_text\":\"BOOST\",\"mode\":\"{}\",\"backend\":\"{}\",\"enabled\":{enabled}}}",
            state.mode.as_str(),
            state.backend.as_str(),
        ),
        Health::Ok => format!(
            "{{\"state\":\"Idle\",\"text\":\"BOOST OFF\",\"short_text\":\"BOOST\",\"mode\":\"{}\",\"backend\":\"{}\",\"enabled\":{enabled}}}",
            state.mode.as_str(),
            state.backend.as_str(),
        ),
    }
}

fn focus_enabled(idle: State, performance: &PerformanceState) -> bool {
    idle.mode == Mode::Idle && performance.mode == PerformanceMode::Performance
}

fn focus_health(idle_health: Health, performance_health: Health) -> Health {
    if idle_health == Health::Error || performance_health == Health::Error {
        Health::Error
    } else if idle_health == Health::Unknown || performance_health == Health::Unknown {
        Health::Unknown
    } else {
        Health::Ok
    }
}

fn focus_status_json(
    idle: State,
    performance: &PerformanceState,
    idle_health: Health,
    performance_health_value: Health,
) -> String {
    let focus = focus_enabled(idle, performance);
    let combined_health = focus_health(idle_health, performance_health_value);
    let partially_enabled = idle.mode != Mode::Off || performance.mode == PerformanceMode::Performance;

    match combined_health {
        Health::Error => format!(
            "{{\"state\":\"Critical\",\"text\":\"FOCUS ERR\",\"short_text\":\"FOCUS!\",\"focus\":{focus},\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
        _ if focus && performance.backend == PerformanceBackend::Unavailable => format!(
            "{{\"state\":\"Warning\",\"text\":\"FOCUS NOOP\",\"short_text\":\"FOCUS?\",\"focus\":{focus},\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
        _ if focus => format!(
            "{{\"state\":\"Good\",\"text\":\"FOCUS ON\",\"short_text\":\"FOCUS\",\"focus\":{focus},\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
        _ if partially_enabled => format!(
            "{{\"state\":\"Info\",\"text\":\"FOCUS MIXED\",\"short_text\":\"FOCUS~\",\"focus\":{focus},\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
        Health::Unknown => format!(
            "{{\"state\":\"Info\",\"text\":\"FOCUS OFF\",\"short_text\":\"FOCUS\",\"focus\":{focus},\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
        Health::Ok => format!(
            "{{\"state\":\"Idle\",\"text\":\"FOCUS OFF\",\"short_text\":\"FOCUS\",\"focus\":{focus},\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
    }
}

fn summary_status_json(
    idle: State,
    performance: &PerformanceState,
    idle_health: Health,
    performance_health_value: Health,
) -> String {
    let combined_health = focus_health(idle_health, performance_health_value);
    let timer = remaining_text(idle)
        .map(|remaining| format!(" {remaining}"))
        .unwrap_or_default();

    match combined_health {
        Health::Error => format!(
            "{{\"state\":\"Critical\",\"text\":\"{} Mode\",\"short_text\":\"{}!\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
            MIXED_ICON,
            MIXED_ICON,
            idle.mode.as_str(),
            performance.mode.as_str(),
            performance.backend.as_str(),
        ),
        _ => match (idle.mode, performance.mode) {
            (Mode::Off, PerformanceMode::Off) => format!(
                "{{\"state\":\"Idle\",\"text\":\"{} Auto\",\"short_text\":\"{}\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
                AUTO_ICON,
                AUTO_ICON,
                idle.mode.as_str(),
                performance.mode.as_str(),
                performance.backend.as_str(),
            ),
            (Mode::Idle, PerformanceMode::Off) => format!(
                "{{\"state\":\"Good\",\"text\":\"{} Awake{}\",\"short_text\":\"{}\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
                COFFEE_ICON,
                timer,
                COFFEE_ICON,
                idle.mode.as_str(),
                performance.mode.as_str(),
                performance.backend.as_str(),
            ),
            (Mode::Off, PerformanceMode::Performance) => {
                let (state, label, short_icon) = if performance.backend == PerformanceBackend::Unavailable {
                    ("Warning", "Boost?", BOOST_ICON)
                } else {
                    ("Good", "Boost", BOOST_ICON)
                };
                format!(
                    "{{\"state\":\"{state}\",\"text\":\"{} {label}\",\"short_text\":\"{short_icon}\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
                    BOOST_ICON,
                    idle.mode.as_str(),
                    performance.mode.as_str(),
                    performance.backend.as_str(),
                )
            }
            (Mode::Idle, PerformanceMode::Performance) => {
                let (state, label, short_icon) = if performance.backend == PerformanceBackend::Unavailable {
                    ("Warning", "Focus?", FOCUS_ICON)
                } else {
                    ("Good", "Focus", FOCUS_ICON)
                };
                format!(
                    "{{\"state\":\"{state}\",\"text\":\"{} {label}\",\"short_text\":\"{short_icon}\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
                    FOCUS_ICON,
                    idle.mode.as_str(),
                    performance.mode.as_str(),
                    performance.backend.as_str(),
                )
            }
            (Mode::Presentation, PerformanceMode::Off) => format!(
                "{{\"state\":\"Warning\",\"text\":\"{} Present{}\",\"short_text\":\"{}\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
                PRESENT_ICON,
                timer,
                PRESENT_ICON,
                idle.mode.as_str(),
                performance.mode.as_str(),
                performance.backend.as_str(),
            ),
            (Mode::Presentation, PerformanceMode::Performance) => format!(
                "{{\"state\":\"Warning\",\"text\":\"{} Mixed\",\"short_text\":\"{}\",\"idle_mode\":\"{}\",\"performance_mode\":\"{}\",\"performance_backend\":\"{}\"}}",
                MIXED_ICON,
                MIXED_ICON,
                idle.mode.as_str(),
                performance.mode.as_str(),
                performance.backend.as_str(),
            ),
        },
    }
}

fn set_focus(
    idle_path: &PathBuf,
    current_idle: State,
    performance_path: &PathBuf,
    current_performance: &PerformanceState,
    enabled: bool,
) -> Result<(State, PerformanceState), String> {
    if enabled {
        let idle_state = apply_idle_state(
            idle_path,
            current_idle,
            State {
                mode: Mode::Idle,
                expires_at: None,
            },
        )?;
        match set_performance_mode(performance_path, current_performance, PerformanceMode::Performance) {
            Ok(performance_state) => Ok((idle_state, performance_state)),
            Err(err) => {
                let _ = apply_idle_state(idle_path, idle_state, current_idle);
                Err(err)
            }
        }
    } else {
        let performance_state = set_performance_mode(performance_path, current_performance, PerformanceMode::Off)?;
        let idle_state = apply_idle_state(
            idle_path,
            current_idle,
            State {
                mode: Mode::Off,
                expires_at: None,
            },
        )?;
        Ok((idle_state, performance_state))
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
        "usage: rg-caffeine [toggle|presentation-toggle|on [duration]|timed [duration]|presentation [duration]|off|enabled|mode|action-allowed <action>|run <action> <cmd...>|status|status-json|summary-status-json|health|performance-toggle|performance-on|performance-off|performance-enabled|performance-mode|performance-backend|performance-status|performance-status-json|performance-health|performance-apply|performance-release|focus-toggle|focus-on|focus-off|focus-enabled|focus-status|focus-status-json|focus-health]"
    );
}

fn main() {
    let path = state_file();
    let performance_path = performance_state_file();
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("toggle");

    let current_state = match load_state(&path) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    let current_performance = match load_performance_state(&performance_path) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    let result = match command {
        "toggle" => {
            let new_state = if current_state.mode == Mode::Idle {
                set_mode(&path, current_state, Mode::Off, None)
            } else {
                set_mode(&path, current_state, Mode::Idle, args.get(1).map(String::as_str))
            };
            match new_state {
                Ok(state) => {
                    let message = match state.mode {
                        Mode::Off => "idle policy restored".to_string(),
                        Mode::Idle => remaining_text(state)
                            .map(|remaining| format!("idle sleep blocked for {remaining}"))
                            .unwrap_or_else(|| "idle sleep blocked".to_string()),
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
                set_mode(&path, current_state, Mode::Off, None)
            } else {
                set_mode(&path, current_state, Mode::Presentation, args.get(1).map(String::as_str))
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
        "on" => match set_mode(&path, current_state, Mode::Idle, args.get(1).map(String::as_str)) {
            Ok(state) => {
                let message = remaining_text(state)
                    .map(|remaining| format!("idle sleep blocked for {remaining}"))
                    .unwrap_or_else(|| "idle sleep blocked".to_string());
                notify(&message);
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "timed" => match set_mode(
            &path,
            current_state,
            Mode::Idle,
            Some(args.get(1).map(String::as_str).unwrap_or(TIMED_DEFAULT)),
        ) {
            Ok(state) => {
                let remaining = remaining_text(state).unwrap_or_else(|| TIMED_DEFAULT.to_string());
                notify(&format!("idle sleep blocked for {remaining}"));
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "presentation" => match set_mode(&path, current_state, Mode::Presentation, args.get(1).map(String::as_str))
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
        "off" => match set_mode(&path, current_state, Mode::Off, None) {
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
        "summary-status-json" => {
            println!(
                "{}",
                summary_status_json(
                    current_state,
                    &current_performance,
                    health(current_state),
                    performance_health(&current_performance),
                )
            );
            Ok(0)
        }
        "health" => {
            println!("{}", health_text(health(current_state)));
            Ok(0)
        }
        "performance-toggle" => {
            let target = if current_performance.mode == PerformanceMode::Performance {
                PerformanceMode::Off
            } else {
                PerformanceMode::Performance
            };
            match set_performance_mode(&performance_path, &current_performance, target) {
                Ok(state) => {
                    let summary = if state.mode == PerformanceMode::Performance {
                        match state.backend {
                            PerformanceBackend::Tlp => "boost enabled via tlp",
                            PerformanceBackend::Unavailable => "boost requested (no backend available)",
                            PerformanceBackend::RgPower => "performance enabled via rgpower",
                            PerformanceBackend::PowerProfilesCtl => {
                                "performance enabled via powerprofilesctl"
                            }
                        }
                    } else {
                        "boost mode released"
                    };
                    notify(summary);
                    Ok(0)
                }
                Err(err) => Err(err),
            }
        }
        "performance-on" => match set_performance_mode(
            &performance_path,
            &current_performance,
            PerformanceMode::Performance,
        ) {
            Ok(state) => {
                let summary = match state.backend {
                    PerformanceBackend::Tlp => "boost enabled via tlp",
                    PerformanceBackend::Unavailable => "boost requested (no backend available)",
                    PerformanceBackend::RgPower => "performance enabled via rgpower",
                    PerformanceBackend::PowerProfilesCtl => "performance enabled via powerprofilesctl",
                };
                notify(summary);
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "performance-off" => match set_performance_mode(
            &performance_path,
            &current_performance,
            PerformanceMode::Off,
        ) {
            Ok(_) => {
                notify("boost mode released");
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "performance-enabled" => Ok(if current_performance.mode == PerformanceMode::Performance {
            0
        } else {
            1
        }),
        "performance-mode" => {
            println!("{}", current_performance.mode.as_str());
            Ok(0)
        }
        "performance-backend" => {
            println!("{}", current_performance.backend.as_str());
            Ok(0)
        }
        "performance-status" => {
            println!("{}", current_performance.status_text());
            Ok(0)
        }
        "performance-status-json" => {
            println!(
                "{}",
                performance_status_json(&current_performance, performance_health(&current_performance))
            );
            Ok(0)
        }
        "performance-health" => {
            println!("{}", health_text(performance_health(&current_performance)));
            Ok(0)
        }
        "performance-apply" => match performance_apply(&performance_path) {
            Ok(_) => Ok(0),
            Err(err) => Err(err),
        },
        "performance-release" => match performance_release(&performance_path) {
            Ok(_) => Ok(0),
            Err(err) => Err(err),
        },
        "focus-toggle" => {
            let target = !focus_enabled(current_state, &current_performance);
            match set_focus(&path, current_state, &performance_path, &current_performance, target) {
                Ok((_, performance_state)) => {
                    let summary = if target {
                        match performance_state.backend {
                            PerformanceBackend::Unavailable => "focus mode enabled (awake only; no power backend)",
                            _ => "focus mode enabled (awake + boost)",
                        }
                    } else {
                        "focus mode disabled"
                    };
                    notify(summary);
                    Ok(0)
                }
                Err(err) => Err(err),
            }
        }
        "focus-on" => match set_focus(&path, current_state, &performance_path, &current_performance, true)
        {
            Ok((_, performance_state)) => {
                let summary = match performance_state.backend {
                    PerformanceBackend::Unavailable => "focus mode enabled (awake only; no power backend)",
                    _ => "focus mode enabled (awake + boost)",
                };
                notify(summary);
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "focus-off" => match set_focus(&path, current_state, &performance_path, &current_performance, false)
        {
            Ok(_) => {
                notify("focus mode disabled");
                Ok(0)
            }
            Err(err) => Err(err),
        },
        "focus-enabled" => Ok(if focus_enabled(current_state, &current_performance) {
            0
        } else {
            1
        }),
        "focus-status" => {
            let text = if focus_enabled(current_state, &current_performance) {
                if current_performance.backend == PerformanceBackend::Unavailable {
                    "on (noop power)"
                } else {
                    "on"
                }
            } else if current_state.mode == Mode::Presentation
                || current_performance.mode == PerformanceMode::Performance
            {
                "mixed"
            } else {
                "off"
            };
            println!("{text}");
            Ok(0)
        }
        "focus-status-json" => {
            println!(
                "{}",
                focus_status_json(
                    current_state,
                    &current_performance,
                    health(current_state),
                    performance_health(&current_performance),
                )
            );
            Ok(0)
        }
        "focus-health" => {
            println!(
                "{}",
                health_text(focus_health(
                    health(current_state),
                    performance_health(&current_performance),
                ))
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_state_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        env::temp_dir().join(format!("rg-caffeine-{name}-{unique}.state"))
    }

    #[test]
    fn performance_state_round_trip_preserves_backend_and_restore_profile() {
        let path = temp_state_path("performance");
        let expected = PerformanceState {
            mode: PerformanceMode::Performance,
            backend: PerformanceBackend::PowerProfilesCtl,
            restore_profile: Some("balanced".to_string()),
        };

        write_performance_state(&path, &expected).expect("write performance state");
        let actual = load_performance_state(&path).expect("load performance state");

        assert_eq!(actual, expected);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn performance_status_json_marks_unavailable_backend_as_warning_when_enabled() {
        let state = PerformanceState {
            mode: PerformanceMode::Performance,
            backend: PerformanceBackend::Unavailable,
            restore_profile: None,
        };

        let json = performance_status_json(&state, Health::Ok);

        assert!(json.contains("\"state\":\"Warning\""), "{json}");
        assert!(json.contains("\"backend\":\"unavailable\""), "{json}");
        assert!(json.contains("\"mode\":\"performance\""), "{json}");
    }

    #[test]
    fn focus_enabled_requires_awake_and_performance() {
        let idle = State {
            mode: Mode::Idle,
            expires_at: None,
        };
        let perf = PerformanceState {
            mode: PerformanceMode::Performance,
            backend: PerformanceBackend::PowerProfilesCtl,
            restore_profile: None,
        };

        assert!(focus_enabled(idle, &perf));
        assert!(!focus_enabled(
            State {
                mode: Mode::Off,
                expires_at: None,
            },
            &perf,
        ));
        assert!(!focus_enabled(
            idle,
            &PerformanceState {
                mode: PerformanceMode::Off,
                backend: PerformanceBackend::PowerProfilesCtl,
                restore_profile: None,
            },
        ));
    }

    #[test]
    fn focus_status_json_is_good_when_presentation_and_performance_are_on() {
        let idle = State {
            mode: Mode::Idle,
            expires_at: None,
        };
        let perf = PerformanceState {
            mode: PerformanceMode::Performance,
            backend: PerformanceBackend::PowerProfilesCtl,
            restore_profile: Some("balanced".to_string()),
        };

        let json = focus_status_json(idle, &perf, Health::Ok, Health::Ok);

        assert!(json.contains("\"state\":\"Good\""), "{json}");
        assert!(json.contains("\"focus\":true"), "{json}");
        assert!(json.contains("\"idle_mode\":\"idle\""), "{json}");
        assert!(json.contains("\"performance_mode\":\"performance\""), "{json}");
    }

    #[test]
    fn summary_status_json_uses_focus_label_for_awake_and_performance() {
        let idle = State {
            mode: Mode::Idle,
            expires_at: None,
        };
        let perf = PerformanceState {
            mode: PerformanceMode::Performance,
            backend: PerformanceBackend::Tlp,
            restore_profile: None,
        };

        let json = summary_status_json(idle, &perf, Health::Ok, Health::Ok);

        assert!(json.contains("\"text\":\""), "{json}");
        assert!(json.contains("Focus"), "{json}");
        assert!(json.contains("\"performance_backend\":\"tlp\""), "{json}");
    }
}
