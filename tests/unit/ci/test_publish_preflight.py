"""The consumer manifest retains every publish crate and rejects missing members."""

import json
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from unittest import mock
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class PublishPreflightTests(unittest.TestCase):
    def test_validator_timeout_fails_clearly(self):
        with mock.patch.object(subprocess, "run", side_effect=subprocess.TimeoutExpired("validator", 30)) as run:
            with self.assertRaisesRegex(AssertionError, "validator exceeded its 30s deadline"):
                self.validate(ROOT)
        self.assertEqual(run.call_args.kwargs["timeout"], 30)

    def validate(self, root):
        try:
            return subprocess.run(
                [sys.executable, str(ROOT / ".github/scripts/release_artifacts.py"),
                 "validate-manifest", "--manifest", "release/publish-artifacts.toml",
                 "--workspace-toml", "Cargo.toml"],
                cwd=root, capture_output=True, text=True, check=False, timeout=30,
            )
        except subprocess.TimeoutExpired:
            self.fail("publish manifest validator exceeded its 30s deadline")

    def test_workspace_and_publish_inventory_match(self):
        result = self.validate(ROOT)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]
        actual = {str(path.parent.relative_to(ROOT)) for path in (ROOT / "crates").glob("*/Cargo.toml")}
        self.assertEqual(set(workspace["members"]), actual)
        consumer = json.loads((ROOT / "release/sc-publish-consumer-input.json").read_text())
        self.assertEqual(sum(crate["publish"] for crate in consumer["crates"]), 5)
        self.assertIn("crates/sc-runtime-db-sqlite", workspace["members"])

    def test_absent_publish_member_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cargo = (ROOT / "Cargo.toml").read_text().replace('    "crates/sc-cron",\n', "")
            (root / "Cargo.toml").write_text(cargo)
            for relative in [Path("release/publish-artifacts.toml"),
                             Path("release/publish-channel-contracts.toml"),
                             *(path.relative_to(ROOT) for path in (ROOT / "crates").glob("*/Cargo.toml"))]:
                (root / relative).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / relative, root / relative)
            result = self.validate(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("manifest references non-member crates: crates/sc-cron/Cargo.toml", result.stderr)
