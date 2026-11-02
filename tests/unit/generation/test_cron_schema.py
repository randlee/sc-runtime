"""Cron config schema stays out of generated projects and matches CronConfig."""

import json
import jsonschema
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "template" / "config" / "cron.schema.json"
DEFAULT_PATH = ROOT / "template" / "config" / "default.json"
CRON_RS = ROOT / "template" / "crates" / "daemon" / "src" / "cron.rs"


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def render_default(db: str) -> dict:
    """Render template/config/default.json for one db value.

    The file's only Liquid is a chain of `{% if %}` / `{% elsif %}` on `db`.
    """
    include = True
    chosen = []
    for line in DEFAULT_PATH.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("{% if ") or stripped.startswith("{% elsif "):
            include = f'db == "{db}"' in stripped
            continue
        if stripped.startswith("{% else"):
            include = False
            continue
        if stripped.startswith("{% endif"):
            include = True
            continue
        if include:
            chosen.append(line)
    return json.loads("\n".join(chosen))


class CronSchemaTests(unittest.TestCase):
    def schema(self):
        schema = load_json(SCHEMA_PATH)
        jsonschema.Draft202012Validator.check_schema(schema)
        return schema

    def test_schema_is_not_the_wizard_contract_and_is_ignored(self):
        self.assertFalse((ROOT / "template" / "answers.schema.json").exists())
        hits = [path for path in (ROOT / "template").rglob("*") if path.name == "answers.schema.json"]
        self.assertEqual(hits, [])
        cargo = tomllib.loads((ROOT / "template" / "cargo-generate.toml").read_text(encoding="utf-8"))
        self.assertEqual(cargo["template"]["exclude"], ["AGENTS.md.j2", "CLAUDE.md.j2"])
        self.assertEqual(cargo["template"]["ignore"], ["config/cron.schema.json"])
        schema = self.schema()
        self.assertEqual(set(schema["properties"]), {"cron"})
        field = schema["properties"]["cron"]["properties"]["max_schedule_rows"]
        self.assertEqual(field["type"], ["integer", "null"])
        self.assertEqual(field["minimum"], 0)
        self.assertEqual(field["default"], 10)
        self.assertNotIn("cargo-generate excludes this file", schema["description"])

    def test_schema_matches_default_json_and_cron_config(self):
        schema = self.schema()
        validator = jsonschema.Draft202012Validator(schema)
        for db in ("none", "sqlite", "postgres", "mysql", "sqlite-postgres", "sqlite-mysql"):
            document = render_default(db)
            validator.validate(document)
            self.assertEqual(document["cron"]["max_schedule_rows"], 10)

        validator.validate({"cron": {"max_schedule_rows": None}})
        validator.validate({"cron": {"max_schedule_rows": 0}})
        validator.validate({})
        errors = list(validator.iter_errors({"cron": {"max_schedule_rows": -1}}))
        self.assertTrue(errors)

        source = CRON_RS.read_text(encoding="utf-8")
        self.assertIn("pub max_schedule_rows: Option<u64>", source)
        self.assertIn("fn default_max_schedule_rows() -> Option<u64> {\n    Some(10)\n}", source)
        self.assertIn("fn json_null_is_unlimited", source)
        self.assertIn("NonZeroUsize::new(width)", source)
        self.assertIn('r#"{"max_schedule_rows": null}"#', source)
        self.assertIn('r#"{"max_schedule_rows":0}"#', source)

    def test_generated_project_omits_the_cron_schema(self):
        if shutil.which("cargo") is None:
            self.skipTest("cargo is not installed")
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            destination = root / "dest"
            destination.mkdir()
            values = root / "values.toml"
            values.write_text(
                '[values]\ndb = "sqlite"\nmcp = true\nlicence = "MIT"\n',
                encoding="utf-8",
            )
            result = subprocess.run(
                [
                    "cargo",
                    "generate",
                    "--path",
                    str(ROOT / "template"),
                    "--name",
                    "widget-app",
                    "--destination",
                    str(destination),
                    "--silent",
                    "--allow-commands",
                    "--template-values-file",
                    str(values),
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr or result.stdout)
            project = destination / "widget-app"
            names = {path.name for path in project.rglob("*") if path.is_file()}
            self.assertNotIn("answers.schema.json", names)
            self.assertNotIn("cron.schema.json", names)
            document = load_json(project / "config" / "default.json")
            jsonschema.Draft202012Validator(self.schema()).validate(document)
            self.assertEqual(document["cron"]["max_schedule_rows"], 10)


if __name__ == "__main__":
    unittest.main()
