#!/usr/bin/env python3
"""Run the fresh release benchmark gate in an isolated checkout on rand-m5."""

from __future__ import annotations

import base64
import importlib.util
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import socket
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
OWNED_HELPER = ROOT / "tests/integration/generation/owned_process.py"
SSH_TARGET = "atmbench@rand-m5"
REPOSITORY = "https://github.com/randlee/sc-runtime.git"
SETUP_TIMEOUT = 120
GATE_TIMEOUT = 1800
SSH_TIMEOUT = 3600
ARTIFACT_PREFIX = "benchmark-ssh-artifacts: "
MAX_ARTIFACT_BYTES = 8 * 1024 * 1024
REMOTE_EVIDENCE_ROOT = Path.home() / ".local/state/sc-runtime/benchmark-preflight"

# Transfer the existing command-ownership helper with this consumer script.
# The remote temporary directory owns both the helper and the fresh checkout.
BOOTSTRAP = """
import importlib.util, json, pathlib, subprocess, sys, tempfile
def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
try:
    request = json.load(sys.stdin)
    with tempfile.TemporaryDirectory(prefix='sc-runtime-benchmark-') as temporary:
        root = pathlib.Path(temporary)
        (root / 'owned_process.py').write_text(request['helper'])
        (root / 'bridge.py').write_text(request['bridge'])
        owned = load('benchmark_owned', root / 'owned_process.py')
        bridge = load('benchmark_bridge', root / 'bridge.py')
        bridge.install_signal_handlers()
        result = bridge.run_remote(request['sha'], root / 'source', owned, request.get('python'))
    sys.exit(result)
except subprocess.TimeoutExpired:
    print('benchmark-ssh: controlled-host command timed out', file=sys.stderr)
    sys.exit(124)
except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
    print(f'benchmark-ssh: controlled-host gate failed: {error}', file=sys.stderr)
    sys.exit(1)
"""


def install_signal_handlers() -> None:
    """Unwind owned command sessions and temporary directories on cancellation."""
    def stop(number, _frame):
        raise SystemExit(128 + number)

    for name in ("SIGHUP", "SIGTERM", "SIGINT"):
        if hasattr(signal, name):
            signal.signal(getattr(signal, name), stop)


def validate_sha(sha: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("benchmark source must be an exact full commit SHA")


def emit_reports(source: Path, previous: set[Path], sha: str) -> int:
    """Transfer new canonical JSON/HTML pairs, including measured failures."""
    reports = sorted(set(source.glob("site/reports/benchmark-*/*.campaign.json")) - previous)
    if not reports:
        return 0
    # A broken connection or transfer limit must not destroy a measured campaign.
    destination = REMOTE_EVIDENCE_ROOT / sha
    destination.mkdir(parents=True, exist_ok=True)
    retained = Path(tempfile.mkdtemp(prefix="run-", dir=destination))
    for report in reports:
        for path in (report, report.with_name(report.name.removesuffix(".campaign.json") + ".html")):
            if path.is_symlink():
                raise ValueError("benchmark artifacts must be regular files, not symlinks")
            target = retained / path.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
    # Complete every pair before touching a potentially disconnected SSH pipe.
    print(f"benchmark-ssh: retained remote evidence at {retained}", flush=True)
    files = {}
    size = 0
    for report in reports:
        campaign = json.loads(report.read_text())
        html_name = campaign["report_html"]
        if Path(html_name).name != html_name or not html_name.endswith(".html"):
            raise ValueError("campaign HTML must be a sibling filename")
        for path in (report, report.parent / html_name):
            size += path.stat().st_size
            if size > MAX_ARTIFACT_BYTES:
                raise ValueError("fresh benchmark artifacts exceed the 8 MiB transfer bound")
            files[path.relative_to(source).as_posix()] = base64.b64encode(path.read_bytes()).decode()
    if files:
        print(ARTIFACT_PREFIX + json.dumps({"source_sha": sha, "files": files}), flush=True)
    return len(reports)


def save_reports(output: str, sha: str, destination: Path) -> set[str]:
    """Preserve transferred pairs in a new immutable evidence directory."""
    statuses = set()
    for line in output.splitlines():
        if not line.startswith(ARTIFACT_PREFIX):
            continue
        artifact = json.loads(line.removeprefix(ARTIFACT_PREFIX))
        if artifact["source_sha"] != sha:
            raise ValueError("benchmark artifact source does not match requested SHA")
        files = {}
        size = 0
        for relative, encoded in artifact["files"].items():
            path = Path(relative)
            if (len(path.parts) != 4 or path.parts[:2] != ("site", "reports")
                    or not path.parts[2].startswith("benchmark-") or ".." in path.parts
                    or "\\" in relative or not path.name.endswith((".campaign.json", ".html"))):
                raise ValueError("invalid benchmark artifact destination")
            content = base64.b64decode(encoded, validate=True)
            size += len(content)
            if size > MAX_ARTIFACT_BYTES:
                raise ValueError("benchmark artifacts exceed the 8 MiB transfer bound")
            files[path] = content
        for path, content in files.items():
            if path.name.endswith(".campaign.json"):
                campaign = json.loads(content)
                if (not isinstance(campaign, dict) or campaign.get("git_sha") != sha
                        or not isinstance(campaign.get("report_html"), str)
                        or path.parent / campaign["report_html"] not in files):
                    raise ValueError("benchmark campaign source or HTML pair is invalid")
        destination.mkdir(parents=True, exist_ok=True)
        evidence = Path(tempfile.mkdtemp(prefix="run-", dir=destination))
        for path, content in files.items():
            (evidence / path).parent.mkdir(parents=True, exist_ok=True)
            (evidence / path).write_bytes(content)
        # Keep all bytes before validation/logging can reject a measured result.
        for path, content in files.items():
            if path.name.endswith(".campaign.json"):
                campaign = json.loads(content)
                status = campaign.get("status")
                if not isinstance(status, str) or status not in {"PASS", "MEASURED", "FAIL", "INCOMPLETE", "SKIPPED"}:
                    raise ValueError(f"invalid benchmark campaign status in {evidence / path}")
                statuses.add(status)
                print(f"benchmark-ssh: retained source={sha} status={status} "
                      f"campaign={evidence / path}", flush=True)
        print(f"benchmark-ssh: retained JSON/HTML evidence at {evidence}", flush=True)
    return statuses


def run_remote(sha: str, source: Path, owned, python: str | None = None) -> int:
    """Fetch only the requested source, measure there, and relay its real status."""
    validate_sha(sha)
    if socket.gethostname().split(".")[0] != "rand-m5":
        raise RuntimeError("SSH endpoint is not the controlled host rand-m5")
    env = dict(os.environ)
    for name in ("SC_BENCH_HOST_LABEL", "SC_BENCH_GATE_ALLOW_ANY_HOST"):
        env.pop(name, None)
    env["GIT_TERMINAL_PROMPT"] = "0"
    env["CARGO_TARGET_DIR"] = str(source / "target")
    interpreter = python or sys.executable
    if not Path(interpreter).is_absolute():
        raise ValueError("SC_BENCH_PYTHON must be an absolute remote interpreter path")
    env["PATH"] = str(Path(interpreter).parent) + os.pathsep + env.get("PATH", "")
    prerequisite = owned.run(
        [interpreter, "-c", "import sc_compose"], env=env,
        timeout=SETUP_TIMEOUT, check=False,
    )
    if prerequisite.returncode:
        raise RuntimeError(
            "benchmark Python needs sc_compose; prepare ~/.cache/sc-runtime/benchmark-python "
            "or set SC_BENCH_PYTHON to an existing remote virtual-environment bin/python3"
        )

    def git(*args, capture_output=False):
        return owned.run(
            ["git", *args], cwd=source, env=env, timeout=SETUP_TIMEOUT,
            check=True, text=True, capture_output=capture_output,
        )

    source.mkdir()
    git("init", "--quiet")
    git("remote", "add", "origin", REPOSITORY)
    git("fetch", "--quiet", "--depth", "1", "origin", sha)
    git("checkout", "--quiet", "--detach", "FETCH_HEAD")
    if git("rev-parse", "HEAD", capture_output=True).stdout.strip() != sha:
        raise RuntimeError("controlled-host checkout does not match requested source SHA")
    if git("status", "--porcelain", capture_output=True).stdout.strip():
        raise RuntimeError("controlled-host checkout is not clean")
    print(f"benchmark-ssh: fresh gate on {SSH_TARGET} source={sha}", flush=True)
    previous = set(source.glob("site/reports/benchmark-*/*.campaign.json"))
    try:
        result = owned.run(
            [interpreter, "scripts/benchmark/check_gate.py"], cwd=source,
            env=env, timeout=GATE_TIMEOUT, check=False,
        )
    finally:
        written = emit_reports(source, previous, sha)
    if result.returncode == 0 and not written:
        raise RuntimeError("fresh benchmark gate returned success without a campaign")
    if git("rev-parse", "HEAD", capture_output=True).stdout.strip() != sha:
        raise RuntimeError("controlled-host source SHA changed during the gate")
    return result.returncode if result.returncode >= 0 else 128 - result.returncode


def load_owned():
    spec = importlib.util.spec_from_file_location("benchmark_owned", OWNED_HELPER)
    owned = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owned)
    return owned


def remote_command(python: str | None) -> str:
    """Select the operator's prepared Python before importing bridge/helper code."""
    bootstrap = " -c " + shlex.quote(BOOTSTRAP)
    if python:
        if not Path(python).is_absolute():
            raise ValueError("SC_BENCH_PYTHON must be an absolute remote interpreter path")
        return "exec " + shlex.quote(python) + bootstrap
    prepared = '"$HOME/.cache/sc-runtime/benchmark-python/bin/python3"'
    return (f"if [ -x {prepared} ]; then exec {prepared}{bootstrap}; "
            f"else exec python3{bootstrap}; fi")


def main() -> int:
    owned = load_owned()
    try:
        sha = owned.run(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, timeout=30,
            capture_output=True, text=True, check=True,
        ).stdout.strip()
        validate_sha(sha)
        dirty = owned.run(
            ["git", "status", "--porcelain", "--untracked-files=no"], cwd=ROOT,
            timeout=30, capture_output=True, text=True, check=True,
        ).stdout.strip()
        if dirty:
            raise RuntimeError("benchmark preflight requires committed tracked source")
        command = [
            "ssh", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes",
            "-o", "ConnectTimeout=15", "-o", "ServerAliveInterval=15",
            "-o", "ServerAliveCountMax=2", SSH_TARGET,
            remote_command(os.environ.get("SC_BENCH_PYTHON")),
        ]
        with tempfile.TemporaryFile() as request:
            request.write(json.dumps({
                "sha": sha, "helper": OWNED_HELPER.read_text(),
                "bridge": Path(__file__).read_text(),
                "python": os.environ.get("SC_BENCH_PYTHON"),
            }).encode())
            request.seek(0)
            result = owned.run(command, cwd=ROOT, stdin=request, timeout=SSH_TIMEOUT,
                               capture_output=True, text=True)
        print(result.stdout, end="", flush=True)
        print(result.stderr, end="", file=sys.stderr, flush=True)
        try:
            statuses = save_reports(result.stdout, sha, ROOT / "target/benchmark-preflight" / sha)
            if result.returncode == 0 and (not statuses or not statuses <= {"PASS", "MEASURED"}):
                raise ValueError("controlled-host success requires fresh PASS or MEASURED campaign evidence")
        except ValueError as error:
            if result.returncode == 0:
                raise
            # An invalid measurement cannot replace the actual remote failure.
            print(f"benchmark-ssh: evidence validation failed: {error}", file=sys.stderr)
        if result.returncode != 0:
            print(
                f"benchmark-ssh: {SSH_TARGET} returned {result.returncode}; "
                "controlled-host access and prerequisites are required; "
                "SSH configuration and known-host verification were preserved",
                file=sys.stderr,
            )
        return result.returncode if result.returncode >= 0 else 128 - result.returncode
    except subprocess.TimeoutExpired as error:
        if error.output:
            output = error.output.decode(errors="replace") if isinstance(error.output, bytes) else error.output
            print(output, end="", flush=True)
        print("benchmark-ssh: controlled-host connection or gate timed out", file=sys.stderr)
        return 124
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"benchmark-ssh: unable to run controlled-host gate: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    install_signal_handlers()
    sys.exit(main())
