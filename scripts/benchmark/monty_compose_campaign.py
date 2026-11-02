#!/usr/bin/env python3
"""Backward-compatible entry point for the Monty compose benchmark campaign."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

_CAMPAIGN = Path(__file__).resolve().parent / "monty" / "campaign.py"


def main() -> int:
    return subprocess.call([sys.executable, str(_CAMPAIGN), *sys.argv[1:]])


if __name__ == "__main__":
    raise SystemExit(main())
