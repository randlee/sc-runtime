"""Driver failures match the generation error inventory."""

import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DRIVER = ROOT / "scripts" / "new_project.py"
DASH = "—"

INVENTORY = {
    "GEN.SCHEMA_INVALID": (
        "answers fail wizard/answers.schema.json",
        "fix the file; cargo is not called; the destination is not created",
    ),
    "GEN.DB_RESERVED": (
        "db is postgres or both",
        "set db to sqlite; the destination is not created",
    ),
    "APP.INVALID_NAME": (
        "name fails the project-name pattern in wizard/answers.schema.json",
        "fix the name to that pattern; the destination is not created",
    ),
    "GEN.TOOL_MISSING": (
        "SC_COMPOSE unset and sc-compose not on PATH",
        "set SC_COMPOSE or install sc-compose on PATH; cargo generate does not run",
    ),
    "GEN.DEST_CONFLICT": (
        "the destination path exists",
        "choose an absent path",
    ),
}


def write_script(directory: Path, name: str, body: str) -> None:
    path = directory / name
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


class DriverTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.log = self.root / "log"
        self.answers = self.root / "answers.json"
        self.dest = self.root / "app"
        self.base = {
            "project_name": "widget-app",
            "db": "sqlite",
            "mcp": True,
            "licence": "MIT",
        }

    def save(self, answers):
        self.answers.write_text(json.dumps(answers), encoding="utf-8")

    def stub(self, name: str, body: str):
        write_script(self.bin, name, body)

    def install_success_stubs(self):
        log = self.log
        self.stub(
            "cargo",
            f"""#!/bin/sh
echo "cargo $*" >> "{log}"
dest=""
prev=""
for arg in "$@"; do
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
        self.stub(
            "sc-compose",
            f"""#!/bin/sh
echo "sc-compose $*" >> "{log}"
prev=""
for arg in "$@"; do
  if [ "$prev" = "--output" ]; then
    printf 'rendered\\n' > "$arg"
  fi
  prev="$arg"
done
exit 0
""",
        )
        self.stub(
            "just",
            f"""#!/bin/sh
echo "just $1" >> "{log}"
exit 0
""",
        )
        self.stub(
            "wyvern",
            f"""#!/bin/sh
echo "wyvern $*" >> "{log}"
exit 0
""",
        )

    def invoke(self, extra_env=None, answers=None, path=None, dest_name="app"):
        if answers is None:
            answers = self.base
        self.dest = self.root / dest_name
        self.save(answers)
        env = os.environ.copy()
        original_path = env.get("PATH", "")
        if path is None:
            env["PATH"] = str(self.bin) + os.pathsep + original_path
        else:
            env["PATH"] = path
        env.pop("SC_COMPOSE", None)
        env.pop("SC_RUNTIME_RELEASE", None)
        env["STUB_LOG"] = str(self.log)
        if extra_env:
            env.update(extra_env)
        return subprocess.run(
            [sys.executable, str(DRIVER), str(self.dest), "--var-file", str(self.answers)],
            cwd=ROOT,
            env=env,
            stdin=subprocess.DEVNULL,
            text=True,
            capture_output=True,
            check=False,
        )

    def assert_line(self, completed, code, cause, recovery):
        self.assertNotEqual(completed.returncode, 0, completed.stderr)
        lines = [line for line in completed.stderr.splitlines() if line]
        self.assertEqual(lines, [f"{code} {cause} {DASH} {recovery}"])
        self.assertNotIn("\n", completed.stderr.rstrip("\n") + "x")

    def assert_absent_cargo(self, completed):
        text = self.log.read_text(encoding="utf-8") if self.log.exists() else ""
        self.assertNotIn("generate", text, completed.stderr)
        self.assertFalse(self.dest.exists())

    def test_schema_invalid_does_not_call_cargo(self):
        self.install_success_stubs()
        for name in ("missing-key.json", "wrong-type.json", "db-outside-enum.json", "unknown-key.json"):
            with self.subTest(name=name):
                if self.log.exists():
                    self.log.unlink()
                if self.dest.exists():
                    self.dest.rmdir()
                answers = json.loads((ROOT / "tests" / "fixtures" / "refused" / name).read_text())
                completed = self.invoke(answers=answers)
                self.assert_line(completed, "GEN.SCHEMA_INVALID", *INVENTORY["GEN.SCHEMA_INVALID"])
                self.assert_absent_cargo(completed)

    def test_reserved_database(self):
        self.install_success_stubs()
        for name in ("postgres.json", "both.json"):
            with self.subTest(name=name):
                if self.log.exists():
                    self.log.unlink()
                answers = json.loads((ROOT / "tests" / "fixtures" / "refused" / name).read_text())
                completed = self.invoke(answers=answers)
                self.assert_line(completed, "GEN.DB_RESERVED", *INVENTORY["GEN.DB_RESERVED"])
                self.assert_absent_cargo(completed)

    def test_invalid_names_come_from_the_vector_file(self):
        self.install_success_stubs()
        vectors = json.loads((ROOT / "tests" / "fixtures" / "app-name-vectors.json").read_text())
        self.assertEqual(vectors["code"], "APP.INVALID_NAME")
        for name in vectors["reject"]:
            with self.subTest(name=name):
                if self.log.exists():
                    self.log.unlink()
                answers = dict(self.base)
                answers["project_name"] = name
                completed = self.invoke(answers=answers)
                self.assert_line(completed, "APP.INVALID_NAME", *INVENTORY["APP.INVALID_NAME"])
                self.assert_absent_cargo(completed)

    def test_tool_missing_names_sc_compose(self):
        self.stub("cargo", f'#!/bin/sh\necho "cargo $*" >> "{self.log}"\nexit 0\n')
        self.stub("just", "#!/bin/sh\nexit 0\n")
        completed = self.invoke(path=str(self.bin))
        self.assert_line(completed, "GEN.TOOL_MISSING", *INVENTORY["GEN.TOOL_MISSING"])
        stderr = completed.stderr
        self.assertIn("sc-compose", stderr)
        self.assertIn("SC_COMPOSE", stderr)
        self.assertIn("PATH", stderr)
        self.assert_absent_cargo(completed)

    def test_destination_conflict(self):
        self.install_success_stubs()
        self.dest.mkdir()
        completed = self.invoke()
        self.assert_line(completed, "GEN.DEST_CONFLICT", *INVENTORY["GEN.DEST_CONFLICT"])
        text = self.log.read_text(encoding="utf-8") if self.log.exists() else ""
        self.assertNotIn("generate", text)

    def test_prefers_sc_compose_env_and_ignores_wyvern(self):
        self.install_success_stubs()
        preferred = self.root / "preferred"
        write_script(
            self.root,
            "preferred",
            f"""#!/bin/sh
echo "preferred $*" >> "{self.log}"
prev=""
for arg in "$@"; do
  if [ "$prev" = "--output" ]; then
    printf 'rendered\\n' > "$arg"
  fi
  prev="$arg"
done
exit 0
""",
        )
        self.stub("sc-compose", f'#!/bin/sh\necho "path-compose $*" >> "{self.log}"\nexit 1\n')
        completed = self.invoke(extra_env={"SC_COMPOSE": str(preferred)})
        self.assertEqual(completed.returncode, 0, completed.stderr)
        text = self.log.read_text(encoding="utf-8")
        self.assertIn("preferred", text)
        self.assertNotIn("path-compose", text)
        self.assertNotIn("wyvern", text)
        self.assertIn("--init", text)
        self.assertIn("--silent", text)
        self.assertIn("--template-values-file", text)
        self.assertIn("--force", text)
        self.assertIn("--no-workspace", text)

    def test_release_mode_skips_the_patch(self):
        self.install_success_stubs()
        completed = self.invoke(extra_env={"SC_RUNTIME_RELEASE": "1"})
        self.assertEqual(completed.returncode, 0, completed.stderr)
        text = (self.dest / "Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn("[patch.crates-io]", text)

    def test_dev_mode_writes_the_checkout_patch(self):
        self.install_success_stubs()
        completed = self.invoke()
        self.assertEqual(completed.returncode, 0, completed.stderr)
        text = (self.dest / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn("[patch.crates-io]", text)
        for name in ("sc-config", "sc-transport", "sc-command", "sc-runtime"):
            self.assertIn(str((ROOT / "crates" / name).resolve().as_posix()), text)

    def test_values_failure_stops_before_cargo(self):
        self.install_success_stubs()
        blocked = self.root / "not-a-directory"
        blocked.write_text("file", encoding="utf-8")
        completed = self.invoke(extra_env={"SC_RUNTIME_VALUES_DIR": str(blocked)})
        self.assert_line(
            completed,
            "GEN.STEP_FAILED",
            "step is values",
            "stderr names that step; fix it; later steps do not run",
        )
        self.assert_absent_cargo(completed)

    def test_each_pipeline_step_failure_stops(self):
        cases = [
            ("cargo-generate", "cargo", 1, []),
            ("sc-compose", "sc-compose", 1, ["cargo-generate"]),
            ("setup", "just", 1, ["cargo-generate", "sc-compose"]),
            ("lint", "just", 1, ["setup"]),
            ("test", "just", 1, ["setup", "lint"]),
            ("build-cli", "just", 1, ["setup", "lint", "test"]),
            ("build-daemon", "just", 1, ["setup", "lint", "test", "build-cli"]),
        ]
        for step, tool, _code, earlier in cases:
            with self.subTest(step=step):
                if self.log.exists():
                    self.log.unlink()
                self.install_failing(step)
                completed = self.invoke(dest_name=step)
                self.assert_line(
                    completed,
                    "GEN.STEP_FAILED",
                    f"step is {step}",
                    "stderr names that step; fix it; later steps do not run",
                )
                text = self.log.read_text(encoding="utf-8") if self.log.exists() else ""
                for name in earlier:
                    if name == "cargo-generate":
                        self.assertIn("generate", text)
                    elif name == "sc-compose":
                        self.assertIn("sc-compose", text)
                    else:
                        self.assertIn(f"just {name}", text)
                later = {
                    "cargo-generate": ["sc-compose", "setup"],
                    "sc-compose": ["setup", "lint"],
                    "setup": ["lint"],
                    "lint": ["test"],
                    "test": ["build-cli"],
                    "build-cli": ["build-daemon"],
                    "build-daemon": [],
                }[step]
                for name in later:
                    self.assertNotIn(f"just {name}", text)
                if step == "cargo-generate":
                    self.assertNotIn("sc-compose", text)

    def install_failing(self, step: str):
        log = self.log
        cargo_status = 1 if step == "cargo-generate" else 0
        compose_status = 1 if step == "sc-compose" else 0
        self.stub(
            "cargo",
            f"""#!/bin/sh
echo "cargo $*" >> "{log}"
dest=""
prev=""
for arg in "$@"; do
  if [ "$prev" = "--destination" ]; then dest="$arg"; fi
  prev="$arg"
done
if [ -n "$dest" ]; then
  mkdir -p "$dest"
  printf 'name = "app"\\n' > "$dest/Cargo.toml"
  printf 'j2\\n' > "$dest/AGENTS.md.j2"
  printf 'j2\\n' > "$dest/CLAUDE.md.j2"
fi
exit {cargo_status}
""",
        )
        self.stub(
            "sc-compose",
            f"""#!/bin/sh
echo "sc-compose $*" >> "{log}"
prev=""
for arg in "$@"; do
  if [ "$prev" = "--output" ]; then printf 'rendered\\n' > "$arg"; fi
  prev="$arg"
done
exit {compose_status}
""",
        )
        fail_recipe = step if step in {"setup", "lint", "test", "build-cli", "build-daemon"} else ""
        self.stub(
            "just",
            f"""#!/bin/sh
echo "just $1" >> "{log}"
if [ "$1" = "{fail_recipe}" ]; then
  exit 1
fi
exit 0
""",
        )


if __name__ == "__main__":
    unittest.main()
