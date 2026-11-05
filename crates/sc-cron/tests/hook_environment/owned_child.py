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
    # JSON strings cannot carry a lone Windows surrogate through Rust serde.
    # Transport every OS string as UTF-16 units and reconstruct without loss.
    def decode(units):
        return b"".join(unit.to_bytes(2, "little") for unit in units).decode("utf-16-le", "surrogatepass")

    args = [decode(value) for value in request["args_utf16"]]
    cwd = decode(request["cwd_utf16"]) if request["cwd_utf16"] is not None else None
    environment = dict(os.environ)
    for raw_key, raw_value in request["env_utf16"]:
        key = decode(raw_key)
        value = decode(raw_value) if raw_value is not None else None
        if value is None:
            environment.pop(key, None)
        else:
            environment[key] = value
    try:
        result = run(args, cwd=cwd, env=environment,
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
