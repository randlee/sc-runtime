"""Prepublication contracts for the observability release and publishing kit."""

import importlib.util
import json
import os
import signal
import shlex
import shutil
import socket
import time
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path, PureWindowsPath

ROOT = Path(__file__).resolve().parents[3]
SIGNAL_MARKERS = {
    "SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST": "1",
    "GITHUB_ACTIONS": "true",
    "RUNNER_ENVIRONMENT": "github-hosted",
}


def signal_test_enabled(environment):
    if not environment.get("SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST"):
        return False
    if any(environment.get(key) != value for key, value in SIGNAL_MARKERS.items()):
        raise ValueError("signal test requires all isolated GitHub-hosted runner markers")
    return True


class GeneratedObservabilityIntegration(unittest.TestCase):
    def generated_app(self):
        temp = tempfile.TemporaryDirectory(prefix="sc-runtime-generated-test-")
        self.addCleanup(temp.cleanup)
        app = Path(temp.name) / "app"
        subprocess.run(
            ["just", "new", str(app), "--var-file", "wizard/fixtures/sqlite-mcp.json", "--development-observability"],
            cwd=ROOT, check=True, timeout=1800, stdin=subprocess.DEVNULL,
        )
        manifest = tomllib.loads((app / "crates/daemon/Cargo.toml").read_text())
        return app, manifest["package"]["name"]

    def run_generated_test(self, app, package, target, name):
        result = subprocess.run(
            ["cargo", "test", "-p", package, "--features", "cron", *target, name, "--", "--exact"],
            cwd=app, check=True, timeout=900, stdin=subprocess.DEVNULL,
            text=True, capture_output=True,
        )
        self.assertIn("running 1 test", result.stdout, result.stdout + result.stderr)
        self.assertIn("test result: ok. 1 passed", result.stdout, result.stdout + result.stderr)

    def test_observability(self):
        app, package = self.generated_app()
        self.run_generated_test(app, package, ["--test", "observability"],
                                "production_http_and_cron_share_jsonl_and_release_handles")

    def test_schedule_limit_mapping(self):
        app, package = self.generated_app()
        for name in ("zero_is_unlimited_and_the_default_is_ten", "json_null_is_unlimited", "configured_database_and_row_limits_reach_store"):
            with self.subTest(name=name):
                self.run_generated_test(app, package, ["--lib"], "cron::tests::" + name)

    def test_pr_fixture_matrix_one_entry_per_fixture(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text()
        fixtures = sorted(p.name for p in (ROOT / "wizard/fixtures").glob("*.json"))
        self.assertTrue(fixtures)
        self.assertIn('pathlib.Path("wizard/fixtures").glob("*.json")', workflow)
        self.assertIn('fixture: ${{ fromJson(needs.fixtures.outputs.names) }}', workflow)
        self.assertIn('wizard/fixtures/${{ matrix.fixture }}', workflow)
        actual = json.loads(subprocess.check_output(
            [sys.executable, "-c", 'import json, pathlib; print(json.dumps(sorted(path.name for path in pathlib.Path("wizard/fixtures").glob("*.json"))))'],
            cwd=ROOT, text=True, timeout=10,
        ))
        self.assertEqual(actual, fixtures)
        self.assertEqual(len(actual), len(set(actual)))
        self.assertIn("sqlite-mcp.json", fixtures)

    def test_sqlite_mcp_no_edits_sequence(self):
        real_just = shutil.which("just")
        self.assertIsNotNone(real_just)
        with tempfile.TemporaryDirectory(prefix="sc-runtime-no-edits-") as temp:
            root = Path(temp)
            app = root / "app"
            wrapper_dir = root / "bin"
            wrapper_dir.mkdir()
            helper = ROOT / "tests/integration/generation/snapshot_just.py"
            if os.name == "nt":
                wrapper = wrapper_dir / "just.cmd"
                wrapper.write_text(f'@"{sys.executable}" "{helper}" %*\r\n')
            else:
                wrapper = wrapper_dir / "just"
                wrapper.write_text(f"#!/bin/sh\nexec {shlex.quote(sys.executable)} {shlex.quote(str(helper))} \"$@\"\n")
                wrapper.chmod(0o755)
            log = root / "snapshots.jsonl"
            env = {**os.environ, "PATH": str(wrapper_dir) + os.pathsep + os.environ.get("PATH", ""),
                   "SC_TEST_REAL_JUST": real_just, "SC_TEST_SNAPSHOT_LOG": str(log)}
            subprocess.run(
                [real_just, "new", str(app), "--var-file", "wizard/fixtures/sqlite-mcp.json", "--development-observability"],
                cwd=ROOT, env=env, check=True, timeout=1800, stdin=subprocess.DEVNULL,
            )
            records = [json.loads(line) for line in log.read_text().splitlines()]
            self.assertEqual([record["recipe"] for record in records],
                             [[name] for name in ("setup", "lint", "test", "build-cli", "build-daemon")])
            baseline = records[0]["before"]
            self.assertTrue(baseline)
            # Cargo may create a lockfile during lint when the template has
            # none. Once created, it too must stay unchanged.
            generated_lock = None
            for record in records:
                for moment in ("before", "after"):
                    current = dict(record[moment])
                    if "Cargo.lock" not in baseline:
                        lock = current.pop("Cargo.lock", None)
                        if generated_lock is not None:
                            self.assertEqual(lock, generated_lock, record["recipe"])
                        elif lock is not None:
                            generated_lock = lock
                    self.assertEqual(baseline, current, record["recipe"])

    def test_signal_guard_refuses_partial_markers(self):
        self.assertFalse(signal_test_enabled({}))
        self.assertFalse(signal_test_enabled({"GITHUB_ACTIONS": "true"}))
        for key in SIGNAL_MARKERS:
            partial = dict(SIGNAL_MARKERS)
            if key == "SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST":
                partial[key] = "wrong"
            else:
                del partial[key]
            with self.subTest(missing=key), self.assertRaises(ValueError):
                signal_test_enabled(partial)
        self.assertTrue(signal_test_enabled(SIGNAL_MARKERS))

    def test_signal_shutdown_isolated_ci(self):
        if not signal_test_enabled(os.environ):
            self.skipTest("real signals require the dedicated GitHub-hosted job")
        import fcntl

        with tempfile.TemporaryDirectory(prefix="sc-runtime-lifecycle-") as temp:
            root = Path(temp)
            app = root / "app"
            subprocess.run(
                ["just", "new", str(app), "--var-file", "wizard/fixtures/sqlite-mcp.json", "--development-observability"],
                cwd=ROOT, check=True, timeout=1800, stdin=subprocess.DEVNULL,
            )
            subprocess.run(
                ["cargo", "build", "-p", "widget-app-daemon", "--features", "cron"],
                cwd=app, check=True, timeout=900, stdin=subprocess.DEVNULL,
            )
            # Deserialize every output line with the actual released LogEvent type.
            parser = root / "log-event-parser"
            (parser / "src").mkdir(parents=True)
            (parser / "Cargo.toml").write_text(
                '[package]\nname = "log-event-parser"\nversion = "0.0.0"\nedition = "2024"\n'
                '[dependencies]\nsc-observability-types = "=1.5.0"\nserde_json = "1"\n'
            )
            (parser / "src/main.rs").write_text(
                'use std::io::{self, BufRead};\n'
                'fn main() { for line in io::stdin().lock().lines() { '
                'let line = line.unwrap(); let _: sc_observability_types::LogEvent = '
                'serde_json::from_str(&line).expect("canonical LogEvent"); } }\n'
            )
            subprocess.run(["cargo", "build", "--manifest-path", str(parser / "Cargo.toml")],
                           check=True, timeout=900, stdin=subprocess.DEVNULL)
            binary = app / "target/debug/widget-app-daemon"
            self.assertTrue(binary.is_file(), str(binary))
            for sig in (signal.SIGINT, signal.SIGTERM):
                with self.subTest(signal=sig.name):
                    runtime = root / sig.name
                    runtime.mkdir()
                    (runtime / "config").mkdir()
                    config = json.loads((app / "config/default.json").read_text())
                    config["daemon"] = {"instance_root": str(runtime)}
                    config["logging"] = {"enableConsoleSink": False, "enableFileSink": True, "logRoot": str(runtime)}
                    (runtime / "config/default.json").write_text(json.dumps(config))
                    sock = runtime / "daemon.sock"
                    output = runtime / "daemon-output.txt"
                    with output.open("wb") as stream:
                        child = subprocess.Popen([str(binary)], cwd=runtime, stdin=subprocess.DEVNULL, stdout=stream, stderr=subprocess.STDOUT)
                        try:
                            deadline = time.monotonic() + 30
                            while True:
                                if child.poll() is not None:
                                    self.fail(f"daemon exited before readiness: {output.read_text(errors='replace')}")
                                try:
                                    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                                        client.settimeout(1)
                                        client.connect(str(sock))
                                        client.sendall(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                                        response = client.recv(4096)
                                    if b" 200 " in response:
                                        break
                                except (OSError, TimeoutError):
                                    pass
                                if time.monotonic() >= deadline:
                                    self.fail(f"health readiness timed out: {output.read_text(errors='replace')}")
                                time.sleep(0.05)
                            for lock in (runtime / "daemon.lock", Path(str(sock) + ".lock")):
                                self.assertTrue(lock.is_file(), str(lock))
                                with lock.open("r+") as lock_file:
                                    with self.assertRaises(BlockingIOError, msg=str(lock)):
                                        fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
                            child.send_signal(sig)
                            self.assertEqual(child.wait(timeout=30), 0, output.read_text(errors="replace"))
                        finally:
                            if child.poll() is None:
                                child.kill()
                                child.wait(timeout=10)
                    self.assertFalse(sock.exists(), "socket was not removed")
                    for lock in (runtime / "daemon.lock", Path(str(sock) + ".lock")):
                        self.assertTrue(lock.is_file(), str(lock))
                        with lock.open("r+") as lock_file:
                            fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
                            fcntl.flock(lock_file, fcntl.LOCK_UN)
                    logs = list((runtime / "logs").glob("*.jsonl"))
                    self.assertEqual(len(logs), 1, str(logs))
                    with logs[0].open("rb") as log_input:
                        subprocess.run([str(parser / "target/debug/log-event-parser")],
                                       stdin=log_input, check=True, timeout=30)
                    events = [json.loads(line) for line in logs[0].read_text().splitlines()]
                    self.assertTrue(events)
                    self.assertEqual({event["service"] for event in events}, {"widget-app"})
                    self.assertTrue(any(event["target"] == "cron.scheduler" and event["action"] == "logger.attached" for event in events), events)
                    self.assertTrue(any(event["target"] == "http.server" and event["action"].startswith("request.") for event in events), events)

    def test_pinned_publish_kit_inventory(self):
        pin = tomllib.loads((ROOT / "release/sc-publish-pin.toml").read_text())["sc-publish"]
        self.assertEqual(pin["repository"], "https://github.com/randlee/sc-publish")
        self.assertEqual(pin["revision"], "9677ab083e7dd3feb258235a342e321ca1b45d8d")
        temp = tempfile.TemporaryDirectory(prefix="sc-publish-inventory-")
        self.addCleanup(temp.cleanup)
        checkout = Path(temp.name)
        git_env = {**os.environ, "GIT_TERMINAL_PROMPT": "0"}
        subprocess.run(["git", "init", "--quiet", str(checkout)], check=True, timeout=30, env=git_env)
        subprocess.run(["git", "-C", str(checkout), "fetch", "--quiet", "--depth", "1", pin["repository"], pin["revision"]], check=True, timeout=120, env=git_env)
        subprocess.run(["git", "-C", str(checkout), "checkout", "--quiet", "--detach", "FETCH_HEAD"], check=True, timeout=30, env=git_env)
        actual = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True, timeout=30, env=git_env).strip()
        self.assertEqual(actual, pin["revision"])
        installer = checkout / "plugins/sc-publish/install.py"
        spec = importlib.util.spec_from_file_location("pinned_publish_installer", installer)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        copies = module.install_inventory(ROOT)
        paths = [relative for _, relative in copies] + list(module.TEMPLATES.values())
        self.assertEqual(len(paths), 85)
        self.assertEqual(len(paths), len(set(paths)))
        for path in paths:
            self.assertFalse(path.is_absolute() or ".." in path.parts or "\\" in str(path))
            self.assertFalse(PureWindowsPath(str(path)).drive)
            self.assertNotEqual(path.parts[:2], ("docs", "sprints"))
            current = ROOT
            for part in path.parts:
                current = current / part
                self.assertFalse(current.is_symlink(), str(path))
            self.assertTrue(current.is_file(), str(path))
        for source, relative in copies:
            self.assertFalse(source.is_symlink(), str(source))
            self.assertEqual(source.read_bytes(), (ROOT / relative).read_bytes(), str(relative))

    def test_release_contract_sources(self):
        manifest = tomllib.loads((ROOT / "template/Cargo.toml").read_text())
        dependencies = manifest["workspace"]["dependencies"]
        for name in ("sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron"):
            self.assertEqual(dependencies[name]["version"], "0.1.1")
            self.assertNotIn("path", dependencies[name])
        for name in ("sc-observability", "sc-observability-types"):
            self.assertEqual(dependencies[name]["version"], "1.5.1")
            self.assertNotIn("path", dependencies[name])
        self.assertNotIn("patch", manifest)
        consumer = json.loads((ROOT / "release/sc-publish-consumer-input.json").read_text())
        crates = sorted((c for c in consumer["crates"] if c["publish"]), key=lambda c: c["publish_order"])
        self.assertEqual([c["package"] for c in crates], ["sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron"])
        self.assertEqual([c["publish_order"] for c in crates], [1, 2, 3, 4, 5])
        checks = {c["id"]: c["run"] for c in consumer["preflight_checks"]}
        self.assertEqual(checks["observability-registry-only"], "python3 scripts/check_observability_release.py")
        self.assertIn("test_release_contract_sources", checks["release-contract-sources"])
        subprocess.run([sys.executable, "-m", "unittest", "tests.integration.generation.test_driver.DriverTests.test_release_mode_skips_the_patch"], cwd=ROOT, check=True)
