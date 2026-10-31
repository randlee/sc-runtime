"""Test-only just wrapper recording generated source around driver recipes."""

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path


def snapshot(root):
    result = {}
    for directory, dirs, files in os.walk(root):
        relative = Path(directory).relative_to(root)
        dirs[:] = [
            name for name in dirs
            if name not in {"target", ".git", "__pycache__"}
            and not (relative == Path(".sc-lint") and name == "venv")
        ]
        for name in files:
            path = Path(directory) / name
            if path.is_file():
                result[str(path.relative_to(root))] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def main():
    before = snapshot(Path.cwd())
    result = subprocess.run([os.environ["SC_TEST_REAL_JUST"], *sys.argv[1:]], timeout=900)
    after = snapshot(Path.cwd())
    with open(os.environ["SC_TEST_SNAPSHOT_LOG"], "a", encoding="utf-8") as log:
        log.write(json.dumps({"recipe": sys.argv[1:], "before": before, "after": after}) + "\n")
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
