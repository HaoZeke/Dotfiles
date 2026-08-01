# Terra calendar hub (RustiCal)

Single Rust CalDAV binary on Terra. See vault:
`Software/Infra/2026-07-22-terra-calendar-sota-rustical.org`

- Binary: `~/.local/bin/rustical` (built from github.com/lennart-k/rustical)
- Config: `~/.config/rustical/config.toml` (loopback 127.0.0.1:4000)
- DB: `~/.local/share/rustical/db.sqlite3`
- Secrets (host-local, never commit): `frontend.password`, `app-token.evolution`
- Laptop tunnel: `terra-caldav-tunnel.service` → localhost:4000
- Status: `terra-calendar-status`

Evolution: CalDAV URL `http://127.0.0.1:4000/caldav`, user `rgoswami`,
password = app token (not frontend password).
