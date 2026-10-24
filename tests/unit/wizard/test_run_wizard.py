"""Runner behaviour copied from the reference wizard: prefill, finish, cancel."""

import importlib.util
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[3]


def load_runner():
    spec = importlib.util.spec_from_file_location(
        "run_wizard_under_test",
        ROOT / "scripts" / "run_wizard.py",
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_script(directory: Path, name: str, body: str) -> Path:
    path = directory / name
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)
    return path


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.runner = load_runner()
        self.schema = json.loads(
            (ROOT / "wizard" / "answers.schema.json").read_text(encoding="utf-8")
        )
        self.fixture = json.loads(
            (ROOT / "wizard" / "fixtures" / "sqlite-mcp.json").read_text(encoding="utf-8")
        )

    def test_prefill_drops_unknown_keys_and_leaves_the_descriptor_file(self):
        original = (ROOT / "wizard" / "wizard.json").read_text(encoding="utf-8")
        prefill = self.root / "prefill.json"
        prefill.write_text(
            json.dumps({"mcp": False, "not_a_field": 1}),
            encoding="utf-8",
        )
        descriptor, cleaned = self.runner.build_merged_descriptor(
            ROOT / "wizard" / "wizard.json",
            prefill,
            self.schema,
        )
        self.assertEqual(cleaned, {"mcp": False})
        self.assertEqual(descriptor["config"]["prefill"], {"mcp": False})
        self.assertEqual((ROOT / "wizard" / "wizard.json").read_text(encoding="utf-8"), original)

    def test_finish_cancel_and_dismiss_write_only_a_schema_valid_result(self):
        cases = {
            "finish": (self.finish_body(self.fixture), 0, True),
            "cancel": ('{"button":"cancel","data":{},"stack":[]}\n', 1, False),
            "dismissed": ('{"button":"dismissed","data":{},"stack":[]}\n', 1, False),
        }
        for name, (body, status, writes) in cases.items():
            with self.subTest(name=name):
                self.assert_run(body, status, writes)

    def test_validator_accepts_fixtures_and_rejects_unknown_keys(self):
        for path in (ROOT / "wizard" / "fixtures").glob("*.json"):
            answers = json.loads(path.read_text(encoding="utf-8"))
            self.runner.validate_answers(answers, self.schema)
        bad = dict(self.fixture)
        bad["not_a_field"] = True
        with self.assertRaises(self.runner.WizardError) as unknown:
            self.runner.validate_answers(bad, self.schema)
        self.assertEqual(unknown.exception.code, "GEN.SCHEMA_INVALID")
        named = dict(self.fixture)
        named["project_name"] = "has space"
        with self.assertRaises(self.runner.WizardError) as invalid_name:
            self.runner.validate_answers(named, self.schema)
        self.assertEqual(invalid_name.exception.code, "APP.INVALID_NAME")

    def test_missing_jsonschema_recovery_matches_inventory(self):
        real_import = __import__

        def fake_import(name, *args, **kwargs):
            if name == "jsonschema":
                raise ImportError("no jsonschema")
            return real_import(name, *args, **kwargs)

        with mock.patch("builtins.__import__", side_effect=fake_import):
            with self.assertRaises(self.runner.WizardError) as raised:
                self.runner.validate_answers(self.fixture, self.schema)
        self.assertEqual(raised.exception.code, "GEN.TOOL_MISSING")
        self.assertEqual(
            raised.exception.recovery,
            "pip install jsonschema==4.26.0; cargo generate does not run",
        )

    def test_missing_binary_names_wyvern_override_and_path(self):
        saved_path = os.environ.get("PATH")
        saved_bin = os.environ.get("WYVERN_BIN")
        empty = self.root / "empty-path"
        empty.mkdir()
        os.environ["PATH"] = str(empty)
        os.environ.pop("WYVERN_BIN", None)
        try:
            with self.assertRaises(self.runner.WizardError) as raised:
                self.runner.find_wyvern_bin()
        finally:
            if saved_path is None:
                os.environ.pop("PATH", None)
            else:
                os.environ["PATH"] = saved_path
            if saved_bin is None:
                os.environ.pop("WYVERN_BIN", None)
            else:
                os.environ["WYVERN_BIN"] = saved_bin
        message = str(raised.exception)
        self.assertIn("wyvern", message)
        self.assertIn("WYVERN_BIN", message)
        self.assertIn("PATH", message)

    def finish_body(self, answers: dict) -> str:
        return json.dumps({"button": "finish", "data": answers, "stack": []}) + "\n"

    def assert_run(self, stdout_body: str, status: int, writes: bool):
        log = self.root / "calls"
        copy = self.root / "descriptor.json"
        stub = write_script(
            self.root,
            "wyvern",
            f"""#!/bin/sh
echo "$*" >> "{log}"
cp "$1" "{copy}"
cat << 'EOF'
{stdout_body}EOF
""",
        )
        output = self.root / "out.json"
        output.unlink(missing_ok=True)
        env = os.environ.copy()
        env["WYVERN_BIN"] = str(stub)
        completed = subprocess.run(
            [
                sys.executable,
                str(ROOT / "scripts" / "run_wizard.py"),
                "--output",
                str(output),
            ],
            cwd=ROOT,
            env=env,
            capture_output=True,
            text=True,
            check=False,
            timeout=10,
        )
        self.assertEqual(completed.returncode, status, completed.stderr)
        self.assertIn("--ui-root", log.read_text(encoding="utf-8"))
        self.assertIn(str(ROOT / "wizard"), log.read_text(encoding="utf-8"))
        if writes:
            self.assertEqual(json.loads(output.read_text(encoding="utf-8")), self.fixture)
            self.runner.validate_answers(json.loads(output.read_text(encoding="utf-8")), self.schema)
        else:
            self.assertFalse(output.exists())
            lines = [line for line in completed.stderr.splitlines() if line.strip()]
            self.assertTrue(lines[-1].startswith("GEN.WIZARD_CANCELLED "), completed.stderr)
            self.assertNotIn("ERROR:", completed.stderr)


if __name__ == "__main__":
    unittest.main()
