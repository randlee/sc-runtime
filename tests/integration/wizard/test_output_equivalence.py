"""Wizard answers and --var-file generate the same project tree.

The Wyvern binary is a stub that returns one fixture. Both runs use the real
driver and real cargo-generate / sc-compose / just when those tools are on
PATH; only Wyvern is stubbed, per REQ-RUN-0603. When sc-compose is absent,
minimal pipeline stubs stand in so the test can still exercise the driver
without hanging on network installs. The comparison skips build and tool output
that embeds a destination path or a timestamp: ``target/``, the
``[patch.crates-io]`` checkout patch, ``.sc-lint/venv/``, and
``.sc-compose/logs/``.
"""

import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DRIVER = ROOT / "scripts" / "new_project.py"
FIXTURE = ROOT / "wizard" / "fixtures" / "sqlite-mcp.json"
GENERATE_TIMEOUT_SECONDS = 1200


def write_script(directory: Path, name: str, body: str) -> None:
    path = directory / name
    path.write_text(body, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IEXEC)


def stub_pipeline_tools(root: Path) -> Path:
    """Minimal cargo/sc-compose/just stubs when the real tools are not on PATH."""
    bin_dir = root / "tool-bin"
    bin_dir.mkdir(exist_ok=True)
    log = root / "pipeline.log"
    write_script(
        bin_dir,
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
  if [ "$prev" = "--destination" ]; then dest="$arg"; fi
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
        bin_dir,
        "sc-compose",
        f"""#!/bin/sh
echo "sc-compose $*" >> "{log}"
prev=""
for arg in "$@"; do
  if [ "$prev" = "--output" ]; then printf 'rendered\\n' > "$arg"; fi
  prev="$arg"
done
exit 0
""",
    )
    write_script(
        bin_dir,
        "just",
        f"""#!/bin/sh
echo "just $1" >> "{log}"
exit 0
""",
    )
    return bin_dir


def without_checkout_patch(data: bytes) -> bytes:
    text = data.decode("utf-8")
    marker = "[patch.crates-io]"
    index = text.find(marker)
    if index == -1:
        return data
    return (text[:index].rstrip() + "\n").encode("utf-8")


def _skip_dir(relative_dir: str, name: str) -> bool:
    """Skip build output, the sc-lint install, and generator logs."""
    child = f"{relative_dir}/{name}".removeprefix("/")
    return name == "target" or child in {".sc-lint/venv", ".sc-compose/logs"}


def snapshot(root: Path) -> dict[str, bytes]:
    """Project files, skipping build output, the tool install, and tool logs."""
    files: dict[str, bytes] = {}
    for dirpath, dirnames, filenames in os.walk(root):
        relative_dir = Path(dirpath).relative_to(root).as_posix()
        if relative_dir == ".":
            relative_dir = ""
        dirnames[:] = [name for name in dirnames if not _skip_dir(relative_dir, name)]
        for name in filenames:
            path = Path(dirpath) / name
            data = path.read_bytes()
            if name == "Cargo.toml":
                data = without_checkout_patch(data)
            files[path.relative_to(root).as_posix()] = data
    return files


class OutputEquivalenceTests(unittest.TestCase):
    def test_stubbed_wizard_matches_var_file_tree(self):
        answers = json.loads(FIXTURE.read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            stub = root / "wyvern"
            payload = json.dumps({"button": "finish", "data": answers, "stack": []})
            write_script(root, "wyvern", f"#!/bin/sh\ncat << 'EOF'\n{payload}\nEOF\n")
            wizard_dest = root / "from-wizard"
            file_dest = root / "from-file"
            if shutil.which("sc-compose") is None:
                if os.name != "posix":
                    self.skipTest("pipeline stubs require a POSIX shell; install sc-compose")
                pipeline_stub = stub_pipeline_tools(root)
            else:
                pipeline_stub = None
            wizard = self.generate(
                [str(wizard_dest)], stub, root, pipeline_stub=pipeline_stub
            )
            self.assertEqual(wizard.returncode, 0, wizard.stderr)
            direct = self.generate(
                [str(file_dest), "--var-file", str(FIXTURE)],
                stub,
                root,
                pipeline_stub=pipeline_stub,
            )
            self.assertEqual(direct.returncode, 0, direct.stderr)
            self.assertNotIn("wyvern", direct.stderr)
            from_wizard = snapshot(wizard_dest)
            from_file = snapshot(file_dest)
            self.assertEqual(sorted(from_wizard), sorted(from_file))
            changed = [rel for rel, data in from_wizard.items() if data != from_file[rel]]
            self.assertEqual(changed, [])

    def generate(
        self,
        args,
        wyvern: Path,
        tmp_root: Path,
        *,
        pipeline_stub: Path | None,
    ):
        if pipeline_stub is not None:
            env = {
                "PATH": os.pathsep.join([str(pipeline_stub), os.defpath]),
                "SC_COMPOSE": str(pipeline_stub / "sc-compose"),
                "WYVERN_BIN": str(wyvern),
                "LANG": "C.UTF-8",
            }
        else:
            env = os.environ.copy()
            env["WYVERN_BIN"] = str(wyvern)
            env.pop("SC_RUNTIME_RELEASE", None)
        try:
            return subprocess.run(
                [sys.executable, str(DRIVER), *args],
                cwd=ROOT,
                env=env,
                stdin=subprocess.DEVNULL,
                text=True,
                capture_output=True,
                check=False,
                timeout=GENERATE_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as exc:
            self.fail(
                f"driver timed out after {GENERATE_TIMEOUT_SECONDS}s\n"
                f"stdout={exc.stdout}\nstderr={exc.stderr}"
            )


if __name__ == "__main__":
    unittest.main()
