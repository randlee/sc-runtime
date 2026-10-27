"""Store layouts prove general-db can be turned off."""

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class GeneralDbDisableTests(unittest.TestCase):
    def test_sqlite_mcp_job_compiles_the_flag_off(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text(encoding="utf-8")
        self.assertIn("matrix.fixture == 'sqlite-mcp.json'", workflow)
        self.assertIn(
            "cargo test -p widget-app-cli --no-default-features --lib general_db_off_omits_delete_commands",
            workflow,
        )
        self.assertIn(
            "cargo test -p widget-app-daemon --no-default-features --lib openapi_paths_omit_general_delete",
            workflow,
        )
        self.assertIn(
            "cargo test -p widget-app-cli --no-default-features --test widget_mcp",
            workflow,
        )

    def test_cli_flag_controls_the_daemon_dev_dependency(self):
        manifest = (ROOT / "template/crates/cli/Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('general-db = ["daemon/general-db"]', manifest)
        self.assertIn(
            'daemon = { path = "../daemon", default-features = false }',
            manifest,
        )
        cli = (ROOT / "template/crates/cli/src/lib.rs").read_text(encoding="utf-8")
        self.assertIn("fn general_db_off_omits_delete_commands", cli)
        routes = (ROOT / "template/crates/daemon/src/routes.rs").read_text(encoding="utf-8")
        self.assertIn("fn openapi_paths_omit_general_delete", routes)
        mcp = (ROOT / "template/crates/cli/tests/widget_mcp.rs").read_text(encoding="utf-8")
        self.assertIn("api_types::DELETE_ROW_PATH", mcp)
        self.assertIn('cfg(not(feature = "general-db"))', mcp)


if __name__ == "__main__":
    unittest.main()
