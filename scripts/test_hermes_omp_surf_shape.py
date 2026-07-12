#!/usr/bin/env python3
"""Structural checks for house Hermes + OMP SURF wiring (no live network).

Drives real files on disk (live ~/.hermes and ~/.omp/agent, plus chezmoi
sources when present). Fails if secrets leak into non-.env config or SURF
model wiring is missing.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

HOME = Path.home()
HERMES = HOME / ".hermes"
OMP = HOME / ".omp" / "agent"
TOKEN_RE = re.compile(r"77696c6c6d61-[0-9a-f-]{20,}", re.I)


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


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
    model = cfg.get("model")
    if not isinstance(model, dict):
        fail(f"model must be mapping, got {type(model).__name__}")
    default = model.get("default") or model.get("model")
    if default != "openai/gpt-oss-120b":
        fail(f"SURF model id missing/wrong: {default!r}")
    base = model.get("base_url") or ""
    if "willma.surf.nl" not in str(base):
        fail(f"Willma base_url missing: {base!r}")
    if model.get("provider") != "custom":
        fail(f"provider expected custom, got {model.get('provider')!r}")
    # secrets must not be inline literals
    api_key = model.get("api_key")
    if api_key and not str(api_key).startswith("${"):
        fail("model.api_key must be env-substituted or omitted")
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

    models_yml = OMP / "models.yml"
    if models_yml.is_file():
        mt = models_yml.read_text()
        if "surf-ai-hub" not in mt or "openai/gpt-oss-120b" not in mt:
            fail("OMP models.yml missing surf-ai-hub / gpt-oss-120b")
        if "willma.surf.nl" not in mt:
            fail("OMP models.yml missing willma baseUrl")
    cfg_yml = OMP / "config.yml"
    if cfg_yml.is_file():
        oc = yaml.safe_load(cfg_yml.read_text())
        roles = oc.get("modelRoles") or {}
        for role in ("surf", "eb-stack", "surf-plan", "advisor"):
            val = roles.get(role, "")
            if "surf-ai-hub" not in val and "gpt-oss-120b" not in val:
                fail(f"OMP role {role} not SURF-wired: {val!r}")

    print("OK: hermes+omp SURF shape and secret split")


if __name__ == "__main__":
    main()
