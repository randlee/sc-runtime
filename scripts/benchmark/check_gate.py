#!/usr/bin/env python3
"""Run --gate for every benchmark subsystem under scripts/benchmark/.

The gate refuses a host label outside the allowlist unless
``SC_BENCH_GATE_ALLOW_ANY_HOST=1``. The allowlist always includes
``rand-m5`` and any labels recorded in that subsystem's ``baselines.json``
(``allowed_hosts``). The bypass is for local development only.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BENCH = Path(__file__).resolve().parent
DEFAULT_GATE_HOSTS = frozenset({"rand-m5"})
ALLOW_ANY_HOST_ENV = "SC_BENCH_GATE_ALLOW_ANY_HOST"
HOST_SAFE = re.compile(r"[^A-Za-z0-9._-]+")


class GateHostError(Exception):
    """The gate host label is not on the allowlist."""


def safe_host(label: str) -> str:
    """Sanitize a host label the same way campaign ids do."""
    cleaned = HOST_SAFE.sub("-", label.strip()).strip("-.")
    if not cleaned:
        raise GateHostError(f"host label is empty after sanitizing: {label!r}")
    return cleaned[:64]


def host_label() -> str:
    """Return ``SC_BENCH_HOST_LABEL`` or the short hostname."""
    raw = os.environ.get("SC_BENCH_HOST_LABEL")
    if raw:
        return safe_host(raw)
    node = os.uname().nodename.split(".")[0]
    return safe_host(node)


def allow_any_host() -> bool:
    """True only for the local-dev bypass. Publish preflight must not set this."""
    return os.environ.get(ALLOW_ANY_HOST_ENV) == "1"


def allowed_gate_hosts(baselines_path: Path) -> set[str]:
    """``rand-m5`` plus ``allowed_hosts`` from ``baselines.json`` when that file exists."""
    hosts = set(DEFAULT_GATE_HOSTS)
    if not baselines_path.is_file():
        return hosts
    try:
        payload = json.loads(baselines_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise GateHostError(f"{baselines_path}: invalid JSON") from exc
    if not isinstance(payload, dict):
        raise GateHostError(f"{baselines_path}: JSON root must be an object")
    recorded = payload.get("allowed_hosts", [])
    if recorded is None:
        recorded = []
    if not isinstance(recorded, list):
        raise GateHostError(f"{baselines_path}: allowed_hosts must be a list of host labels")
    for item in recorded:
        if not isinstance(item, str) or not item.strip():
            raise GateHostError(f"{baselines_path}: allowed_hosts entries must be non-empty strings")
        hosts.add(safe_host(item))
    return hosts


def assert_gate_host(baselines_path: Path, label: str | None = None) -> str:
    """Reject a host that is not on the allowlist.

    ``SC_BENCH_GATE_ALLOW_ANY_HOST=1`` skips the check. Any other value,
    including an unset variable, keeps the allowlist in force.
    """
    resolved = host_label() if label is None else safe_host(label)
    if allow_any_host():
        return resolved
    allowed = allowed_gate_hosts(baselines_path)
    if resolved not in allowed:
        allowed_list = ", ".join(sorted(allowed))
        raise GateHostError(
            f"gate host {resolved!r} is not in the allowlist ({allowed_list}). "
            "Authoritative timings come from atmbench@rand-m5. "
            "Set SC_BENCH_GATE_ALLOW_ANY_HOST=1 only for local development."
        )
    return resolved


def discover_subsystems() -> list[str]:
    names: list[str] = []
    for entry in sorted(BENCH.iterdir()):
        if not entry.is_dir():
            continue
        if (entry / "campaign.py").is_file():
            names.append(entry.name)
    return names


def main() -> int:
    subsystems = discover_subsystems()
    if not subsystems:
        print("error: no scripts/benchmark/<subsystem>/campaign.py found", file=sys.stderr)
        return 1
    for name in subsystems:
        baselines = BENCH / name / "baselines.json"
        try:
            assert_gate_host(baselines)
        except GateHostError as exc:
            print(f"error: {name}: {exc}", file=sys.stderr)
            return 1
    for name in subsystems:
        script = BENCH / name / "campaign.py"
        print(f"benchmark-gate: {name}", flush=True)
        completed = subprocess.run(
            [sys.executable, str(script), "--gate"],
            cwd=ROOT,
            check=False,
        )
        if completed.returncode != 0:
            return completed.returncode
    print("benchmark-gate: all subsystems passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
