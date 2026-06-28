# Terra buildbot (chezmoi)

Hostname: **rgam5terra** only.

Safe: hzlinux profile, profile.d caches, optional borg system timer units (no passphrase),
sshd hardening drop-ins, nix.conf fragment.

Secrets: `~/.config/borgmatic/config.yaml` via **nimvault** (passphrase via `pass`, not git).
