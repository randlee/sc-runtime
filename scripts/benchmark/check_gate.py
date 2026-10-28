#!/usr/bin/env python3
"""Run --gate for every benchmark subsystem under scripts/benchmark/."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BENCH = Path(__file__).resolve().parent


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
