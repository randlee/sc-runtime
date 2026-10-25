"""Schema validity and fixture acceptance."""

import importlib.util
import json
import jsonschema
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def driver():
    spec = importlib.util.spec_from_file_location("new_project", ROOT / "scripts" / "new_project.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SchemaTests(unittest.TestCase):
    def test_schema_matches_its_meta_schema(self):
        schema = load_json(ROOT / "wizard" / "answers.schema.json")
        self.assertEqual(schema["$schema"], "https://json-schema.org/draft/2020-12/schema")
        self.assertEqual(schema["x-schema-version"], 1)
        jsonschema.Draft202012Validator.check_schema(schema)

    def test_driver_has_no_option_list(self):
        source = (ROOT / "scripts" / "new_project.py").read_text(encoding="utf-8")
        self.assertNotIn('["sqlite"', source)
        self.assertNotIn('["postgres"', source)
        self.assertNotIn('["db", "mcp"', source)
        self.assertNotIn("wyvern", source.lower())
        self.assertNotIn("/sc-compose", source)
        self.assertNotIn("sys.stdin", source)


if __name__ == "__main__":
    unittest.main()
