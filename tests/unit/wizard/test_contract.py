"""Wizard pages match the answers schema and the a-6 boundary holds."""

import json
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
WIZARD = ROOT / "wizard"
PAGES = WIZARD / "pages"
A6 = "a321ea2cde4d44d8723c44f4215922465a9dfa93"
ENTRY_MODE_FUNCTIONS = {"parse_args", "main"}
NEW_ENTRY_FUNCTION = "entry_answers"


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def split_functions(source: str) -> tuple[str, dict[str, str]]:
    parts = re.split(r"(?m)^(?=def )", source)
    preamble = parts[0]
    functions = {}
    for part in parts[1:]:
        name = part.split("(", 1)[0].removeprefix("def ")
        functions[name] = part
    return preamble, functions


class WizardContractTests(unittest.TestCase):
    def schema(self):
        return load_json(WIZARD / "answers.schema.json")

    def test_layout_files_exist(self):
        self.assertTrue((WIZARD / "wizard.json").is_file())
        self.assertTrue((PAGES / "wizard-answers.js").is_file())
        self.assertTrue((PAGES / "wizard-page.css").is_file())
        self.assertTrue((ROOT / "scripts" / "run_wizard.py").is_file())
        html = list(PAGES.glob("*.html"))
        self.assertGreaterEqual(len(html), 1)

    def test_referenced_pages_exist_and_carry_no_copied_product_name(self):
        descriptor = load_json(WIZARD / "wizard.json")
        self.assertEqual(descriptor["type"], "wizard")
        referenced = {descriptor["page"]["html"]}
        for page in PAGES.glob("*.html"):
            referenced.update(re.findall(r'html:\s*"([^"]+)"', page.read_text(encoding="utf-8")))
        for relative in referenced:
            self.assertTrue((WIZARD / relative).is_file(), relative)
        completed = subprocess.run(
            ["grep", "-ri", "nuget", str(WIZARD / "wizard.json"), str(PAGES)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(completed.stdout, "")
        self.assertNotEqual(completed.returncode, 0)

    def test_pages_expose_exactly_the_schema_properties(self):
        schema = self.schema()
        found = []
        for page in PAGES.glob("*.html"):
            found.extend(re.findall(r'data-answer="([^"]+)"', page.read_text(encoding="utf-8")))
        self.assertEqual(set(found), set(schema["properties"]))
        self.assertEqual(len(found), len(set(found)))

    def test_defaults_mirror_the_schema(self):
        schema = self.schema()
        helper = (PAGES / "wizard-answers.js").read_text(encoding="utf-8")
        body = re.search(r"var DEFAULTS = \{([^}]*)\};", helper)
        self.assertIsNotNone(body)
        defaults = {}
        for key, raw in re.findall(r"(\w+)\s*:\s*([^,\n]+)", body.group(1)):
            raw = raw.strip()
            if raw == "true":
                defaults[key] = True
            elif raw == "false":
                defaults[key] = False
            elif len(raw) >= 2 and raw[0] == '"' and raw[-1] == '"':
                defaults[key] = raw[1:-1]
            else:
                self.fail(f"unparsed default {key}: {raw}")
        expected = {
            name: spec["default"]
            for name, spec in schema["properties"].items()
            if isinstance(spec, dict) and "default" in spec
        }
        self.assertEqual(defaults, expected)
        self.assertNotIn("project_name", defaults)

        identity = (PAGES / "page-1-identity.html").read_text(encoding="utf-8")
        options = (PAGES / "page-3-options.html").read_text(encoding="utf-8")
        licence = re.search(
            r'<input\b[^>]*data-answer="licence"[^>]*>',
            identity,
        )
        self.assertIsNotNone(licence)
        self.assertIn(f'value="{schema["properties"]["licence"]["default"]}"', licence.group(0))
        project = re.search(
            r'<input\b[^>]*data-answer="project_name"[^>]*>',
            identity,
        )
        self.assertIsNotNone(project)
        self.assertNotIn("value=", project.group(0))

        option_tags = re.findall(r'<option value="([^"]+)"([^>]*)>', options)
        self.assertEqual(
            [value for value, _attrs in option_tags],
            schema["properties"]["db"]["enum"],
        )
        selected = [value for value, attrs in option_tags if "selected" in attrs]
        self.assertEqual(selected, [schema["properties"]["db"]["default"]])

        mcp = re.search(r'<input\b[^>]*data-answer="mcp"[^>]*>', options)
        self.assertIsNotNone(mcp)
        if schema["properties"]["mcp"]["default"] is True:
            self.assertIn("checked", mcp.group(0))
        else:
            self.assertNotIn("checked", mcp.group(0))

    def test_pages_do_not_copy_the_project_name_pattern(self):
        pattern = self.schema()["properties"]["project_name"]["pattern"]
        for path in [WIZARD / "wizard.json", *PAGES.iterdir(), ROOT / "scripts" / "run_wizard.py"]:
            if path.is_file():
                self.assertNotIn(pattern, path.read_text(encoding="utf-8"), path.name)

    def test_scripts_have_no_install_path_and_driver_has_no_wizard_binary_name(self):
        driver = (ROOT / "scripts" / "new_project.py").read_text(encoding="utf-8")
        runner = (ROOT / "scripts" / "run_wizard.py").read_text(encoding="utf-8")
        self.assertNotIn("wyvern", driver.lower())
        for text in (driver, runner):
            self.assertIsNone(
                re.search(r'["\'](?:/[^"\']+|\\\\[^"\']+)wyvern[^"\']*["\']', text, re.I)
            )
        self.assertIn("WYVERN_BIN", runner)
        self.assertIn("PATH", runner)

    def test_template_does_not_carry_wizard_assets(self):
        banned = {
            "wizard.json",
            "wizard-answers.js",
            "wizard-page.css",
            "run_wizard.py",
            "new_project.py",
        }
        hits = [path for path in (ROOT / "template").rglob("*") if path.name in banned]
        self.assertEqual(hits, [])

    def test_just_new_forwards_arguments(self):
        text = (ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn("scripts/new_project.py {{dest}} {{args}}", text)

    def test_generation_suite_is_byte_identical_to_a6(self):
        completed = subprocess.run(
            [
                "git",
                "diff",
                "--name-only",
                A6,
                "--",
                "tests/unit/generation",
                "tests/integration/generation",
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(completed.stdout, "")

    def test_driver_diff_is_entry_mode_only(self):
        old = subprocess.check_output(
            ["git", "show", f"{A6}:scripts/new_project.py"],
            cwd=ROOT,
            text=True,
        )
        new = (ROOT / "scripts" / "new_project.py").read_text(encoding="utf-8")
        old_preamble, old_functions = split_functions(old)
        new_preamble, new_functions = split_functions(new)
        self.assertEqual(new_preamble, old_preamble)
        self.assertEqual(set(new_functions) - set(old_functions), {NEW_ENTRY_FUNCTION})
        for name, body in old_functions.items():
            if name in ENTRY_MODE_FUNCTIONS:
                continue
            self.assertEqual(new_functions[name], body, name)
        self.assertIn("--prefill", new_functions["parse_args"])
        self.assertIn("--var-file", new_functions["parse_args"])
        self.assertIn("entry_answers", new_functions["main"])


if __name__ == "__main__":
    unittest.main()
