#!/usr/bin/env bash
# Behavior checks for the consolidated SURF profile validator and installer.
set -euo pipefail

fail() {
    printf 'FAIL: %s\n' "$*" >&2
    exit 1
}

if [ $# -gt 1 ]; then
    printf 'usage: rg-surf-vpn-install-selftest.sh [installer]\n' >&2
    exit 2
fi

repo="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
installer="${1:-$repo/dot_local/bin/executable_rg-surf-vpn-install}"
[ -x "$installer" ] || fail "consolidated installer not executable: $installer"

tmp_dir="$(mktemp -d)"
trap 'rtrash -rf "$tmp_dir"' EXIT
mkdir -p "$tmp_dir/bin" "$tmp_dir/etc/openvpn/client" "$tmp_dir/var/lib/rg-surf-vpn"

openssl req -x509 -newkey rsa:2048 -nodes -days 2 \
    -subj '/CN=VPN CA' \
    -keyout "$tmp_dir/ca.key" -out "$tmp_dir/ca.pem" >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes \
    -subj '/CN=test-client/OU=medewerkers-split' \
    -keyout "$tmp_dir/client.key" -out "$tmp_dir/client.csr" >/dev/null 2>&1
openssl x509 -req -days 2 -sha256 \
    -in "$tmp_dir/client.csr" \
    -CA "$tmp_dir/ca.pem" -CAkey "$tmp_dir/ca.key" -CAcreateserial \
    -out "$tmp_dir/client.pem" >/dev/null 2>&1

ca_fingerprint="$(openssl x509 -in "$tmp_dir/ca.pem" -noout -fingerprint -sha256 | cut -d= -f2)"

write_profile() {
    local destination="$1" key_file="$2"
    {
        printf '%s\n' \
            'dev tun' \
            'client' \
            'nobind' \
            'remote-cert-tls server' \
            'verb 3' \
            'server-poll-timeout 10' \
            'tls-version-min 1.3' \
            'data-ciphers AES-256-GCM:CHACHA20-POLY1305' \
            'reneg-sec 0' \
            'script-security 0' \
            '<ca>'
        cat "$tmp_dir/ca.pem"
        printf '%s\n' '</ca>' '<cert>'
        cat "$tmp_dir/client.pem"
        printf '%s\n' '</cert>' '<key>'
        cat "$key_file"
        printf '%s\n' \
            '</key>' \
            '<tls-crypt>' \
            '#' \
            '# 2048 bit OpenVPN static key' \
            '#' \
            '-----BEGIN OpenVPN Static key V1-----' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '0123456789abcdef0123456789abcdef' \
            '-----END OpenVPN Static key V1-----' \
            '</tls-crypt>' \
            'remote medewerkers-split.surf.eduvpn.nl 1197 udp' \
            'remote medewerkers-split.surf.eduvpn.nl 1197 tcp'
    } >"$destination"
    chmod 600 "$destination"
}

profile="$tmp_dir/pending.ovpn"
write_profile "$profile" "$tmp_dir/client.key"

metadata="$tmp_dir/metadata.json"
RG_SURF_CA_SHA256="$ca_fingerprint" "$installer" --validate-only "$profile" >"$metadata"
jq -e '
    .profile == "medewerkers-split"
    and .remote_host == "medewerkers-split.surf.eduvpn.nl"
    and .remote_port == 1197
    and (.cert_sha256 | type == "string")
    and (.not_after_epoch | type == "number")
' "$metadata" >/dev/null || fail "valid profile metadata is incomplete"

unsafe="$tmp_dir/unsafe.ovpn"
cp "$profile" "$unsafe"
printf 'script-security 2\n' >>"$unsafe"
if RG_SURF_CA_SHA256="$ca_fingerprint" "$installer" --validate-only "$unsafe" >/dev/null 2>&1; then
    fail "validator accepted an executable OpenVPN directive"
fi

openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 \
    -out "$tmp_dir/other.key" >/dev/null 2>&1
mismatch="$tmp_dir/mismatch.ovpn"
write_profile "$mismatch" "$tmp_dir/other.key"
if RG_SURF_CA_SHA256="$ca_fingerprint" "$installer" --validate-only "$mismatch" >/dev/null 2>&1; then
    fail "validator accepted a private key that does not match the certificate"
fi

calls="$tmp_dir/calls"
: >"$calls"
cat >"$tmp_dir/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'systemctl %s\n' "$*" >>"$RG_TEST_CALLS"
case "${1:-}" in
is-active|restart) exit 0 ;;
*) exit 0 ;;
esac
EOF
chmod +x "$tmp_dir/bin/systemctl"

cat >"$tmp_dir/bin/journalctl" <<'EOF'
#!/usr/bin/env bash
printf 'Initialization Sequence Completed\n'
EOF
chmod +x "$tmp_dir/bin/journalctl"

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
if [ "$*" = "ahostsv4 servicedesk.surf.nl" ]; then
    printf '145.100.179.10 STREAM servicedesk.surf.nl\n'
fi
EOF
chmod +x "$tmp_dir/bin/getent"

cat >"$tmp_dir/bin/sleep" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$tmp_dir/bin/sleep"

destination="$tmp_dir/etc/openvpn/client/surf-split.conf"
status_file="$tmp_dir/var/lib/rg-surf-vpn/status.json"

run_install() {
    RG_SURF_CA_SHA256="$ca_fingerprint" \
    RG_SURF_SYSTEMCTL="$tmp_dir/bin/systemctl" \
    RG_SURF_JOURNALCTL="$tmp_dir/bin/journalctl" \
    RG_SURF_IP="$tmp_dir/bin/ip" \
    RG_SURF_GETENT="$tmp_dir/bin/getent" \
    RG_SURF_SLEEP="$tmp_dir/bin/sleep" \
    RG_SURF_OVPN="$destination" \
    RG_SURF_STATUS_FILE="$status_file" \
    RG_SURF_SOURCE_UID="$(id -u)" \
    RG_SURF_WAIT_ATTEMPTS=1 \
    RG_TEST_CALLS="$calls" \
    "$installer" install "$profile"
}

run_install >/dev/null
cmp -s "$profile" "$destination" || fail "installer did not promote the validated bytes"
[ "$(stat -c '%a' "$destination")" = 600 ] || fail "installed profile mode is not 0600"
[ "$(grep -c '^systemctl restart openvpn-client@surf-split.service$' "$calls")" = 1 ] \
    || fail "new certificate must restart the OpenVPN unit exactly once"
jq -e '
    .result == "active"
    and .canary_host == "servicedesk.surf.nl"
    and .canary_ip == "145.100.179.10"
    and .canary_device == "tun0"
' "$status_file" >/dev/null || fail "installer status does not record active tunnel metadata"

run_install >/dev/null
[ "$(grep -c '^systemctl restart openvpn-client@surf-split.service$' "$calls")" = 1 ] \
    || fail "same certificate with a healthy tunnel must not restart again"

cp "$profile" "$tmp_dir/accepted.ovpn"
printf 'script-security 2\n' >>"$profile"
if run_install >/dev/null 2>&1; then
    fail "installer accepted a profile rejected by validation"
fi
cmp -s "$tmp_dir/accepted.ovpn" "$destination" \
    || fail "rejected profile replaced the installed configuration"

printf 'ok\n'
