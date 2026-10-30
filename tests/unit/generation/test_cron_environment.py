"""Generated shared hook policy, administrator defaults and checked example."""
import json
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch
import jsonschema

from tests.unit.generation.test_cron_schema import render_default

ROOT = Path(__file__).resolve().parents[3]


class CronEnvironmentTests(unittest.TestCase):
    def test_empty_administrator_defaults_and_absolute_authorization_schema(self):
        schema = json.loads((ROOT / "template/config/cron.schema.json").read_text())
        validator = jsonschema.Draft202012Validator(schema)
        empty = {"allow_process": [], "allow_dotenv_files": [], "allow_dotenv_roots": []}
        for db in ("none", "sqlite", "postgres", "mysql", "sqlite-postgres", "sqlite-mysql"):
            config = render_default(db)
            validator.validate(config)
            self.assertEqual(config["cron"]["environment"], empty)
        for field in ("allow_dotenv_files", "allow_dotenv_roots"):
            validator.validate({"cron": {"environment": {field: ["/var/lib/app/agent.env"]}}})
            validator.validate({"cron": {"environment": {field: [r"C:\app\agent.env"]}}})
            with self.assertRaises(jsonschema.ValidationError):
                validator.validate({"cron": {"environment": {field: ["relative.env"]}}})
        with self.assertRaises(jsonschema.ValidationError):
            validator.validate({"cron": {"environment": {"allow_process": ["bad-name"]}}})

    def test_prebuilt_typed_policy_keeps_resolver_and_separately_wires_files_roots(self):
        source = (ROOT / "template/crates/daemon/src/cron.rs").read_text()
        for token in ("pub fn hook_environment_host", ".allow_process(", ".allow_dotenv_file(", ".allow_dotenv_root(", "environment: sc_cron::HookEnvironmentHost", "scheduler.with_hook_environment(environment)", "prebuilt_policy_retains_application_reference_resolver_through_open", "config.row_limit()"):
            self.assertIn(token, source)
        open_source = source.split("pub fn open(", 1)[1].split("/// Row-cap fields", 1)[0]
        self.assertNotIn("hook_environment_host()", open_source)
        self.assertIn('cron = ["dep:sc-cron"]', (ROOT / "template/crates/daemon/Cargo.toml").read_text())
        self.assertIn('monty = ["cron", "sc-cron/monty"]', (ROOT / "template/crates/daemon/Cargo.toml").read_text())

    def test_checked_example_receives_identity_team_clear_reference_dotenv_without_render_helper(self):
        calls = []

        def cli(command, args):
            calls.append((command, args))
            return {"stdout": "recorded", "stderr": "", "exit_code": 0}

        # This fixture contains no real secret. Runtime integration tests execute
        # the same source through Monty with typed policy and a recording resolver.
        with patch.dict("os.environ", {"ATM_IDENTITY": "planner-7", "ATM_TEAM": "runtime", "OTHER_API_KEY": "recording-reference", "AGENT_MODE": "recording"}, clear=True):
            namespace = runpy.run_path(str(ROOT / "template/examples/monty/cron.py"), init_globals={"sc_compose_run": cli, "atm_run": cli})
            hook = {"protocol": "sc-cron.hook.v1", "event": "cron.tick", "schedule": "example", "payload": {"identity": "planner-7", "team": "runtime", "compose_args": ["--root", "/templates", "--file", "note.j2"]}, "deadline": "2026-11-01T00:00:00Z"}
            self.assertEqual(namespace["INITIAL_IDENTITY"], "planner-7")
            self.assertEqual(namespace["cron_stop"](hook)["action"], "continue")
            self.assertIsNone(namespace["cron_tick"](hook))
        self.assertEqual(calls, [("render", hook["payload"]["compose_args"]), ("agent", ["list"])])
        source = (ROOT / "template/examples/monty/cron.py").read_text()
        self.assertNotIn("render_template", source)
        doc = (ROOT / "docs/sc-cron/monty-generated-example.md").read_text()
        for token in ("RecordingResolver", ".with_environment(", ".set(\"ATM_IDENTITY\"", ".set(\"ATM_TEAM\"", ".clear(\"OPENAI_API_KEY\"", ".reference(\"OTHER_API_KEY\"", ".dotenv(", "cron::open", "[REDACTED_ENV]"):
            self.assertIn(token, doc)


if __name__ == "__main__":
    unittest.main()
