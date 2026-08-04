#!/usr/bin/env bash
# Behavior checks for SURF authorization renewal and profile spooling.
set -euo pipefail

fail() {
    printf 'FAIL: %s\n' "$*" >&2
    exit 1
}

if [ $# -gt 1 ]; then
    printf 'usage: rg-surf-vpn-controller-selftest.sh [controller]\n' >&2
    exit 2
fi

repo="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
controller="${1:-$repo/dot_local/bin/executable_rg-surf-vpn}"
[ -x "$controller" ] || fail "SURF controller not executable: $controller"

tmp_dir="$(mktemp -d)"
trap 'rtrash -rf "$tmp_dir"' EXIT
fake_root="$tmp_dir/fake"
mkdir -p "$fake_root/eduvpn" "$fake_root/eduvpn_common" "$tmp_dir/state/rg-surf-vpn" "$tmp_dir/bin"
calls="$tmp_dir/calls"
: >"$calls"

cat >"$fake_root/eduvpn_common/state.py" <<'PY'
from enum import IntEnum


class StateType(IntEnum):
    ENTER = 1
    LEAVE = 2


class State(IntEnum):
    DEREGISTERED = 0
    MAIN = 1
    ADDING_SERVER = 2
    OAUTH_STARTED = 3
    GETTING_CONFIG = 4
    ASK_LOCATION = 5
    ASK_PROFILE = 6
    GOT_CONFIG = 7
PY

cat >"$fake_root/eduvpn_common/event.py" <<'PY'
def class_state_transition(state, state_type):
    def decorate(function):
        function._test_transition = (state, state_type)
        return function
    return decorate
PY

cat >"$fake_root/eduvpn_common/main.py" <<'PY'
import json
import os
import time
from enum import IntEnum

from .state import State, StateType


class ServerType(IntEnum):
    UNKNOWN = 0
    INSTITUTE_ACCESS = 1
    SECURE_INTERNET = 2
    CUSTOM = 3

    def __str__(self):
        if self == ServerType.INSTITUTE_ACCESS:
            return "Institute Access Server"
        return self.name


class EduVPN:
    def __init__(self, name, version, config_directory):
        self.callbacks = []
        self.profile = None
        self.token_getter = None
        self.token_setter = None
        self.cancelled = False

    def record(self, value):
        with open(os.environ["RG_TEST_EDUVPN_CALLS"], "a", encoding="utf-8") as stream:
            stream.write(value + "\n")

    def register_class_callbacks(self, callback):
        self.callbacks.append(callback)

    def register(self):
        self.record("register")

    def deregister(self):
        self.record("deregister")

    def set_token_handler(self, getter, setter):
        self.token_getter = getter
        self.token_setter = setter

    def emit(self, state, data):
        for callback in self.callbacks:
            for name in dir(callback):
                method = getattr(callback, name)
                if getattr(method, "_test_transition", None) == (state, StateType.ENTER):
                    method(State.MAIN, data)

    def cookie_reply(self, cookie, value):
        self.profile = value
        self.record("profile=" + value)

    def cancel(self):
        self.record("cancel")
        self.cancelled = True

    def get_servers(self):
        missing = os.environ.get("RG_TEST_SERVER_MODE", "present") == "missing"
        institutes = [] if missing else [{
            "identifier": "https://surf.eduvpn.nl/",
            "display_name": {"en": "SURF BV"},
            "support_contacts": [],
            "profiles": {"map": {}, "current": ""},
            "delisted": False,
        }]
        return json.dumps({
            "institute_access_servers": institutes,
            "custom_servers": [],
            "secure_internet_server": None,
        })

    def add_server(self, server_type, identifier, oauth_started=None):
        self.record("add-server=" + identifier)

    def renew_session(self):
        # The controller must never call this: from the Main state the real
        # library rejects the transition to OAuthStarted.
        self.record("renew-session")
        raise RuntimeError("fsm invalid transition attempt from 'Main' to 'OAuthStarted'")

    def get_config(self, server_type, identifier, prefer_tcp=False, startup=False):
        self.record("get-config")
        stored = self.token_getter(identifier, int(server_type))
        oauth_forced = os.environ.get("RG_TEST_OAUTH_MODE", "tokens") == "required"
        if stored is None or oauth_forced:
            # No usable tokens: the real core starts OAuth from inside the
            # config request. A silent caller cancels here; an interactive
            # caller opens the browser and the flow continues.
            self.emit(State.OAUTH_STARTED, "https://auth.invalid/secret-state")
            # Give a silent caller's async cancel time to land.
            for _ in range(200):
                if self.cancelled:
                    break
                time.sleep(0.001)
            if self.cancelled:
                raise RuntimeError("cancelled through client")
            self.record("oauth-complete")
        elif sorted(json.loads(stored)) != ["access_token", "expires_at", "refresh_token"]:
            raise RuntimeError("incomplete token getter")
        profile_data = {
            "cookie": 7,
            "data": {
                "map": {
                    "medewerkers": {"display_name": {"en": "all"}},
                    "medewerkers-split": {"display_name": {"en": "split"}},
                },
                "current": "medewerkers-split",
            },
        }
        self.emit(State.ASK_PROFILE, json.dumps(profile_data))
        for _ in range(100):
            if self.profile:
                break
            time.sleep(0.001)
        if self.profile != "medewerkers-split":
            raise RuntimeError("wrong profile")
        self.token_setter(
            identifier,
            int(server_type),
            json.dumps({
                "access_token": "rotated-access",
                "refresh_token": "rotated-refresh",
                "expires_at": 1999999999,
            }),
        )
        return json.dumps({
            "config": "VALID PROFILE\n",
            "protocol": 1,
            "default_gateway": False,
            "dns_search_domains": ["ia.surf.nl"],
            "should_failover": False,
        })
PY

cat >"$fake_root/eduvpn/__init__.py" <<'PY'
__version__ = "4.7.2-test"
PY

cat >"$fake_root/eduvpn/variants.py" <<'PY'
from pathlib import Path


class Variant:
    client_id = "org.eduvpn.app.linux"
    config_prefix = Path("/tmp/fake-eduvpn")
    name = "eduVPN"


EDUVPN = Variant()
PY

cat >"$fake_root/eduvpn/keyring.py" <<'PY'
import json


class DBusKeyring:
    def __init__(self, variant):
        pass

    def load(self, attributes):
        return json.dumps({
            "access": "stored-access",
            "refresh": "stored-refresh",
            "expires_at": "1888888888",
        })

    def save(self, label, attributes, secret):
        parsed = json.loads(secret)
        if sorted(parsed) != ["access", "expires_at", "refresh"]:
            raise RuntimeError("incomplete keyring save")
PY

cat >"$tmp_dir/bin/browser" <<'EOF'
#!/usr/bin/env bash
printf 'browser\n' >>"$RG_TEST_CALLS"
EOF
chmod +x "$tmp_dir/bin/browser"

cat >"$tmp_dir/bin/installer" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'validate %s\n' "$*" >>"$RG_TEST_CALLS"
[ "$1" = --validate-only ]
[ "$(cat "$2")" = "VALID PROFILE" ]
[ "${RG_TEST_VALIDATE_MODE:-success}" = success ]
cat <<'JSON'
{"cert_sha256":"AA:BB","not_after":"2026-08-03T00:00:00+00:00","not_after_epoch":1785715200,"profile":"medewerkers-split","remote_host":"medewerkers-split.surf.eduvpn.nl","remote_port":1197}
JSON
EOF
chmod +x "$tmp_dir/bin/installer"

cat >"$tmp_dir/bin/notify-send" <<'EOF'
#!/usr/bin/env bash
printf 'notify\n' >>"$RG_TEST_CALLS"
printf '%s\n' "${RG_TEST_NOTIFY_ACTION:-later}"
EOF
chmod +x "$tmp_dir/bin/notify-send"

cat >"$tmp_dir/bin/timeout" <<'EOF'
#!/usr/bin/env bash
shift
exec "$@"
EOF
chmod +x "$tmp_dir/bin/timeout"

cat >"$tmp_dir/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
[ "${1:-}" = is-active ]
EOF
chmod +x "$tmp_dir/bin/systemctl"

cat >"$tmp_dir/bin/ip" <<'EOF'
#!/usr/bin/env bash
case "$*" in
"link show tun0") exit 0 ;;
"route get 145.100.179.10") printf '145.100.179.10 dev tun0 src 10.8.0.2\n' ;;
*) exit 1 ;;
esac
EOF
chmod +x "$tmp_dir/bin/ip"

cat >"$tmp_dir/bin/getent" <<'EOF'
#!/usr/bin/env bash
printf '145.100.179.10 STREAM servicedesk.surf.nl\n'
EOF
chmod +x "$tmp_dir/bin/getent"

stdout="$tmp_dir/stdout"
stderr="$tmp_dir/stderr"
pending="$tmp_dir/state/rg-surf-vpn/pending.ovpn"
status_file="$tmp_dir/status.json"

run_controller() {
    set +e
    local -a environment=(
        PYTHONPATH="$fake_root"
        XDG_STATE_HOME="$tmp_dir/state"
        RG_TEST_EDUVPN_CALLS="$calls"
        RG_TEST_CALLS="$calls"
        RG_SURF_BROWSER="$tmp_dir/bin/browser"
        RG_SURF_INSTALLER="$tmp_dir/bin/installer"
        RG_SURF_STATUS_FILE="$status_file"
        RG_SURF_NOTIFY_SEND="$tmp_dir/bin/notify-send"
        RG_SURF_TIMEOUT="$tmp_dir/bin/timeout"
        RG_SURF_SYSTEMCTL="$tmp_dir/bin/systemctl"
        RG_SURF_IP="$tmp_dir/bin/ip"
        RG_SURF_GETENT="$tmp_dir/bin/getent"
        RG_SURF_NOW=1000000000
        RG_SURF_NOTIFY_TIMEOUT=1
        RG_TEST_SERVER_MODE="${RG_TEST_SERVER_MODE:-present}"
        RG_TEST_VALIDATE_MODE="${RG_TEST_VALIDATE_MODE:-success}"
        RG_TEST_NOTIFY_ACTION="${RG_TEST_NOTIFY_ACTION:-later}"
        RG_TEST_OAUTH_MODE="${RG_TEST_OAUTH_MODE:-tokens}"
    )
    # The escalation scenarios exercise the built-in cooldown defaults.
    if [ "${RG_TEST_COOLDOWN_DEFAULTS:-0}" != 1 ]; then
        environment+=(RG_SURF_NOTIFY_COOLDOWN=0)
    fi
    env "${environment[@]}" python3 "$controller" "$@" >"$stdout" 2>"$stderr"
    rc=$?
    set -e
    return "$rc"
}

# Interactive renewal forces a fresh OAuth through get_config; the illegal
# renew_session transition must never be attempted.
RG_TEST_SERVER_MODE=missing run_controller renew --interactive \
    || fail "interactive renewal with missing server failed"
[ "$(cat "$pending")" = "VALID PROFILE" ] || fail "interactive renewal did not spool the validated profile"
rg -q '^add-server=https://surf.eduvpn.nl/$' "$calls" || fail "missing server was not repaired"
! rg -q '^renew-session$' "$calls" || fail "interactive renewal used the illegal renew_session transition"
rg -q '^browser$' "$calls" || fail "interactive renewal did not open browser authorization"
rg -q '^oauth-complete$' "$calls" || fail "interactive renewal did not complete a fresh OAuth"
rg -q '^get-config$' "$calls" || fail "interactive renewal did not retrieve a profile"
rg -q '^validate --validate-only ' "$calls" || fail "candidate was not validated before spooling"
! rg -qi 'stored-access|stored-refresh|rotated-access|rotated-refresh|secret-state' "$stdout" "$stderr" \
    || fail "controller leaked OAuth material or the authorization URL"

printf 'OLD PROFILE\n' >"$pending"
chmod 600 "$pending"
: >"$calls"
if RG_TEST_VALIDATE_MODE=fail run_controller renew --interactive; then
    fail "interactive renewal accepted a candidate rejected by validation"
fi
[ "$(cat "$pending")" = "OLD PROFILE" ] || fail "rejected candidate replaced the pending profile"

# Near expiry with a working refresh token: the timer renews silently, with
# no browser and no notification.
cat >"$status_file" <<'JSON'
{"result":"active","not_after_epoch":1000003600,"cert_sha256":"11:22"}
JSON
printf 'OLD PROFILE\n' >"$pending"
: >"$calls"
run_controller renew --timer || fail "near-expiry silent renewal failed"
rg -q '^get-config$' "$calls" || fail "near-expiry timer did not attempt a silent renewal"
! rg -q '^browser$' "$calls" || fail "silent renewal opened a browser"
! rg -q '^notify$' "$calls" || fail "silent renewal raised a notification"
[ "$(cat "$pending")" = "VALID PROFILE" ] || fail "silent renewal did not spool the renewed profile"

# Near expiry when SURF demands reauthorization: the silent attempt cancels
# without a browser and the operator is notified.
cat >"$status_file" <<'JSON'
{"result":"active","not_after_epoch":1000003600,"cert_sha256":"11:22"}
JSON
printf 'OLD PROFILE\n' >"$pending"
: >"$calls"
RG_TEST_OAUTH_MODE=required run_controller renew --timer \
    || fail "authorization-required timer check failed"
rg -q '^cancel$' "$calls" || fail "silent attempt did not cancel the OAuth flow"
! rg -q '^browser$' "$calls" || fail "silent attempt opened a browser"
rg -q '^notify$' "$calls" || fail "authorization-required timer did not notify"
[ "$(cat "$pending")" = "OLD PROFILE" ] || fail "authorization-required timer changed the pending profile"

# Escalation: with built-in cooldown defaults, a recent notification throttles
# the near-expiry reminder but not the expired-tunnel reminder.
printf '999995000\n' >"$tmp_dir/state/rg-surf-vpn/auth-notified-at"
cat >"$status_file" <<'JSON'
{"result":"active","not_after_epoch":1000003600,"cert_sha256":"11:22"}
JSON
: >"$calls"
RG_TEST_COOLDOWN_DEFAULTS=1 RG_TEST_OAUTH_MODE=required run_controller renew --timer \
    || fail "throttled near-expiry timer check failed"
! rg -q '^notify$' "$calls" || fail "near-expiry reminder ignored the six-hour cooldown"

printf '999995000\n' >"$tmp_dir/state/rg-surf-vpn/auth-notified-at"
cat >"$status_file" <<'JSON'
{"result":"active","not_after_epoch":999999999,"cert_sha256":"11:22"}
JSON
: >"$calls"
RG_TEST_COOLDOWN_DEFAULTS=1 RG_TEST_OAUTH_MODE=required run_controller renew --timer \
    || fail "expired escalation timer check failed"
rg -q '^notify$' "$calls" || fail "expired tunnel did not escalate past the cooldown"

cat >"$status_file" <<'JSON'
{"result":"active","not_after_epoch":2000000000,"cert_sha256":"11:22"}
JSON
: >"$calls"
run_controller renew --timer || fail "healthy timer check failed"
[ ! -s "$calls" ] || fail "healthy long-lived tunnel triggered controller activity"

printf 'ok\n'
