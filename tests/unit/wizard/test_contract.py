"""Wizard pages match the answers schema and the a-6 boundary holds."""

import importlib.util
import json
import os
import re
import subprocess
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
WIZARD = ROOT / "wizard"
PAGES = WIZARD / "pages"
A6 = "a321ea2cde4d44d8723c44f4215922465a9dfa93"
ENTRY_MODE_FUNCTIONS = {"parse_args", "main"}
NEW_ENTRY_FUNCTION = "entry_answers"
SUBPROCESS_TIMEOUT_SECONDS = 30


def _git_run(args: list[str]) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["GIT_TERMINAL_PROMPT"] = "0"
    env["GIT_SSH_COMMAND"] = "ssh -o BatchMode=yes"
    try:
        return subprocess.run(
            args,
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            timeout=SUBPROCESS_TIMEOUT_SECONDS,
            stdin=subprocess.DEVNULL,
            env=env,
        )
    except subprocess.TimeoutExpired as exc:
        raise AssertionError(
            f"git timed out after {SUBPROCESS_TIMEOUT_SECONDS}s: {' '.join(args)}\n"
            f"stdout={exc.stdout}\nstderr={exc.stderr}"
        ) from exc


def a6_git_ref() -> str:
    """Resolve sprint a-6 baseline; fetch it when CI uses a shallow clone."""
    for ref in (A6, "origin/sprint/a-6-template-from-json"):
        ok = _git_run(["git", "rev-parse", "--verify", f"{ref}^{{commit}}"])
        if ok.returncode == 0:
            return ref
    _git_run(["git", "fetch", "origin", A6, "--depth=1"])
    ok = _git_run(["git", "rev-parse", "--verify", f"{A6}^{{commit}}"])
    if ok.returncode != 0:
        raise AssertionError(
            f"sprint a-6 baseline {A6} is not available in this clone "
            f"(rev-parse after fetch failed; stderr={ok.stderr!r})"
        )
    return A6


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

    def run_bounded(self, args, *, check_output=False):
        try:
            if check_output:
                return subprocess.check_output(
                    args,
                    cwd=ROOT,
                    text=True,
                    timeout=SUBPROCESS_TIMEOUT_SECONDS,
                )
            return subprocess.run(
                args,
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
                timeout=SUBPROCESS_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as exc:
            self.fail(
                f"{args[0]} timed out after {SUBPROCESS_TIMEOUT_SECONDS}s\n"
                f"stdout={exc.stdout}\nstderr={exc.stderr}"
            )

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
        completed = self.run_bounded(
            ["grep", "-ri", "nuget", str(WIZARD / "wizard.json"), str(PAGES)]
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
            "answers.schema.json",
        }
        hits = [path for path in (ROOT / "template").rglob("*") if path.name in banned]
        self.assertEqual(hits, [])

    def test_cargo_generate_db_choices_match_the_schema(self):
        schema = json.loads((WIZARD / "answers.schema.json").read_text(encoding="utf-8"))
        cargo = tomllib.loads((ROOT / "template" / "cargo-generate.toml").read_text(encoding="utf-8"))
        choices = ["none", "sqlite", "postgres", "mysql", "sqlite-postgres", "sqlite-mysql"]
        self.assertEqual(cargo["placeholders"]["db"]["choices"], choices)
        self.assertEqual(schema["properties"]["db"]["enum"], choices)
        self.assertEqual(cargo["placeholders"]["db"]["default"], schema["properties"]["db"]["default"])
        self.assertNotIn("x-v01-reserved", json.dumps(schema))
        self.assertEqual(schema["x-schema-version"], 2)

    def test_just_new_forwards_arguments(self):
        text = (ROOT / "Justfile").read_text(encoding="utf-8")
        self.assertIn("scripts/new_project.py {{dest}} {{args}}", text)

    def test_just_test_discovers_wizard_suites(self):
        spec = importlib.util.spec_from_file_location(
            "run_generation_tests_under_test",
            ROOT / "scripts" / "run_generation_tests.py",
        )
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertIn(("unit", "wizard"), module.SUITES)
        self.assertIn(("integration", "wizard"), module.SUITES)
        self.assertIn(("unit", "generation"), module.SUITES)
        self.assertIn(("integration", "generation"), module.SUITES)

    def test_driver_drops_the_reserved_database_failure(self):
        source = (ROOT / "scripts" / "new_project.py").read_text(encoding="utf-8")
        self.assertNotIn("GEN.DB_RESERVED", source)
        self.assertNotIn("reserved_database", source)
        self.assertIn("def entry_answers", source)
        self.assertIn("--allow-commands", source)
        postgres = (
            ROOT / "template/crates/store-adaptor-sqlite-postgres/src/lib.rs"
        ).read_text(encoding="utf-8")
        mysql = (
            ROOT / "template/crates/store-adaptor-sqlite-mysql/src/lib.rs"
        ).read_text(encoding="utf-8")
        shared = 'include!("../../store-adaptor/src/membership.rs");'
        self.assertIn(shared, postgres)
        self.assertIn(shared, mysql)
        self.assertNotIn("fn membership_from_toml", postgres)
        self.assertNotIn("fn membership_from_toml", mysql)


if __name__ == "__main__":
    unittest.main()
