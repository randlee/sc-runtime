#!/usr/bin/env python3
"""Validate and render benchmark campaign example blocks.

``--index-only`` on a subsystem campaign script calls ``require_examples``
and rewrites published HTML from the stored ``examples`` object. A campaign
missing ``examples.hook_source``, ``examples.resume_loop``, or
``examples.cli_argv`` exits non-zero.
"""

from __future__ import annotations

import html
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_EXAMPLE_KEYS = ("hook_source", "resume_loop", "cli_argv")
_EXAMPLE_TITLES = {
    "hook_source": "Hook snippet",
    "resume_loop": "feed_start / resume loop",
    "cli_argv": "sc-compose CLI argv",
}
_PRE_CODE = re.compile(r"<pre><code>.*?</code></pre>", re.DOTALL)


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


def example_blocks_html(examples: dict[str, Any]) -> str:
    """Render the three required example blocks as ``<pre><code>``."""
    require_examples({"examples": examples}, label="examples")
    parts: list[str] = []
    for key in REQUIRED_EXAMPLE_KEYS:
        body = html.escape(str(examples[key]))
        title = html.escape(_EXAMPLE_TITLES[key])
        parts.append(f"<h3>{title}</h3>\n<pre><code>{body}</code></pre>")
    return "\n".join(parts)


def apply_examples_to_html(page: str, examples: dict[str, Any]) -> str:
    """Put campaign JSON examples into the published HTML.

    The first three ``<pre><code>`` blocks are replaced in
    ``hook_source``, ``resume_loop``, ``cli_argv`` order. A page that does
    not already have those blocks gains a code-examples section.
    """
    require_examples({"examples": examples}, label="campaign")
    blocks = [
        f"<pre><code>{html.escape(str(examples[key]))}</code></pre>"
        for key in REQUIRED_EXAMPLE_KEYS
    ]
    matches = list(_PRE_CODE.finditer(page))
    if len(matches) >= len(blocks):
        pieces: list[str] = []
        cursor = 0
        for match, block in zip(matches[: len(blocks)], blocks):
            pieces.append(page[cursor : match.start()])
            pieces.append(block)
            cursor = match.end()
        pieces.append(page[cursor:])
        return "".join(pieces)
    section = "<section>\n<h2>Code examples</h2>\n" + example_blocks_html(examples) + "\n</section>\n"
    if "</body>" in page:
        return page.replace("</body>", section + "</body>", 1)
    return page + section


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
