"""Placeholder keys equal schema keys except the project-name key."""

import importlib.util
import json
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def project_name_key() -> str:
    spec = importlib.util.spec_from_file_location("new_project", ROOT / "scripts" / "new_project.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.PROJECT_NAME_KEY


PROJECT_NAME_KEY = project_name_key()


class KeySetTests(unittest.TestCase):
    def test_placeholder_keys_match_schema_except_project_name(self):
        schema = json.loads((ROOT / "wizard" / "answers.schema.json").read_text(encoding="utf-8"))
        cargo = tomllib.loads((ROOT / "template" / "cargo-generate.toml").read_text(encoding="utf-8"))
        schema_keys = set(schema["properties"]) - {PROJECT_NAME_KEY}
        placeholder_keys = set(cargo["placeholders"])
        self.assertEqual(schema_keys, placeholder_keys)
        self.assertIn(PROJECT_NAME_KEY, schema["properties"])
        self.assertNotIn(PROJECT_NAME_KEY, placeholder_keys)


if __name__ == "__main__":
    unittest.main()
