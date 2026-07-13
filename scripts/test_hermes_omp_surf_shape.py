#!/usr/bin/env python3
"""Structural checks for house Hermes + OMP SURF wiring (no live network).

Drives real files on disk (live ~/.hermes and ~/.omp/agent, plus chezmoi
encrypted sources when decryptable). Fails if secrets leak into non-.env
config or SURF model wiring is missing.
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

import yaml

HOME = Path.home()
HERMES = HOME / ".hermes"
OMP = HOME / ".omp" / "agent"
CHZ = HOME / ".local" / "share" / "chezmoi"
TOKEN_RE = re.compile(r"77696c6c6d61-[0-9a-f-]{20,}", re.I)
WILLMA_HOST = "willma.surf.nl"
SURF_MODEL = "openai/gpt-oss-120b"


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def age_decrypt(path: Path) -> str | None:
    """Decrypt a chezmoi age blob if identity is available; else None."""
    if not path.is_file():
        return None
    identity = Path(os.environ.get("CHEZMOI_AGE_IDENTITY", str(HOME / "key.txt")))
    if not identity.is_file():
        return None
    try:
        r = subprocess.run(
            ["age", "-d", "-i", str(identity), str(path)],
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return None
    if r.returncode != 0:
        return None
    return r.stdout


def assert_hermes_surf_model(model: dict, providers: dict | None, label: str) -> None:
    if not isinstance(model, dict):
        fail(f"{label}: model must be mapping, got {type(model).__name__}")
    default = model.get("default") or model.get("model")
    if default != SURF_MODEL:
        fail(f"{label}: SURF model id missing/wrong: {default!r}")
    base = str(model.get("base_url") or "")
    provider = str(model.get("provider") or "")
    providers = providers or {}
    # Docs-valid: provider custom + base_url, OR named providers.<id> with Willma URL
    named_ok = False
    if provider and provider in providers and isinstance(providers[provider], dict):
        pbase = str(providers[provider].get("base_url") or providers[provider].get("api") or "")
        if WILLMA_HOST in pbase or WILLMA_HOST in base:
            named_ok = True
    custom_ok = provider == "custom" and WILLMA_HOST in base
    willma_name_ok = provider in {"willma", "surf", "surf-willma"} and (
        WILLMA_HOST in base or named_ok
    )
    if not (custom_ok or named_ok or willma_name_ok):
        fail(
            f"{label}: need custom+Willma base_url or named providers.* Willma entry; "
            f"got provider={provider!r} base_url={base!r} providers={list(providers)}"
        )
    api_key = model.get("api_key")
    if api_key and not str(api_key).startswith("${"):
        fail(f"{label}: model.api_key must be env-substituted or omitted")


def main() -> None:
    cfg_path = HERMES / "config.yaml"
    if not cfg_path.is_file():
        fail(f"missing {cfg_path}")
    text = cfg_path.read_text()
    if TOKEN_RE.search(text):
        fail("plaintext Willma token in config.yaml")
    cfg = yaml.safe_load(text)
    ver = cfg.get("_config_version")
    if not isinstance(ver, int) or ver < 33:
        fail(f"_config_version expected >=33, got {ver!r}")
    assert_hermes_surf_model(cfg.get("model") or {}, cfg.get("providers") or {}, "live hermes")

    env_path = HERMES / ".env"
    if env_path.is_file():
        keys = {
            line.split("=", 1)[0]
            for line in env_path.read_text().splitlines()
            if "=" in line and not line.strip().startswith("#")
        }
        if "OPENAI_API_KEY" not in keys:
            fail(".env missing OPENAI_API_KEY")
    else:
        fail("missing ~/.hermes/.env")

    # Sealed Hermes source (when decryptable) must match the same contract
    sealed_hermes = CHZ / "private_dot_hermes" / "encrypted_private_config.yaml.age"
    sealed_text = age_decrypt(sealed_hermes)
    if sealed_text is not None:
        if TOKEN_RE.search(sealed_text):
            fail("plaintext Willma token in sealed hermes config")
        sc = yaml.safe_load(sealed_text)
        assert_hermes_surf_model(sc.get("model") or {}, sc.get("providers") or {}, "sealed hermes")
        print("OK: sealed hermes decrypt matches SURF shape")
    else:
        print("SKIP: sealed hermes decrypt unavailable")

    models_yml = OMP / "models.yml"
    if models_yml.is_file():
        mt = models_yml.read_text()
        if "surf-ai-hub" not in mt or SURF_MODEL not in mt:
            fail("OMP models.yml missing surf-ai-hub / gpt-oss-120b")
        if WILLMA_HOST not in mt:
            fail("OMP models.yml missing willma baseUrl")

    cfg_yml = OMP / "config.yml"
    if cfg_yml.is_file():
        oc = yaml.safe_load(cfg_yml.read_text())
        roles = oc.get("modelRoles") or {}
        for role in ("surf", "eb-stack", "surf-plan", "advisor"):
            val = roles.get(role, "")
            if "surf-ai-hub" not in val and "gpt-oss-120b" not in val:
                fail(f"OMP live role {role} not SURF-wired: {val!r}")

    # Host-conditional sealed OMP template: rgSURFLat branch is SURF-primary
    omp_tmpl = CHZ / "dot_omp" / "agent" / "encrypted_config.yml.tmpl.age"
    tmpl = age_decrypt(omp_tmpl)
    if tmpl is not None:
        if '{{- if eq .chezmoi.hostname "rgSURFLat" -}}' not in tmpl:
            fail("OMP sealed config template missing rgSURFLat host branch")
        if "{{- else -}}" not in tmpl or "{{- end -}}" not in tmpl:
            fail("OMP sealed config template missing else/end host branches")
        i = tmpl.index('{{- if eq .chezmoi.hostname "rgSURFLat" -}}')
        j = tmpl.index("{{- else -}}")
        surf_branch = tmpl[i:j]
        if f"default: surf-ai-hub/{SURF_MODEL}" not in surf_branch:
            fail("rgSURFLat branch default is not SURF-primary gpt-oss-120b")
        for needle in (
            "smol: surf-ai-hub/",
            "plan: surf-ai-hub/",
            "commit: surf-ai-hub/",
            "vision: surf-ai-hub/",
            "fallback: surf-ai-hub/",
            "mnemopi",
            "omp-surf/skills",
        ):
            if needle not in surf_branch:
                fail(f"rgSURFLat branch missing SOTA needle: {needle}")
        else_branch = tmpl[j:]
        # Both host branches are Willma-only (Alibaba coding-plan retired).
        if f"default: surf-ai-hub/{SURF_MODEL}" not in else_branch:
            fail("else-branch workstation default is not SURF-primary gpt-oss-120b")
        if "alibaba" in else_branch.lower() and "retired" not in else_branch.lower():
            # comment about retirement is fine; live role ids must not remain
            if "alibaba/" in else_branch:
                fail("else-branch still maps roles to alibaba/*")
        print("OK: sealed OMP host-conditional template (both branches SURF-primary)")
    else:
        print("SKIP: OMP sealed template decrypt unavailable")

    print("OK: hermes+omp SURF shape and secret split")


if __name__ == "__main__":
    main()
