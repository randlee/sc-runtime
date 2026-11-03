#!/usr/bin/env python3
"""Run generation unit and integration tests."""

import os
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# `just test` discovers these suites. Generation stays byte-identical to a-6;
# wizard tests are the a-8 contract and entry-mode checks. The CI scope
# classifier is not part of that frozen generation suite.
SUITES = (
    ("unit", "generation"),
    ("integration", "generation"),
    ("unit", "wizard"),
    ("integration", "wizard"),
    ("unit", "ci"),
)


def main() -> None:
    # CI reuses this target across sequential temporary apps. Keep root Cargo
    # artifacts separate; local runs retain Cargo's normal target selection.
    if target := os.environ.get("SC_RUNTIME_GENERATED_TARGET_DIR"):
        os.environ["CARGO_TARGET_DIR"] = str(Path(target).resolve())
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    for kind, name in SUITES:
        folder = ROOT / "tests" / kind / name
        suite.addTests(loader.discover(str(folder), pattern="test_*.py", top_level_dir=str(folder)))
    # Include the existing bounded root-level release source-contract module.
    suite.addTests(loader.discover(
        str(ROOT / "tests" / "unit"),
        pattern="test_observability_release.py",
        top_level_dir=str(ROOT / "tests" / "unit"),
    ))
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    raise SystemExit(0 if result.wasSuccessful() else 1)


if __name__ == "__main__":
    main()
