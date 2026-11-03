"""Store layouts prove general-db can be turned off."""

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


class GeneralDbDisableTests(unittest.TestCase):
    def test_sqlite_mcp_job_compiles_the_flag_off(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text(encoding="utf-8")
        step = (
            workflow.split("      - name: Store layout general-db disable\n", 1)[1].split("\n  plain-cargo-generate:", 1)[0]
        )
        self.assertIn("matrix.fixture == 'sqlite-mcp.json'", step)
        self.assertIn("SQLX_OFFLINE: 'true'", step)
        self.assertIn("SC_RUNTIME_GENERATED_DIR: ${{ runner.temp }}/app", step)
        self.assertIn("run: just test generated-general-db-off", step)
        recipe = (ROOT / "Justfile").read_text(encoding="utf-8")
        commands = (
            recipe.split('layer == "generated-general-db-off"', 1)[1]
            .split('else if layer ==', 1)[0]
        )
        self.assertIn(
            "cargo test -p widget-app-cli --no-default-features --lib general_db_off_omits_delete_commands",
            commands,
        )
        self.assertIn(
            "cargo test -p widget-app-daemon --no-default-features --lib openapi_paths_omit_general_delete",
            commands,
        )
        self.assertIn(
            "cargo test -p widget-app-cli --no-default-features --test widget_mcp",
            commands,
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
