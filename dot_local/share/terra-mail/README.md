# Terra mail stack (chezmoi)

Hostname gate: **`rgam5terra`** only applies system pieces via
`.chezmoiscripts/run_onchange_linux_terra_mail_stack.sh.tmpl`.

| Path | Sensitivity | Management |
|------|-------------|------------|
| `dovecot/local.conf`, `dovecot/dovecot.conf` | Safe (no passwords) | Chezmoi → script installs `/etc/dovecot/*` |
| `~/.config/systemd/user/davmail@.service` | Safe | Hostname-gated tmpl |
| `~/.config/davmail/*.properties` | Sensitive | **nimvault** + gitignore |
| `~/.config/davmail/*.token` | Secret | Host-local only; gitignored; do **not** nimvault |
| `/etc/dovecot/ssl-key.pem` | Secret | Generated on host if missing; never in git |
| `/etc/dovecot/ssl-cert.pem` | Public | Optional copy in `dovecot/ssl-cert.pem` |

On a new Terra: `nimvault unseal` (chezmoi source) then `chezmoi apply`.
