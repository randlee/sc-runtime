"""The benchmark bridge preserves source identity, failures, and measured evidence."""

import base64
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "remote_benchmark_gate", ROOT / "scripts/benchmark/run_remote_gate.py"
)
BRIDGE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BRIDGE)
SHA = "a" * 40


class RemoteBenchmarkGateTests(unittest.TestCase):
    def remote_owner(self, sha=SHA, dirty="", status=0, measured=False):
        owned = mock.Mock()

        def run(command, **kwargs):
            if command[1:3] == ["-c", "import sc_compose"]:
                return subprocess.CompletedProcess(command, 0)
            if command[:2] == ["git", "rev-parse"]:
                return subprocess.CompletedProcess(command, 0, sha + "\n")
            if command[:2] == ["git", "status"]:
                return subprocess.CompletedProcess(command, 0, dirty)
            if command[0] != "git" and measured:
                reports = kwargs["cwd"] / "site/reports/benchmark-monty"
                reports.mkdir(parents=True)
                payload = {"git_sha": SHA, "report_html": "fresh.html", "status": "FAIL"}
                (reports / "fresh.campaign.json").write_text(json.dumps(payload))
                (reports / "fresh.html").write_text("<html>measured failure</html>")
            return subprocess.CompletedProcess(command, status if command[0] != "git" else 0)

        owned.run.side_effect = run
        return owned

    def test_wrong_source_or_dirty_checkout_never_runs_gate(self):
        for sha, dirty in (("b" * 40, ""), (SHA, " M Cargo.toml\n")):
            with self.subTest(sha=sha, dirty=dirty), tempfile.TemporaryDirectory() as directory:
                owned = self.remote_owner(sha=sha, dirty=dirty)
                with mock.patch.object(BRIDGE.socket, "gethostname", return_value="rand-m5.local"):
                    with self.assertRaises(RuntimeError):
                        BRIDGE.run_remote(SHA, Path(directory) / "source", owned)
                self.assertFalse(any("scripts/benchmark/check_gate.py" in call.args[0]
                                     for call in owned.run.call_args_list))

    def test_real_hostname_required_even_with_label_override(self):
        owned = self.remote_owner()
        with mock.patch.object(BRIDGE.socket, "gethostname", return_value="runner"), \
                mock.patch.dict(os.environ, {"SC_BENCH_HOST_LABEL": "rand-m5"}):
            with self.assertRaisesRegex(RuntimeError, "not the controlled host"):
                BRIDGE.run_remote(SHA, Path("unused"), owned)
        owned.run.assert_not_called()

    def test_success_without_fresh_evidence_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory, \
                mock.patch.object(BRIDGE.socket, "gethostname", return_value="rand-m5"), \
                contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(RuntimeError, "success without a campaign"):
                BRIDGE.run_remote(SHA, Path(directory) / "source", self.remote_owner())

    def test_exact_fetch_bounded_execution_and_measured_failure_retention(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source"
            owned = self.remote_owner(status=19, measured=True)
            output = io.StringIO()
            with mock.patch.object(BRIDGE.socket, "gethostname", return_value="rand-m5"), \
                    mock.patch.dict(os.environ, {
                        "SC_BENCH_HOST_LABEL": "spoof", "SC_BENCH_GATE_ALLOW_ANY_HOST": "1",
                    }), mock.patch.object(BRIDGE, "REMOTE_EVIDENCE_ROOT", Path(directory) / "remote-evidence"), \
                    contextlib.redirect_stdout(output):
                self.assertEqual(BRIDGE.run_remote(SHA, source, owned, "/prepared/venv/bin/python3"), 19)
            fetch = next(call for call in owned.run.call_args_list if call.args[0][1] == "fetch")
            self.assertEqual(fetch.args[0][-1], SHA)
            gate = next(call for call in owned.run.call_args_list
                        if "scripts/benchmark/check_gate.py" in call.args[0])
            self.assertEqual(gate.args[0][1], "scripts/benchmark/check_gate.py")
            self.assertEqual(gate.args[0][0], "/prepared/venv/bin/python3")
            self.assertTrue(gate.kwargs["env"]["PATH"].startswith("/prepared/venv/bin" + os.pathsep))
            self.assertEqual(gate.kwargs["timeout"], BRIDGE.GATE_TIMEOUT)
            self.assertNotIn("SC_BENCH_HOST_LABEL", gate.kwargs["env"])
            self.assertNotIn("SC_BENCH_GATE_ALLOW_ANY_HOST", gate.kwargs["env"])
            self.assertEqual(gate.kwargs["env"]["CARGO_TARGET_DIR"], str(source / "target"))
            remote_reports = list((Path(directory) / "remote-evidence").glob("*/run-*/site/reports/benchmark-monty/fresh.campaign.json"))
            self.assertEqual(len(remote_reports), 1)
            destination = Path(directory) / "evidence"
            with contextlib.redirect_stdout(io.StringIO()):
                BRIDGE.save_reports(output.getvalue(), SHA, destination)
                BRIDGE.save_reports(output.getvalue(), SHA, destination)
            saved = sorted(destination.glob("run-*/site/reports/benchmark-monty/fresh.campaign.json"))
            self.assertEqual(len(saved), 2, "a repeat transfer must not overwrite evidence")
            for report in saved:
                self.assertEqual(json.loads(report.read_text())["status"], "FAIL")
                self.assertEqual(report.with_name("fresh.html").read_text(), "<html>measured failure</html>")

    def test_artifact_source_traversal_and_unpaired_html_fail_closed(self):
        campaign = base64.b64encode(json.dumps({
            "git_sha": SHA, "report_html": "missing.html", "status": "MEASURED",
        }).encode()).decode()
        for source, files in (
            ("b" * 40, {}),
            (SHA, {"site/reports/benchmark-monty/../../escape.html": ""}),
            (SHA, {"site/reports/benchmark-monty/fresh.campaign.json": campaign}),
        ):
            with tempfile.TemporaryDirectory() as directory:
                destination = Path(directory) / "evidence"
                output = BRIDGE.ARTIFACT_PREFIX + json.dumps({"source_sha": source, "files": files})
                with self.assertRaises(ValueError):
                    BRIDGE.save_reports(output, SHA, destination)
                self.assertFalse(destination.exists())

    def test_transfer_limit_keeps_remote_incomplete_campaign_and_html(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reports = root / "source/site/reports/benchmark-monty"
            reports.mkdir(parents=True)
            (reports / "fresh.campaign.json").write_text(json.dumps({
                "git_sha": SHA, "report_html": "fresh.html", "status": "INCOMPLETE",
            }))
            (reports / "fresh.html").write_text("<html>incomplete measurement</html>")
            with mock.patch.object(BRIDGE, "MAX_ARTIFACT_BYTES", 0), \
                    mock.patch.object(BRIDGE, "REMOTE_EVIDENCE_ROOT", root / "retained"), \
                    contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(ValueError, "transfer bound"):
                    BRIDGE.emit_reports(root / "source", set(), SHA)
            saved = list((root / "retained").glob("*/run-*/site/reports/benchmark-monty/fresh.campaign.json"))
            self.assertEqual(len(saved), 1)
            self.assertEqual(json.loads(saved[0].read_text())["status"], "INCOMPLETE")
            self.assertTrue(saved[0].with_name("fresh.html").is_file())

    def test_ssh_failure_and_timeout_are_not_passes(self):
        for failure, expected in ((255, 255), (subprocess.TimeoutExpired("ssh", 1), 124)):
            owned = mock.Mock()

            def run(command, **kwargs):
                if command[:2] == ["git", "rev-parse"]:
                    return subprocess.CompletedProcess(command, 0, SHA)
                if command[:2] == ["git", "status"]:
                    return subprocess.CompletedProcess(command, 0, "")
                request = json.load(kwargs["stdin"])
                self.assertEqual(request["sha"], SHA)
                self.assertIn("def run(", request["helper"])
                self.assertIn("StrictHostKeyChecking=yes", command)
                self.assertIn("BatchMode=yes", command)
                self.assertEqual(command[-2], "atmbench@rand-m5")
                self.assertEqual(kwargs["timeout"], BRIDGE.SSH_TIMEOUT)
                if isinstance(failure, Exception):
                    raise failure
                return subprocess.CompletedProcess(command, failure, "", "SSH unavailable\n")

            owned.run.side_effect = run
            with mock.patch.object(BRIDGE, "load_owned", return_value=owned), \
                    contextlib.redirect_stderr(io.StringIO()), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(BRIDGE.main(), expected)

    def test_bootstrap_cleans_only_owned_checkout_on_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "remote-path"
            unrelated = Path(directory) / "user-checkout"
            unrelated.mkdir()
            (unrelated / "keep").write_text("user data")
            bridge = (
                "def install_signal_handlers(): pass\n"
                "def run_remote(sha, source, owned, python):\n"
                "    source.mkdir()\n"
                f"    source.with_name('sentinel').write_text('owned')\n"
                f"    __import__('pathlib').Path({str(marker)!r}).write_text(str(source.parent))\n"
                "    raise RuntimeError('gate setup failed')\n"
            )
            result = subprocess.run(
                [sys.executable, "-c", BRIDGE.BOOTSTRAP], input=json.dumps({
                    "sha": SHA, "helper": "", "bridge": bridge,
                }), capture_output=True, text=True, timeout=10,
            )
            self.assertEqual(result.returncode, 1)
            self.assertIn("gate setup failed", result.stderr)
            self.assertFalse(Path(marker.read_text()).exists())
            self.assertEqual((unrelated / "keep").read_text(), "user data")

    def test_branch_names_and_shell_payloads_are_not_source_shas(self):
        for value in ("develop", SHA[:7], SHA + ";exit 0", "A" * 40):
            with self.assertRaises(ValueError):
                BRIDGE.validate_sha(value)

    def test_missing_prepared_python_dependency_is_actionable_and_does_not_fetch(self):
        owned = mock.Mock()
        owned.run.return_value = subprocess.CompletedProcess([], 1)
        with mock.patch.object(BRIDGE.socket, "gethostname", return_value="rand-m5"):
            with self.assertRaisesRegex(RuntimeError, "prepare ~/.cache/sc-runtime/benchmark-python"):
                BRIDGE.run_remote(SHA, Path("unused"), owned)
        self.assertEqual(owned.run.call_count, 1)
        self.assertEqual(owned.run.call_args.args[0][1:], ["-c", "import sc_compose"])

    def test_remote_interpreter_override_is_quoted_and_relative_paths_rejected(self):
        self.assertIn("$HOME/.cache/sc-runtime/benchmark-python/bin/python3", BRIDGE.remote_command(None))
        self.assertIn("else exec python3", BRIDGE.remote_command(None))
        self.assertTrue(BRIDGE.remote_command("/prepared venv/bin/python3").startswith("exec '/prepared venv/bin/python3' -c "))
        with self.assertRaises(ValueError):
            BRIDGE.remote_command("relative/python3")
