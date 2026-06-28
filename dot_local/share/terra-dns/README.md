# Terra DNS (chezmoi)

Hostname: **rgam5terra** only (`run_onchange_linux_terra_dns.sh.tmpl`).

| Artifact | Sensitivity |
|----------|-------------|
| NM `dns=none` drop-in | Safe |
| `ctrld.service.example` | Safe placeholder |
| `~/.local/share/terra-dns/private/ctrld.service` | **nimvault** (ControlD path tokens) |
| unbound | Distro defaults; Terra uses 127.0.0.1:53 → ctrld :5354 (operator verifies) |

Laptop `rg-fix-*` / us-egress scripts stay **rgx1gen11**-gated and must not install this stack.
