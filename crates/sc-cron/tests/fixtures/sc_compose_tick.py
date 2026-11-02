#!/usr/bin/env python3
"""Reference tick hook: render string template via PyO3 sc_compose.

Requires: pip install sc-compose
"""
from __future__ import annotations

import json
import sys

import sc_compose


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(2)
    cmd = sys.argv[1]
    if cmd == "cron.stop":
        print(json.dumps({"action": "continue", "reason": "ok"}))
        return
    if cmd == "cron.tick":
        hook = json.load(sys.stdin)
        payload = hook["payload"]
        sc_compose.render_template(payload["template"], payload["context"])
        return
    sys.exit(2)


if __name__ == "__main__":
    main()
