"""Exercise root just selectors used by the template fixture workflow."""

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class WorkflowDispatchTests(unittest.TestCase):
    def test_generated_selectors_preserve_commands_and_working_directory(self):
        just = shutil.which("just")
        self.assertIsNotNone(just)
        if os.name == "nt":
            for selector in ("generated-workspace", "generated-general-db-off"):
                result = subprocess.run([just, "--dry-run", "test", selector], cwd=ROOT,
                                        text=True, capture_output=True, check=True, timeout=10)
                self.assertIn('cd "$SC_RUNTIME_GENERATED_DIR"', result.stdout)
            return
        with tempfile.TemporaryDirectory(prefix="fixture-dispatch-") as temp:
            root = Path(temp)
            app = root / "generated app"
            app.mkdir()
            log = root / "calls"
            cargo = root / "cargo"
            cargo.write_text('#!/bin/sh\nprintf "%s|%s|%s\\n" "$PWD" "$SQLX_OFFLINE" "$*" >> "$SC_TEST_CALLS"\n')
            cargo.chmod(0o755)
            env = {**os.environ, "PATH": str(root) + os.pathsep + os.environ.get("PATH", ""),
                   "SC_RUNTIME_GENERATED_DIR": str(app), "SQLX_OFFLINE": "true", "SC_TEST_CALLS": str(log)}
            for selector in ("generated-workspace", "generated-general-db-off"):
                subprocess.run([just, "test", selector], cwd=ROOT, env=env, check=True, timeout=10)
            calls = log.read_text().splitlines()
            self.assertEqual(calls, [
                f"{app}|true|check --workspace",
                f"{app}|true|test -p widget-app-cli --no-default-features --lib general_db_off_omits_delete_commands",
                f"{app}|true|test -p widget-app-daemon --no-default-features --lib openapi_paths_omit_general_delete",
                f"{app}|true|test -p widget-app-cli --no-default-features --test widget_mcp",
            ])

    def test_workflow_uses_root_just_selectors(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text()
        fixture_job = workflow.split("  generate:\n", 1)[1].split("  plain-cargo-generate:\n", 1)[0]
        self.assertIn("run: just test generated-workspace", fixture_job)
        self.assertIn("run: just test generated-general-db-off", fixture_job)
        self.assertNotIn("cargo check --workspace", fixture_job)
        self.assertNotIn("cargo test -p widget-app", fixture_job)


if __name__ == "__main__":
    unittest.main()
