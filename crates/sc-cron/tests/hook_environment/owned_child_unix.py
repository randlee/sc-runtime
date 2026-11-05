"""Pinned Unix test launcher; cleanup reuses the canonical session inspector."""

import json
import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[4]))
from tests.integration.generation.owned_process import _check_session_inspection, _launch, _stop_owned_session


def launch(request_path):
    request_path = Path(request_path)
    request = json.loads(request_path.read_text())
    decode = lambda raw: os.fsdecode(bytes(raw))
    if request["cwd"] is not None:
        os.chdir(decode(request["cwd"]))
    for key, value in request["env"]:
        key = decode(key)
        if value is None:
            os.environ.pop(key, None)
        else:
            os.environ[key] = decode(value)
    root = request_path.parent
    try:
        _check_session_inspection()
    except RuntimeError as error:
        print(str(error), file=sys.stderr)
        (root / "status").write_text(json.dumps({"returncode": 127}))
        return 127
    config = root / "launcher.json"
    config.write_text(json.dumps({"args": [decode(raw) for raw in request["args"]],
                                  "start": str(root / "start"), "status": str(root / "status")}))
    # Rust already assigned the session before the launcher's first instruction.
    # _launch keeps that identity alive after command completion until cleanup.
    (root / "ready").touch()
    return _launch(config)


if __name__ == "__main__":
    if sys.argv[1] == "launch":
        sys.exit(launch(sys.argv[2]))
    if sys.argv[1] == "stop":
        _stop_owned_session(int(sys.argv[2]))
        sys.exit(0)
    sys.exit(2)
