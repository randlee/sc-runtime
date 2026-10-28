#!/usr/bin/env python3
"""Validate benchmark campaign JSON example blocks."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_EXAMPLE_KEYS = ("hook_source", "resume_loop", "cli_argv")


class ReportError(Exception):
    """Campaign JSON is missing required example blocks."""


def require_examples(payload: dict[str, Any], *, label: str) -> None:
    examples = payload.get("examples")
    if not isinstance(examples, dict):
        raise ReportError(f"{label}: missing examples object")
    for key in REQUIRED_EXAMPLE_KEYS:
        value = examples.get(key)
        if not isinstance(value, str) or not value.strip():
            raise ReportError(f"{label}: examples.{key} must be a non-empty string")


def main(argv: list[str] | None = None) -> int:
    args = argv if argv is not None else sys.argv[1:]
    if not args:
        print("usage: render_report.py <campaign.json> [...]", file=sys.stderr)
        return 2
    for name in args:
        path = Path(name)
        payload = json.loads(path.read_text(encoding="utf-8"))
        require_examples(payload, label=str(path))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, json.JSONDecodeError, ReportError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
