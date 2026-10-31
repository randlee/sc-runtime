"""Prepublication contracts for the observability release and publishing kit."""

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path, PureWindowsPath

ROOT = Path(__file__).resolve().parents[3]


class GeneratedObservabilityIntegration(unittest.TestCase):
    def test_pinned_publish_kit_inventory(self):
        pin = tomllib.loads((ROOT / "release/sc-publish-pin.toml").read_text())["sc-publish"]
        self.assertEqual(pin["repository"], "https://github.com/randlee/sc-publish")
        self.assertEqual(pin["revision"], "9677ab083e7dd3feb258235a342e321ca1b45d8d")
        temp = tempfile.TemporaryDirectory(prefix="sc-publish-inventory-")
        self.addCleanup(temp.cleanup)
        checkout = Path(temp.name)
        git_env = {**os.environ, "GIT_TERMINAL_PROMPT": "0"}
        subprocess.run(["git", "init", "--quiet", str(checkout)], check=True, timeout=30, env=git_env)
        subprocess.run(["git", "-C", str(checkout), "fetch", "--quiet", "--depth", "1", pin["repository"], pin["revision"]], check=True, timeout=120, env=git_env)
        subprocess.run(["git", "-C", str(checkout), "checkout", "--quiet", "--detach", "FETCH_HEAD"], check=True, timeout=30, env=git_env)
        actual = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True, timeout=30, env=git_env).strip()
        self.assertEqual(actual, pin["revision"])
        installer = checkout / "plugins/sc-publish/install.py"
        spec = importlib.util.spec_from_file_location("pinned_publish_installer", installer)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        copies = module.install_inventory(ROOT)
        paths = [relative for _, relative in copies] + list(module.TEMPLATES.values())
        self.assertEqual(len(paths), 85)
        self.assertEqual(len(paths), len(set(paths)))
        for path in paths:
            self.assertFalse(path.is_absolute() or ".." in path.parts or "\\" in str(path))
            self.assertFalse(PureWindowsPath(str(path)).drive)
            self.assertNotEqual(path.parts[:2], ("docs", "sprints"))
            current = ROOT
            for part in path.parts:
                current = current / part
                self.assertFalse(current.is_symlink(), str(path))
            self.assertTrue(current.is_file(), str(path))
        for source, relative in copies:
            self.assertFalse(source.is_symlink(), str(source))
            self.assertEqual(source.read_bytes(), (ROOT / relative).read_bytes(), str(relative))

    def test_release_contract_sources(self):
        manifest = tomllib.loads((ROOT / "template/Cargo.toml").read_text())
        dependencies = manifest["workspace"]["dependencies"]
        for name in ("sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron"):
            self.assertEqual(dependencies[name]["version"], "0.1.1")
            self.assertNotIn("path", dependencies[name])
        for name in ("sc-observability", "sc-observability-types"):
            self.assertEqual(dependencies[name]["version"], "1.5.1")
            self.assertNotIn("path", dependencies[name])
        self.assertNotIn("patch", manifest)
        consumer = json.loads((ROOT / "release/sc-publish-consumer-input.json").read_text())
        crates = sorted((c for c in consumer["crates"] if c["publish"]), key=lambda c: c["publish_order"])
        self.assertEqual([c["package"] for c in crates], ["sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron"])
        self.assertEqual([c["publish_order"] for c in crates], [1, 2, 3, 4, 5])
        checks = {c["id"]: c["run"] for c in consumer["preflight_checks"]}
        self.assertEqual(checks["observability-registry-only"], "python3 scripts/check_observability_release.py")
        self.assertIn("test_release_contract_sources", checks["release-contract-sources"])
        subprocess.run([sys.executable, "-m", "unittest", "tests.integration.generation.test_driver.DriverTests.test_release_mode_skips_the_patch"], cwd=ROOT, check=True)
