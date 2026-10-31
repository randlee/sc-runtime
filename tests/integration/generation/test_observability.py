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

try:
    from tests.integration.generation.owned_process import run as run_owned
except ModuleNotFoundError:
    from owned_process import run as run_owned

def owned_check_output(args, **kwargs):
    return run_owned(args, capture_output=True, check=True, **kwargs).stdout


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
        run_owned(
            ["just", "new", str(app), "--var-file", "wizard/fixtures/sqlite-mcp.json", "--development-observability"],
            cwd=ROOT, check=True, timeout=1800, stdin=subprocess.DEVNULL,
        )
        manifest = tomllib.loads((app / "crates/daemon/Cargo.toml").read_text())
        return app, manifest["package"]["name"]

    def run_generated_test(self, app, package, target, name, *, monty=False, ignored=False):
        cargo = ["cargo", "+1.96.1"] if monty else ["cargo"]
        result = run_owned(
            [*cargo, "test", "-p", package, "--features", "monty" if monty else "cron", *target, name, "--", "--exact", *(["--ignored"] if ignored else [])],
            cwd=app, check=True, timeout=900, stdin=subprocess.DEVNULL,
            text=True, capture_output=True,
        )
        self.assertIn("running 1 test", result.stdout, result.stdout + result.stderr)
        self.assertIn("test result: ok. 1 passed", result.stdout, result.stdout + result.stderr)

    def test_observability(self):
        app, package = self.generated_app()
        self.run_generated_test(app, package, ["--test", "observability"],
                                "production_http_and_cron_share_jsonl_and_release_handles")
        self.run_generated_test(app, package, ["--test", "cron_environment"],
                                "generated_policy_resolver_and_custom_registry_preserve_python_monty_snapshots", monty=True)

    def test_schedule_limit_mapping(self):
        app, package = self.generated_app()
        for name in ("zero_is_unlimited_and_the_default_is_ten", "json_null_is_unlimited", "configured_database_and_row_limits_reach_store"):
            with self.subTest(name=name):
                self.run_generated_test(app, package, ["--lib"], "cron::tests::" + name)

    def test_agent_error_inventory(self):
        app, package = self.generated_app()
        self.run_generated_test(app, package, ["--lib"],
                                "agent_error::tests::operation_remains_primary_and_shutdown_only_fails_success")
        self.run_generated_test(app, package, ["--bin", "widget-app-daemon"],
                                "tests::config_startup_uses_each_library_code_once")
        self.run_generated_test(app, package, ["--bin", "widget-app-daemon"],
                                "tests::runtime_failure_before_stores_releases_handles_without_waiting_for_worker")
        self.run_generated_test(app, package, ["--bin", "widget-app-daemon"],
                                "tests::lock_loser_leaves_fresh_cron_database_absent_and_never_starts_worker")
        self.run_generated_test(app, package, ["--bin", "widget-app-daemon"],
                                "tests::cron_startup_failure_preserves_exact_code_cause_and_recovery", ignored=True)

    def test_pr_fixture_matrix_one_entry_per_fixture(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text()
        fixtures = sorted(p.name for p in (ROOT / "wizard/fixtures").glob("*.json"))
        self.assertTrue(fixtures)
        self.assertIn('pathlib.Path("wizard/fixtures").glob("*.json")', workflow)
        self.assertIn('fixture: ${{ fromJson(needs.fixtures.outputs.names) }}', workflow)
        self.assertIn('wizard/fixtures/${{ matrix.fixture }}', workflow)
        actual = json.loads(owned_check_output(
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
            run_owned(
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
            run_owned(
                ["just", "new", str(app), "--var-file", "wizard/fixtures/sqlite-mcp.json", "--development-observability"],
                cwd=ROOT, check=True, timeout=1800, stdin=subprocess.DEVNULL,
            )
            def build_executable(command, cwd, name):
                result = run_owned(
                    [*command, "--message-format=json"], cwd=cwd, check=True,
                    timeout=900, stdin=subprocess.DEVNULL, text=True, capture_output=True,
                )
                executables = [message["executable"] for line in result.stdout.splitlines()
                               if (message := json.loads(line)).get("reason") == "compiler-artifact"
                               and message.get("target", {}).get("name") == name
                               and message.get("executable")]
                self.assertEqual(len(executables), 1, result.stdout + result.stderr)
                return Path(executables[0])

            binary = build_executable(
                ["cargo", "build", "-p", "widget-app-daemon", "--features", "cron"],
                app, "widget-app-daemon",
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
            parser_binary = build_executable(
                ["cargo", "build", "--manifest-path", str(parser / "Cargo.toml")],
                parser, "log-event-parser",
            )
            self.assertTrue(binary.is_file(), str(binary))
            for sig in (signal.SIGINT, signal.SIGTERM):
                with self.subTest(signal=sig.name):
                    runtime = root / sig.name
                    runtime.mkdir()
                    (runtime / "config").mkdir()
                    config = json.loads((app / "config/default.json").read_text())
                    config["instance_root"] = str(runtime)
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
                        run_owned([str(parser_binary)],
                                       stdin=log_input, check=True, timeout=30)
                    events = [json.loads(line) for line in logs[0].read_text().splitlines()]
                    self.assertTrue(events)
                    self.assertEqual({event["service"] for event in events}, {"widget-app"})
                    self.assertTrue(any(event["target"] == "cron.scheduler" and event["action"] == "logger.attached" for event in events), events)
                    self.assertTrue(any(event["target"] == "http.server" and event["action"].startswith("request.") for event in events), events)

    @unittest.skipUnless(os.environ.get("SC_RUNTIME_REGISTRY_PROOF") == "1",
                         "registry proof runs after the authorized publication")
    def test_release_mode_uses_registry(self):
        run_owned([sys.executable, "scripts/check_observability_release.py"],
                       cwd=ROOT, check=True, timeout=120, stdin=subprocess.DEVNULL)
        with tempfile.TemporaryDirectory(prefix="sc-runtime-registry-proof-") as temp:
            app = Path(temp) / "app"
            run_owned(
                [sys.executable, "scripts/new_project.py", str(app), "--var-file", "wizard/fixtures/sqlite-mcp.json", "--release"],
                cwd=ROOT, check=True, timeout=1800, stdin=subprocess.DEVNULL,
            )
            manifest = tomllib.loads((app / "Cargo.toml").read_text())
            self.assertNotIn("patch", manifest)
            package = tomllib.loads((app / "crates/daemon/Cargo.toml").read_text())["package"]["name"]
            metadata = json.loads(owned_check_output(
                ["cargo", "+1.96.1", "metadata", "--all-features", "--format-version", "1"],
                cwd=app, text=True, timeout=900, stdin=subprocess.DEVNULL,
            ))
            expected = {"sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron",
                        "sc-observability", "sc-observability-types"}
            packages = [p for p in metadata["packages"] if p["name"] in expected]
            self.assertEqual({p["name"] for p in packages}, expected)
            for item in packages:
                self.assertTrue((item["source"] or "").startswith("registry+"), item)
                expected_version = "1.5.1" if item["name"].startswith("sc-observability") else "0.1.1"
                self.assertEqual(item["version"], expected_version)
            for feature in ("cron", "monty"):
                run_owned(["cargo", "+1.96.1", "test", "-p", package, "--features", feature],
                               cwd=app, check=True, timeout=900, stdin=subprocess.DEVNULL)

    def test_pinned_publish_kit_inventory(self):
        pin = tomllib.loads((ROOT / "release/sc-publish-pin.toml").read_text())["sc-publish"]
        self.assertEqual(pin["repository"], "https://github.com/randlee/sc-publish")
        self.assertEqual(pin["revision"], "9677ab083e7dd3feb258235a342e321ca1b45d8d")
        temp = tempfile.TemporaryDirectory(prefix="sc-publish-inventory-")
        self.addCleanup(temp.cleanup)
        checkout = Path(temp.name)
        git_env = {**os.environ, "GIT_TERMINAL_PROMPT": "0"}
        run_owned(["git", "init", "--quiet", str(checkout)], check=True, timeout=30, env=git_env)
        run_owned(["git", "-C", str(checkout), "fetch", "--quiet", "--depth", "1", pin["repository"], pin["revision"]], check=True, timeout=120, env=git_env)
        run_owned(["git", "-C", str(checkout), "checkout", "--quiet", "--detach", "FETCH_HEAD"], check=True, timeout=30, env=git_env)
        actual = owned_check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True, timeout=30, env=git_env).strip()
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
        preflight = consumer["preflight_checks"]
        self.assertLess(next(i for i, check in enumerate(preflight) if check["id"] == "observability-registry-only"),
                        next(i for i, check in enumerate(preflight) if check["id"] == "release-contract-sources"))
        self.assertFalse(any("test_release_mode_uses_registry" in check["run"] for check in preflight),
                         "postpublication proof cannot block the publication preflight")
        command = ("SC_RUNTIME_REGISTRY_PROOF=1 python3 -m unittest "
                   "tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_mode_uses_registry")
        guide = (ROOT / "docs/development/sc-publish.md").read_text()
        self.assertIn("after all five publications", guide)
        self.assertLess(guide.index("after all five publications"), guide.index(command))
        plan = (ROOT / "docs/plans/phase-b/sprint-b-18-template-observability-integration.md").read_text()
        self.assertLess(plan.index("Deferred release validation (`run-h0h`)"), plan.index(command))
        run_owned([sys.executable, "-m", "unittest", "tests.integration.generation.test_driver.DriverTests.test_release_mode_skips_the_patch"], cwd=ROOT, check=True, timeout=30)
