"""Valid fixtures pass the schema. Refused files fail it."""

import importlib.util
import json
import jsonschema
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def driver():
    spec = importlib.util.spec_from_file_location("new_project", ROOT / "scripts" / "new_project.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FixtureTests(unittest.TestCase):
    def setUp(self):
        self.driver = driver()
        self.schema = json.loads(
            (ROOT / "wizard" / "answers.schema.json").read_text(encoding="utf-8")
        )

    def test_every_wizard_fixture_is_accepted(self):
        paths = sorted((ROOT / "wizard" / "fixtures").glob("*.json"))
        self.assertGreaterEqual(len(paths), 3)
        for path in paths:
            answers = json.loads(path.read_text(encoding="utf-8"))
            self.assertIn("licence", answers)
            self.assertTrue(self.driver.schema_accepts(self.schema, answers), path.name)
            self.assertFalse(self.driver.reserved_database(self.schema, answers), path.name)

    def test_sqlite_mcp_and_licence_variants(self):
        mcp = json.loads((ROOT / "wizard" / "fixtures" / "sqlite-mcp.json").read_text())
        off = json.loads((ROOT / "wizard" / "fixtures" / "sqlite-no-mcp.json").read_text())
        apache = json.loads((ROOT / "wizard" / "fixtures" / "sqlite-apache.json").read_text())
        self.assertEqual(mcp["db"], "sqlite")
        self.assertIs(mcp["mcp"], True)
        self.assertEqual(off["db"], "sqlite")
        self.assertIs(off["mcp"], False)
        self.assertNotEqual(apache["licence"], "MIT")

    def test_schema_invalid_files_are_rejected(self):
        refused = ROOT / "tests" / "fixtures" / "refused"
        for name in ("missing-key.json", "wrong-type.json", "db-outside-enum.json", "unknown-key.json"):
            answers = json.loads((refused / name).read_text(encoding="utf-8"))
            self.assertFalse(self.driver.schema_accepts(self.schema, answers), name)

    def test_driver_matches_jsonschema_on_every_fixture(self):
        validator = jsonschema.Draft202012Validator(self.schema)
        paths = sorted((ROOT / "wizard" / "fixtures").glob("*.json"))
        paths += sorted((ROOT / "tests" / "fixtures" / "refused").glob("*.json"))
        self.assertGreaterEqual(len(paths), 7)
        for path in paths:
            answers = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(
                self.driver.schema_accepts(self.schema, answers),
                validator.is_valid(answers),
                path.name,
            )


if __name__ == "__main__":
    unittest.main()
