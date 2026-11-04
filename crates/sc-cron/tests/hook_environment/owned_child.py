"""Windows Rust-test bridge to the existing bounded process-tree owner."""

import json
import os
from pathlib import Path
import subprocess
import sys

# The test executable may run in a selected cwd; imports belong to this checkout.
sys.path.insert(0, str(Path(__file__).resolve().parents[4]))
from tests.integration.generation.owned_process import run


def main(request_path):
    request = json.loads(Path(request_path).read_text())
    environment = dict(os.environ)
    for key, value in request["env"]:
        if value is None:
            environment.pop(key, None)
        else:
            environment[key] = value
    try:
        result = run(request["args"], cwd=request["cwd"], env=environment,
                     timeout=request["timeout"], stdin=subprocess.DEVNULL,
                     capture_output=True)
    except subprocess.TimeoutExpired as error:
        sys.stdout.buffer.write(error.output or b"")
        sys.stderr.buffer.write(error.stderr or b"")
        print("owned test child exceeded its deadline; tree cleanup completed", file=sys.stderr)
        return 124
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.buffer.write(result.stderr)
    return result.returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
