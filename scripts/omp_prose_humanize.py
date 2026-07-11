#!/usr/bin/env python3
"""OMP prose humanize: single gate (style+grammar) + optional Willma rewrite loop.

Gate entry (what agents should run before shipping prose)::

  python3 scripts/omp_prose_humanize.py gate path/to/file.md
  python3 scripts/omp_prose_humanize.py gate --json path/to/file.md

Rewrite loop (Willma free primary; token via pass show surf/ai-hub/token)::

  python3 scripts/omp_prose_humanize.py fix path/to/file.md
  python3 scripts/omp_prose_humanize.py fix --no-llm path/to/file.md   # gate only
  python3 scripts/omp_prose_humanize.py fix --max-attempts 3 path.md

The gate prefers ``proseguard`` with the ``prose-human`` profile (lexical
rgoswami styles + harper). If proseguard is missing, it falls back to
``vale`` + ``harper-cli`` and merges findings.

Exit codes: 0 clean (no actionable findings), 1 actionable findings remain,
2 tool/setup error.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

REPO = Path(__file__).resolve().parent.parent
BASE_URL = "https://willma.surf.nl/api/v0"
WILLMA_MODEL = "openai/gpt-oss-120b"
DEFAULT_PROFILE_CANDIDATES = (
    Path.home() / "Git/Github/Tools/proseguard/profiles/examples/prose-human.toml",
    REPO / "dot_local/share/omp-surf/prose-human.toml",
    Path(__file__).resolve().parent / "fixtures/prose/prose-human.toml",
)


@dataclass
class Finding:
    rule: str
    severity: str
    message: str
    target: str
    engine: str = ""
    line: int | None = None

    def actionable(self, floor: str = "warning") -> bool:
        order = {"suggestion": 0, "warning": 1, "error": 2}
        return order.get(self.severity.lower(), 1) >= order.get(floor.lower(), 1)


def resolve_token() -> str | None:
    for env in ("PROSEGUARD_WILLMA_TOKEN", "XAIT_WILLMA_TOKEN", "WILLMA_TOKEN", "SURF_AI_HUB_TOKEN"):
        v = os.environ.get(env, "").strip()
        if v:
            return v
    try:
        out = subprocess.run(
            ["pass", "show", "surf/ai-hub/token"],
            check=False,
            capture_output=True,
            text=True,
            timeout=30,
        )
        if out.returncode == 0:
            tok = (out.stdout or "").splitlines()[0].strip()
            if tok:
                return tok
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
        pass
    key = Path.home() / ".config/surf-ai-hub/api_key"
    if key.is_file():
        tok = key.read_text().strip()
        if tok:
            return tok
    return None


def find_proseguard() -> str | None:
    return shutil.which("proseguard") or (
        str(Path.home() / ".local/bin/proseguard")
        if (Path.home() / ".local/bin/proseguard").is_file()
        else None
    )


def find_profile(explicit: str | None) -> Path | None:
    if explicit:
        p = Path(explicit)
        return p if p.is_file() else None
    for c in DEFAULT_PROFILE_CANDIDATES:
        if c.is_file():
            return c
    return None


def run_proseguard_gate(
    path: Path,
    *,
    profile: Path,
    styles: Path | None,
) -> tuple[list[Finding], str, int]:
    pg = find_proseguard()
    if not pg:
        raise FileNotFoundError("proseguard binary not found")
    cmd = [pg, "--profile", str(profile), "--format", "json", str(path)]
    if styles and styles.is_dir():
        cmd[1:1] = ["--styles-path", str(styles)]
    proc = subprocess.run(cmd, check=False, capture_output=True, text=True)
    raw = (proc.stdout or "") + (proc.stderr or "")
    findings: list[Finding] = []
    # JSON is on stdout
    try:
        data = json.loads(proc.stdout or "{}")
    except json.JSONDecodeError:
        return findings, raw, 2 if proc.returncode not in (0, 1) else proc.returncode
    for f in data.get("findings") or []:
        loc = f.get("location") or {}
        findings.append(
            Finding(
                rule=str(f.get("rule") or ""),
                severity=str(f.get("severity") or "warning"),
                message=str(f.get("message") or ""),
                target=str(f.get("target") or path),
                engine=str(f.get("engine") or ""),
                line=loc.get("line"),
            )
        )
    return findings, raw, proc.returncode


def run_vale_harper_fallback(path: Path) -> tuple[list[Finding], str, int]:
    """Fallback gate: vale line output + harper-cli lint."""
    findings: list[Finding] = []
    blobs: list[str] = []
    code = 0
    vale = shutil.which("vale")
    if vale:
        proc = subprocess.run(
            [vale, "--output=line", str(path)],
            check=False,
            capture_output=True,
            text=True,
        )
        blobs.append(proc.stdout or "")
        blobs.append(proc.stderr or "")
        # vale: line format path:line:col:severity: message
        for line in (proc.stdout or "").splitlines():
            m = re.match(
                r"^(?P<path>.+?):(?P<line>\d+):(?:\d+:)?(?P<sev>\w+):\s*(?P<msg>.+)$",
                line,
            )
            if not m:
                continue
            sev = m.group("sev").lower()
            if sev not in ("error", "warning", "suggestion"):
                sev = "warning"
            findings.append(
                Finding(
                    rule="vale",
                    severity=sev,
                    message=m.group("msg"),
                    target=m.group("path"),
                    engine="vale",
                    line=int(m.group("line")),
                )
            )
        if proc.returncode not in (0, 1):
            code = 2
        elif proc.returncode == 1:
            code = 1
    harper = shutil.which("harper-cli") or shutil.which("harper")
    if harper:
        # harper-cli lint FILE --format json
        args = [harper, "lint", str(path), "--format", "json"]
        proc = subprocess.run(args, check=False, capture_output=True, text=True)
        blobs.append(proc.stdout or "")
        blobs.append(proc.stderr or "")
        try:
            data = json.loads(proc.stdout or "[]")
        except json.JSONDecodeError:
            data = []
        if isinstance(data, list):
            for file_block in data:
                for lint in file_block.get("lints") or []:
                    findings.append(
                        Finding(
                            rule=str(lint.get("rule") or "harper"),
                            severity="suggestion",
                            message=str(lint.get("message") or ""),
                            target=str(path),
                            engine="harper",
                            line=lint.get("line"),
                        )
                    )
    actionable = [f for f in findings if f.actionable("warning")]
    if code == 0 and actionable:
        code = 1
    return findings, "\n".join(blobs), code


def gate_file(
    path: Path,
    *,
    profile: str | None = None,
    styles: str | None = None,
    prefer_fallback: bool = False,
) -> tuple[list[Finding], str, int, str]:
    """Return findings, raw log, exit-ish code, backend name."""
    styles_path = Path(styles) if styles else Path.home() / ".config/vale/styles"
    prof = find_profile(profile)
    if not prefer_fallback and find_proseguard() and prof:
        try:
            findings, raw, code = run_proseguard_gate(path, profile=prof, styles=styles_path)
            return findings, raw, code, f"proseguard:{prof}"
        except Exception as e:  # noqa: BLE001
            # fall through
            fb_note = f"proseguard failed ({e}); falling back to vale+harper-cli\n"
    else:
        fb_note = ""
    findings, raw, code = run_vale_harper_fallback(path)
    return findings, fb_note + raw, code, "vale+harper-cli"


def format_findings(findings: list[Finding]) -> str:
    if not findings:
        return "(no findings)"
    lines = []
    for f in findings:
        loc = f":{f.line}" if f.line else ""
        lines.append(f"[{f.severity}] {f.rule}{loc} ({f.engine}): {f.message}")
    return "\n".join(lines)


def willma_rewrite(text: str, findings: list[Finding], token: str) -> str:
    system = (
        "You rewrite technical prose so it reads as natural human writing. "
        "Remove AI writing tells: stacked intensifiers, 'delve', 'robust/scalable/"
        "flexible' tricolons, 'it is important to note', 'furthermore/moreover' "
        "padding, 'cutting-edge', 'unlock the full potential', 'at the end of the day', "
        "and vague 'this approach' openers. Keep facts, numbers, and structure. "
        "Prefer short concrete sentences. Output ONLY the rewritten document, "
        "no preamble or code fences."
    )
    user = (
        "Lint findings to fix (style/grammar):\n"
        f"{format_findings(findings)}\n\n"
        "--- ORIGINAL ---\n"
        f"{text}\n"
        "--- END ---\n"
        "Rewrite the full document."
    )
    body = {
        "model": WILLMA_MODEL,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "max_tokens": 2000,
        "temperature": 0.4,
        "reasoning_effort": "low",
    }
    req = urllib.request.Request(
        BASE_URL.rstrip("/") + "/chat/completions",
        data=json.dumps(body).encode(),
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=180) as resp:
        payload = json.load(resp)
    content = (
        ((payload.get("choices") or [{}])[0].get("message") or {}).get("content") or ""
    ).strip()
    # strip accidental fences
    m = re.match(r"^```(?:markdown|md|org)?\s*\n([\s\S]*?)\n```\s*$", content)
    if m:
        content = m.group(1)
    return content


def findings_signature(findings: list[Finding]) -> set[str]:
    return {f"{f.rule}|{f.message}" for f in findings if f.actionable()}


# --- pure helpers for unit tests ---------------------------------------------

def actionable_count(findings: list[Finding], floor: str = "warning") -> int:
    return sum(1 for f in findings if f.actionable(floor))


def reduced(before: list[Finding], after: list[Finding]) -> bool:
    """True if actionable set shrank or count dropped."""
    b, a = findings_signature(before), findings_signature(after)
    if len(a) < len(b):
        return True
    return actionable_count(after) < actionable_count(before)


def build_rewrite_prompt_preview(text: str, findings: list[Finding]) -> str:
    """Pure: ensure findings are embedded for the model (tested without HTTP)."""
    block = format_findings(findings)
    return f"FINDINGS:\n{block}\n\nTEXT:\n{text}"


def cmd_gate(args: argparse.Namespace) -> int:
    path = Path(args.path)
    if not path.is_file():
        print(f"error: not a file: {path}", file=sys.stderr)
        return 2
    findings, raw, code, backend = gate_file(
        path, profile=args.profile, styles=args.styles_path
    )
    print(f"backend: {backend}")
    if args.json:
        print(
            json.dumps(
                {
                    "backend": backend,
                    "findings": [asdict(f) for f in findings],
                    "actionable": actionable_count(findings),
                },
                indent=2,
            )
        )
    else:
        print(format_findings(findings))
        print(
            f"summary: {len(findings)} finding(s), "
            f"{actionable_count(findings)} actionable; raw_exit={code}"
        )
    return 1 if actionable_count(findings) else 0


def cmd_fix(args: argparse.Namespace) -> int:
    path = Path(args.path)
    if not path.is_file():
        print(f"error: not a file: {path}", file=sys.stderr)
        return 2
    original = path.read_text()
    findings, raw, code, backend = gate_file(
        path, profile=args.profile, styles=args.styles_path
    )
    print(f"backend: {backend}")
    print(f"pre: {actionable_count(findings)} actionable / {len(findings)} total")
    print(format_findings(findings))
    if actionable_count(findings) == 0:
        print("clean: nothing to rewrite")
        return 0
    if args.no_llm or os.environ.get("OMP_PROSE_NO_LLM") in ("1", "true", "yes"):
        print("skip: rewrite disabled (--no-llm / OMP_PROSE_NO_LLM)")
        return 1
    token = resolve_token()
    if not token:
        print("skip: no Willma token (pass show surf/ai-hub/token)")
        return 1
    max_attempts = max(1, int(args.max_attempts))
    current = original
    work = path
    for attempt in range(1, max_attempts + 1):
        print(f"rewrite attempt {attempt}/{max_attempts} ...")
        try:
            rewritten = willma_rewrite(current, findings, token)
        except urllib.error.HTTPError as e:
            print(f"skip: Willma HTTP {e.code}: {e.read()[:200]!r}")
            return 1
        except Exception as e:  # noqa: BLE001
            print(f"skip: Willma error {type(e).__name__}: {e}")
            return 1
        if not rewritten.strip():
            print("skip: empty rewrite")
            return 1
        # write either in-place or to --out
        out_path = Path(args.out) if args.out else path
        out_path.write_text(rewritten if rewritten.endswith("\n") else rewritten + "\n")
        findings2, _, _, _ = gate_file(
            out_path, profile=args.profile, styles=args.styles_path
        )
        print(
            f"post-{attempt}: {actionable_count(findings2)} actionable / "
            f"{len(findings2)} total"
        )
        print(format_findings(findings2))
        if actionable_count(findings2) == 0:
            print("clean after rewrite")
            return 0
        if not reduced(findings, findings2) and attempt == max_attempts:
            print("no further reduction")
            return 1
        findings = findings2
        current = rewritten
        work = out_path
    return 1 if actionable_count(findings) else 0


def cmd_self_test() -> int:
    ok = True

    def check(name: str, cond: bool, detail: str = "") -> None:
        nonlocal ok
        if cond:
            print(f"SELF-TEST PASS {name}")
        else:
            ok = False
            print(f"SELF-TEST FAIL {name}: {detail}")

    f_warn = Finding("r.Weasel", "warning", "avoid very", "t.md", "lexical", 1)
    f_sug = Finding("r.Soft", "suggestion", "eprime", "t.md", "lexical", 2)
    f_err = Finding("r.AI", "error", "ai tell", "t.md", "lexical", 3)
    check("actionable-warn", actionable_count([f_warn, f_sug]) == 1)
    check("actionable-err", actionable_count([f_err, f_sug]) == 1)
    check("reduced-true", reduced([f_warn, f_err], [f_warn]))
    check("reduced-false", not reduced([f_warn], [f_warn, f_err]))
    preview = build_rewrite_prompt_preview("Hello very world", [f_warn])
    check("prompt-has-finding", "avoid very" in preview and "Hello very world" in preview)
    # fixtures exist relative to repo
    ai = REPO / "scripts/fixtures/prose/ai-tell.md"
    clean = REPO / "scripts/fixtures/prose/clean.md"
    check("fixture-ai", ai.is_file())
    check("fixture-clean", clean.is_file())
    return 0 if ok else 1


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)

    g = sub.add_parser("gate", help="run prose gate (style+grammar)")
    g.add_argument("path")
    g.add_argument("--json", action="store_true")
    g.add_argument("--profile", default=None)
    g.add_argument("--styles-path", default=None)

    f = sub.add_parser("fix", help="gate + Willma rewrite loop")
    f.add_argument("path")
    f.add_argument("--max-attempts", type=int, default=3)
    f.add_argument("--no-llm", action="store_true")
    f.add_argument("--out", default=None, help="write rewrite here (default: in-place)")
    f.add_argument("--profile", default=None)
    f.add_argument("--styles-path", default=None)

    sub.add_parser("self-test", help="pure unit tests")

    args = ap.parse_args(argv)
    if args.cmd == "gate":
        return cmd_gate(args)
    if args.cmd == "fix":
        return cmd_fix(args)
    if args.cmd == "self-test":
        return cmd_self_test()
    return 2


if __name__ == "__main__":
    sys.exit(main())
