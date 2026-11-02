#!/usr/bin/env python3
"""Run release source contracts with bounded, owned generation prerequisites."""

import importlib.util
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
PYTHON = sys.executable
DEADLINES = (120, 300, 120)


def main() -> int:
    spec = importlib.util.spec_from_file_location(
        "release_contract_owned", ROOT / "tests/integration/generation/owned_process.py")
    owned = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(owned)

    def stop(number, _frame):
        # Unwind owned.run first; it stops descendants before venv cleanup.
        raise SystemExit(128 + number)

    for name in ("SIGHUP", "SIGINT", "SIGTERM"):
        if hasattr(signal, name):
            signal.signal(getattr(signal, name), stop)
    env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
    try:
        with tempfile.TemporaryDirectory(prefix="sc-release-contracts-", dir=os.environ.get("RUNNER_TEMP")) as temporary:
            venv = Path(temporary) / "venv"
            python = venv / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
            commands = (
                [PYTHON, "-m", "venv", str(venv)],
                [str(python), "-m", "pip", "install", "--disable-pip-version-check", "-r", str(ROOT / "requirements-generation.txt")],
                [str(python), "-m", "unittest", "tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_contract_sources"],
            )
            for command, timeout in zip(commands, DEADLINES):
                owned.run(command, cwd=ROOT, env=env, timeout=timeout,
                          check=True, stdin=subprocess.DEVNULL)
    except subprocess.TimeoutExpired as error:
        print(f"release source contracts: command exceeded {error.timeout}s deadline: {error.cmd}", file=sys.stderr)
        return 124
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"release source contracts: command failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
