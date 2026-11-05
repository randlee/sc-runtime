"""The public custom-provider context works without a framework worker."""
import os
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tests/integration/generation"))
from owned_process import run


class ApplicationControlContract(unittest.TestCase):
    def test_ordinary_feature_consumer(self):
        # This separate workspace never enables sc-cron's test-utils dev edge.
        manifest = ROOT / "tests/contracts/phase-c/application-control/Cargo.toml"
        result = run(
            ["cargo", "run", "--locked", "--manifest-path", str(manifest)],
            cwd=ROOT,
            env=dict(os.environ, CARGO_BUILD_JOBS="2"),
            timeout=300,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("ordinary application-owned context contract passed", result.stdout)
