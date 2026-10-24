"""Driver entry modes: interactive, prefill, cancel, and --var-file."""

import json
import os
import stat
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DRIVER = ROOT / "scripts" / "new_project.py"
DASH = "—"


def write_script(directory: Path, name: str, body: str) -> None:
    path = directory / name
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class EntryModeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.log = self.root / "log"
        self.values = self.root / "values.toml"
        self.compose = self.root / "compose.json"
        self.name = self.root / "name.txt"
        self.descriptor = self.root / "descriptor.json"
        self.fixture_path = ROOT / "wizard" / "fixtures" / "sqlite-mcp.json"
        self.fixture = json.loads(self.fixture_path.read_text(encoding="utf-8"))

    def stub_tools(self, wizard_stdout: str | None):
        log = self.log
        values = self.values
        compose = self.compose
        name = self.name
        descriptor = self.descriptor
        write_script(
            self.bin,
            "cargo",
            f"""#!/bin/sh
echo "cargo $*" >> "{log}"
if [ "$1" = "generate" ] && [ "$2" = "--version" ]; then
  echo "cargo-generate 0.25.1"
  exit 0
fi
dest=""
prev=""
for arg in "$@"; do
  if [ "$prev" = "--template-values-file" ]; then
    cp "$arg" "{values}"
  fi
  if [ "$prev" = "--name" ]; then
    printf '%s\\n' "$arg" > "{name}"
  fi
  if [ "$prev" = "--destination" ]; then
    dest="$arg"
  fi
  prev="$arg"
done
if [ -n "$dest" ]; then
  mkdir -p "$dest"
  printf 'name = "app"\\n' > "$dest/Cargo.toml"
  printf 'j2\\n' > "$dest/AGENTS.md.j2"
  printf 'j2\\n' > "$dest/CLAUDE.md.j2"
fi
exit 0
""",
        )
        write_script(
            self.bin,
            "sc-compose",
            f"""#!/bin/sh
echo "sc-compose $*" >> "{log}"
prev=""
for arg in "$@"; do
  if [ "$prev" = "--var-file" ]; then
    cp "$arg" "{compose}"
  fi
  if [ "$prev" = "--output" ]; then
    printf 'rendered\\n' > "$arg"
  fi
  prev="$arg"
done
exit 0
""",
        )
        write_script(
            self.bin,
            "just",
            f"""#!/bin/sh
echo "just $1" >> "{log}"
exit 0
""",
        )
        if wizard_stdout is None:
            return
        write_script(
            self.bin,
            "wyvern",
            f"""#!/bin/sh
echo "wyvern $*" >> "{log}"
cp "$1" "{descriptor}"
cat << 'EOF'
{wizard_stdout}EOF
""",
        )

    def invoke(self, args, extra_env=None, include_wyvern=True):
        env = {
            key: os.environ[key]
            for key in (
                "HOME",
                "USER",
                "LANG",
                "LC_ALL",
                "LC_CTYPE",
                "SYSTEMROOT",
                "SystemRoot",
                "COMSPEC",
                "USERPROFILE",
                "WINDIR",
                "TMP",
                "TEMP",
            )
            if key in os.environ
        }
        env["PATH"] = f"{self.bin}:/bin:/usr/bin"
        env["TMPDIR"] = str(self.root)
        if extra_env:
            env.update(extra_env)
        if not include_wyvern:
            env.pop("WYVERN_BIN", None)
        try:
            return subprocess.run(
                [sys.executable, str(DRIVER), *args],
                cwd=ROOT,
                env=env,
                stdin=subprocess.DEVNULL,
                text=True,
                capture_output=True,
                check=False,
                timeout=15,
            )
        except subprocess.TimeoutExpired as exc:
            self.fail(f"driver timed out\nstdout={exc.stdout}\nstderr={exc.stderr}")

    def log_text(self) -> str:
        if not self.log.exists():
            return ""
        return self.log.read_text(encoding="utf-8")

    def clear_captures(self):
        for path in (self.log, self.values, self.compose, self.name, self.descriptor):
            if path.exists():
                path.unlink()

    def captured_answers(self) -> dict:
        values = tomllib.loads(self.values.read_text(encoding="utf-8"))["values"]
        values["project_name"] = self.name.read_text(encoding="utf-8").strip()
        return values

    def finish(self, answers: dict) -> str:
        return json.dumps({"button": "finish", "data": answers, "stack": []}) + "\n"

    def test_interactive_finish_uses_the_wizard_result(self):
        self.stub_tools(self.finish(self.fixture))
        dest = self.root / "interactive"
        completed = self.invoke([str(dest)])
        self.assertEqual(completed.returncode, 0, completed.stderr)
        text = self.log_text()
        self.assertEqual(len([line for line in text.splitlines() if line.startswith("wyvern ")]), 1)
        self.assertEqual(self.captured_answers(), self.fixture)
        self.assertEqual(json.loads(self.compose.read_text(encoding="utf-8")), self.fixture)
        self.assertIn("--silent", text)
        self.assertIn("--template-values-file", text)
        self.assertTrue(dest.is_dir())

    def test_prefill_is_review_material_and_the_wizard_result_is_used(self):
        self.stub_tools(self.finish(self.fixture))
        prefill = self.root / "prefill.json"
        prefill.write_text(json.dumps({"mcp": False, "not_a_field": 1}), encoding="utf-8")
        dest = self.root / "prefill-app"
        completed = self.invoke([str(dest), "--prefill", str(prefill)])
        self.assertEqual(completed.returncode, 0, completed.stderr)
        descriptor = json.loads(self.descriptor.read_text(encoding="utf-8"))
        self.assertEqual(descriptor["config"]["prefill"], {"mcp": False})
        self.assertNotIn("not_a_field", json.dumps(descriptor))
        self.assertIn("not_a_field", completed.stderr)
        captured = self.captured_answers()
        self.assertEqual(captured, self.fixture)
        self.assertIs(captured["mcp"], True)
        on_disk = json.loads((ROOT / "wizard" / "wizard.json").read_text(encoding="utf-8"))
        self.assertNotIn("prefill", on_disk.get("config", {}))

    def test_each_fixture_matches_var_file_generation_inputs(self):
        for path in sorted((ROOT / "wizard" / "fixtures").glob("*.json")):
            with self.subTest(path=path.name):
                answers = json.loads(path.read_text(encoding="utf-8"))
                self.clear_captures()
                self.stub_tools(self.finish(answers))
                wizard_dest = self.root / f"wizard-{path.stem}"
                wizard = self.invoke([str(wizard_dest)])
                self.assertEqual(wizard.returncode, 0, wizard.stderr)
                from_wizard = self.captured_answers()
                wizard_compose = json.loads(self.compose.read_text(encoding="utf-8"))
                self.clear_captures()
                file_dest = self.root / f"file-{path.stem}"
                direct = self.invoke([str(file_dest), "--var-file", str(path)])
                self.assertEqual(direct.returncode, 0, direct.stderr)
                self.assertNotIn("wyvern ", self.log_text())
                self.assertEqual(from_wizard, self.captured_answers())
                self.assertEqual(wizard_compose, json.loads(self.compose.read_text(encoding="utf-8")))
                self.assertEqual(from_wizard, answers)

    def test_cancel_does_not_call_cargo_or_create_a_destination(self):
        self.stub_tools('{"button":"cancel","data":{},"stack":[]}\n')
        dest = self.root / "cancelled"
        completed = self.invoke([str(dest)])
        self.assertNotEqual(completed.returncode, 0)
        lines = [line for line in completed.stderr.splitlines() if line.strip()]
        self.assertTrue(lines[-1].startswith("GEN.WIZARD_CANCELLED "), completed.stderr)
        self.assertNotIn("ERROR:", completed.stderr)
        self.assertNotIn("cargo", self.log_text())
        self.assertFalse(dest.exists())

    def test_missing_wyvern_names_the_binary_and_creates_nothing(self):
        self.stub_tools(None)
        dest = self.root / "missing"
        completed = self.invoke([str(dest)], include_wyvern=False)
        self.assertNotEqual(completed.returncode, 0)
        lines = [line for line in completed.stderr.splitlines() if line.strip()]
        self.assertTrue(lines[-1].startswith("GEN.TOOL_MISSING "), completed.stderr)
        self.assertIn("wyvern", completed.stderr)
        self.assertIn("WYVERN_BIN", completed.stderr)
        self.assertIn("PATH", completed.stderr)
        self.assertNotIn("ERROR:", completed.stderr)
        self.assertNotIn("cargo", self.log_text())
        self.assertFalse(dest.exists())
        self.assertNotIn("Traceback", completed.stderr)

    def test_var_file_ignores_prefill_and_a_wyvern_override(self):
        marker = self.root / "called"
        write_script(
            self.root,
            "wyvern-override",
            f"""#!/bin/sh
echo called > "{marker}"
echo '{{"button":"finish","data":{{}},"stack":[]}}'
""",
        )
        self.stub_tools(self.finish(self.fixture))
        dest = self.root / "var-file"
        completed = self.invoke(
            [str(dest), "--var-file", str(self.fixture_path), "--prefill", str(self.fixture_path)],
            extra_env={"WYVERN_BIN": str(self.root / "wyvern-override")},
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertFalse(marker.exists())
        self.assertNotIn("wyvern ", self.log_text())
        self.assertEqual(self.captured_answers(), self.fixture)

    def test_runner_rejects_use_inventory_codes(self):
        unknown = dict(self.fixture)
        unknown["not_a_field"] = True
        invalid = dict(self.fixture)
        invalid["project_name"] = "has space"
        cases = (
            ("unknown", unknown, "GEN.SCHEMA_INVALID"),
            ("name", invalid, "APP.INVALID_NAME"),
        )
        for label, answers, code in cases:
            with self.subTest(label=label):
                self.clear_captures()
                self.stub_tools(self.finish(answers))
                dest = self.root / label
                completed = self.invoke([str(dest)])
                self.assertNotEqual(completed.returncode, 0)
                lines = [line for line in completed.stderr.splitlines() if line.strip()]
                self.assertTrue(lines[-1].startswith(f"{code} "), completed.stderr)
                self.assertNotIn("ERROR:", completed.stderr)
                self.assertNotIn("cargo", self.log_text())
                self.assertFalse(dest.exists())

    def test_reserved_database_from_the_wizard_stays_on_the_driver(self):
        answers = dict(self.fixture)
        answers["db"] = "postgres"
        self.stub_tools(self.finish(answers))
        dest = self.root / "reserved"
        completed = self.invoke([str(dest)])
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn(
            f"GEN.DB_RESERVED db is postgres or both {DASH} set db to sqlite; the destination is not created",
            completed.stderr,
        )
        lines = [line for line in completed.stderr.splitlines() if line.strip()]
        self.assertTrue(lines[-1].startswith("GEN.DB_RESERVED "), completed.stderr)
        self.assertNotIn("ERROR:", completed.stderr)
        self.assertNotIn("cargo", self.log_text())
        self.assertFalse(dest.exists())


if __name__ == "__main__":
    unittest.main()
