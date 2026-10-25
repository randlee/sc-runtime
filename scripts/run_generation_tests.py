#!/usr/bin/env python3
"""Run generation unit and integration tests."""

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# `just test` discovers these suites. Generation stays byte-identical to a-6;
# wizard tests are the a-8 contract and entry-mode checks.
SUITES = (
    ("unit", "generation"),
    ("integration", "generation"),
    ("unit", "wizard"),
    ("integration", "wizard"),
)


def main() -> None:
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    for kind, name in SUITES:
        folder = ROOT / "tests" / kind / name
        suite.addTests(loader.discover(str(folder), pattern="test_*.py", top_level_dir=str(folder)))
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    raise SystemExit(0 if result.wasSuccessful() else 1)


if __name__ == "__main__":
    main()
