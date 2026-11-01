"""Behavioral cleanup contracts in private temporary fixtures; no real trash use."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


SOURCE = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ["RG_SPACE_SWEEP_BIN"]).resolve()
REMOTE_SCRIPT = (SOURCE / "remote-sweep.sh").read_text()


RTRASH_STUB = r'''#!/usr/bin/python3
import json, os, pathlib, shutil, sys
args = sys.argv[1:]
scope = pathlib.Path(args[args.index("--trash-dir") + 1])
with open(os.environ["SWEEP_TEST_LOG"], "a") as log:
    log.write(json.dumps({"command": args[0], "scope": str(scope), "mode": scope.stat().st_mode & 0o777}) + "\n")
if args[0] == "put":
    if os.environ.get("SWEEP_TEST_FAIL_PUT"):
        sys.exit(3)
    source = pathlib.Path(args[-1])
    assert scope.parent == source.parent
    assert not list(scope.iterdir())
    (scope / "files").mkdir()
    (scope / "info").mkdir()
    shutil.move(str(source), str(scope / "files" / source.name))
    (scope / "info" / (source.name + ".trashinfo")).write_text("fixture")
elif args[0] == "empty":
    assert args[:3] == ["empty", "--plain", "--trash-dir"]
    assert scope.name.startswith(".artifact-trash.")
    for item in (scope / "files").iterdir():
        shutil.rmtree(item) if item.is_dir() else item.unlink()
    for item in (scope / "info").iterdir():
        item.unlink()
else:
    raise AssertionError(args)
print(json.dumps({"ok": 1, "failed": 0}))
'''


class CleanupContract(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="space-sweep-contract-")
        self.root = Path(self.temporary.name)
        self.home = self.root / "home"
        self.home.mkdir()
        self.project = self.home / "wt" / "project"
        self.project.mkdir(parents=True)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.log = self.root / "trash-calls.jsonl"
        self.env = os.environ.copy()
        self.env.update(HOME=str(self.home), PATH=f"{self.bin}:/usr/bin:/bin",
                        SWEEP_TEST_LOG=str(self.log), LC_ALL="C")
        for name in list(self.env):
            if name.startswith("GIT_") or name.startswith("SWEEP_TEST_FAIL"):
                self.env.pop(name)
        self.write_program("rtrash", RTRASH_STUB)
        self.write_program("lsof", "#!/bin/sh\nexit 1\n")

    def tearDown(self):
        self.temporary.cleanup()

    def write_program(self, name, text):
        target = self.bin / name
        target.write_text(text)
        target.chmod(0o755)
        return target

    def meson(self, name="release-build", source=None):
        build = self.project / name
        source = source or self.project / "source"
        source.mkdir(parents=True, exist_ok=True)
        (build / "meson-private").mkdir(parents=True)
        (build / "meson-info").mkdir()
        (build / "meson-private" / "coredata.dat").write_bytes(b"generated fixture")
        (build / "meson-info" / "meson-info.json").write_text(json.dumps(
            {"directories": {"source": str(source), "build": str(build)}}))
        (build / "object.o").write_bytes(b"object fixture")
        return build

    def cmake(self, name="cmake-custom", in_source=False):
        build = self.project / name
        (build / "CMakeFiles").mkdir(parents=True)
        source = build if in_source else self.project / "source"
        source.mkdir(exist_ok=True)
        (build / "CMakeCache.txt").write_text(
            f"CMAKE_HOME_DIRECTORY:INTERNAL={source}\nCMAKE_CACHEFILE_DIR:INTERNAL={build}\n")
        return build

    def launch(self, mode, *, remote=False, age=0, exclude=(), dry_run=False, yes=True):
        if not remote:
            args = [str(BINARY), mode, "--min-age-days", str(age), "--json", "all"]
            if mode == "clean" and yes: args.insert(2, "--yes")
            if dry_run: args.insert(2, "--dry-run")
            return subprocess.run(args, env=self.env, text=True, capture_output=True)
        prefix = ["set -euo pipefail", f"HOME_DIR={shlex_quote(str(self.home))}",
                  f"ROOTS=({shlex_quote(str(self.home / 'wt'))})", "PRUNE_PATHS=()",
                  "EXCLUDE_PATHS=(" + " ".join(shlex_quote(str(path)) for path in exclude) + ")",
                  "CATEGORIES='rust python tox pixi venv js go java native'", f"MODE={mode}",
                  "OUTPUT_FORMAT=json", f"DRY_RUN={int(dry_run)}", f"YES={int(yes)}", "LIMIT=50", f"MIN_AGE_DAYS={age}"]
        return subprocess.run(["/bin/bash"], input="\n".join(prefix) + "\n" + REMOTE_SCRIPT,
                              env=self.env, text=True, capture_output=True)

    def calls(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()] if self.log.exists() else []

    def assert_report_paths(self, remote, expected):
        result = self.launch("report", remote=remote)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        data = json.loads(result.stdout)
        paths = {item["path"] for item in data["top_paths"]}
        # Local report paths use ~; the remote report also renders home paths.
        expanded = {str(self.home) + path[1:] if path.startswith("~/") else path for path in paths}
        self.assertEqual(expanded, {str(path) for path in expected}, result.stdout)
        self.assertEqual(data["matched_paths"], len(expected))
        self.assertEqual(self.calls(), [])

    def test_marker_discovery_local_and_remote_custom_names_and_non_cargo_target(self):
        builds = [self.meson("target"), self.cmake("arbitrary-config")]
        self.assert_report_paths(False, builds)
        self.assert_report_paths(True, builds)

    def test_nested_native_build_is_counted_once(self):
        outer = self.meson()
        self.cmake("release-build/subproject/generated")
        self.assert_report_paths(False, [outer])
        self.assert_report_paths(True, [outer])

    def test_unmarked_build_named_directory_is_not_a_candidate(self):
        (self.project / "build").mkdir()
        (self.project / "build" / "source.cpp").write_text("int main() {}")
        self.assert_report_paths(False, [])
        self.assert_report_paths(True, [])

    def test_cmake_in_source_build_is_refused(self):
        build = self.cmake(in_source=True)
        for remote in (False, True):
            result = self.launch("clean", remote=remote)
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_malformed_generated_metadata_is_reportable_but_not_cleanable(self):
        build = self.meson()
        (build / "meson-info" / "meson-info.json").write_text("not json")
        self.assert_report_paths(False, [build])
        self.assert_report_paths(True, [build])
        for remote in (False, True):
            result = self.launch("clean", remote=remote)
            self.assertNotEqual(result.returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_tracked_content_is_refused(self):
        build = self.meson()
        self.write_program("git", "#!/bin/sh\nprintf 'tracked-source.cpp\\n'\n")
        for remote in (False, True):
            result = self.launch("clean", remote=remote)
            self.assertNotEqual(result.returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_inherited_git_locator_cannot_redirect_tracking_inspection(self):
        build = self.meson()
        self.env["GIT_DIR"] = "/different/index"
        self.write_program("git", "#!/bin/sh\nif [ -z \"${GIT_DIR-}\" ]; then printf 'tracked-source.cpp\\n'; fi\n")
        for remote in (False, True):
            self.assertNotEqual(self.launch("clean", remote=remote).returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_broken_enclosing_git_metadata_is_refused(self):
        build = self.meson()
        (self.project / ".git").write_text("gitdir: /nonexistent/worktree-metadata\n")
        for remote in (False, True):
            result = self.launch("clean", remote=remote)
            self.assertNotEqual(result.returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_nested_repository_is_refused(self):
        build = self.meson()
        (build / "dependency" / ".git").mkdir(parents=True)
        for remote in (False, True):
            result = self.launch("clean", remote=remote)
            self.assertNotEqual(result.returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_recent_native_build_is_retained_for_local_and_remote_age_filter(self):
        build = self.meson()
        for remote in (False, True):
            result = self.launch("report", remote=remote, age=7)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(json.loads(result.stdout)["matched_paths"], 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_maximum_age_does_not_overflow_remote_arithmetic(self):
        build = self.meson()
        result = self.launch("report", remote=True, age=2**64 - 1)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(json.loads(result.stdout)["matched_paths"], 0)
        self.assertTrue(build.exists())

    def test_excluded_descendant_protects_whole_remote_build(self):
        build = self.meson()
        result = self.launch("report", remote=True, exclude=[build / "object.o"])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(json.loads(result.stdout)["matched_paths"], 0)
        self.assertEqual(self.calls(), [])

    def test_open_artifact_and_unknown_inspection_are_refused(self):
        build = self.meson()
        for script in ("#!/bin/sh\nprintf '1234\\n'\nexit 0\n",
                       "#!/bin/sh\nprintf 'cannot inspect\\n' >&2\nexit 1\n"):
            self.write_program("lsof", script)
            for remote in (False, True):
                result = self.launch("clean", remote=remote)
                self.assertNotEqual(result.returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_isolated_purge_only_targets_this_operation_and_preserves_old_trash(self):
        previous = self.home / ".local/share/Trash/files/retained"
        previous.mkdir(parents=True)
        (previous / "important.txt").write_text("retain")
        for remote in (False, True):
            build = self.meson("remote" if remote else "local")
            result = self.launch("clean", remote=remote)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertFalse(build.exists())
            self.assertEqual((previous / "important.txt").read_text(), "retain")
        calls = self.calls()
        self.assertEqual([call["command"] for call in calls], ["put", "empty", "put", "empty"])
        for pair in (calls[:2], calls[2:]):
            self.assertEqual(pair[0]["scope"], pair[1]["scope"])
            self.assertEqual(pair[0]["mode"], 0o700)
            self.assertNotIn("Trash", Path(pair[0]["scope"]).parts)
        self.assertNotEqual(calls[0]["scope"], calls[2]["scope"])

    def test_failed_put_never_requests_purge(self):
        build = self.meson()
        self.env["SWEEP_TEST_FAIL_PUT"] = "1"
        result = self.launch("clean")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual([call["command"] for call in self.calls()], ["put"])

    def test_dry_run_never_invokes_rtrash(self):
        build = self.meson()
        for remote in (False, True):
            result = self.launch("clean", remote=remote, dry_run=True, yes=False)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_yes_is_required(self):
        build = self.meson()
        for remote in (False, True):
            self.assertNotEqual(self.launch("clean", remote=remote, yes=False).returncode, 0)
        self.assertTrue(build.exists())
        self.assertEqual(self.calls(), [])

    def test_existing_local_bin_rtrash_is_used_when_path_omits_it(self):
        installed = self.home / ".local/bin/rtrash"
        installed.parent.mkdir(parents=True)
        (self.bin / "rtrash").rename(installed)
        for remote in (False, True):
            build = self.meson("fallback-remote" if remote else "fallback-local")
            result = self.launch("clean", remote=remote)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertFalse(build.exists())
        self.assertEqual([call["command"] for call in self.calls()], ["put", "empty", "put", "empty"])


def shlex_quote(value):
    return "'" + value.replace("'", "'\\''") + "'"


if __name__ == "__main__":
    unittest.main()
