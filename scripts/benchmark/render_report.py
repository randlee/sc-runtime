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
    "resume_loop": "production drive_feed resume loop",
    "cli_argv": "sc-compose CLI argv",
}
_PRE_CODE = re.compile(r"<pre><code>.*?</code></pre>", re.DOTALL)
_H3_AND_PRE = re.compile(r"<h3>.*?</h3>\s*<pre><code>.*?</code></pre>", re.DOTALL)


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


def _example_pre(examples: dict[str, Any], key: str) -> str:
    return f"<pre><code>{html.escape(str(examples[key]))}</code></pre>"


def _example_titled(examples: dict[str, Any], key: str) -> str:
    title = html.escape(_EXAMPLE_TITLES[key])
    return f"<h3>{title}</h3>\n{_example_pre(examples, key)}"


def _splice(page: str, matches: list[re.Match[str]], blocks: list[str]) -> str:
    pieces: list[str] = []
    cursor = 0
    for match, block in zip(matches, blocks, strict=True):
        pieces.append(page[cursor : match.start()])
        pieces.append(block)
        cursor = match.end()
    pieces.append(page[cursor:])
    return "".join(pieces)


def example_blocks_html(examples: dict[str, Any]) -> str:
    """Render the three required example blocks as ``<pre><code>``."""
    require_examples({"examples": examples}, label="examples")
    return "\n".join(_example_titled(examples, key) for key in REQUIRED_EXAMPLE_KEYS)


def apply_examples_to_html(page: str, examples: dict[str, Any]) -> str:
    """Put campaign JSON examples into the published HTML.

    The first three example blocks are replaced in ``hook_source``,
    ``resume_loop``, ``cli_argv`` order. A heading immediately before a
    code block is retitled with that block (the resume loop is the
    production ``execute_feed_with_compose_program`` excerpt). A page that does not already
    have those blocks gains a code-examples section.
    """
    require_examples({"examples": examples}, label="campaign")
    titled = [_example_titled(examples, key) for key in REQUIRED_EXAMPLE_KEYS]
    headed = list(_H3_AND_PRE.finditer(page))
    if len(headed) >= len(titled):
        return _splice(page, headed[: len(titled)], titled)
    blocks = [_example_pre(examples, key) for key in REQUIRED_EXAMPLE_KEYS]
    matches = list(_PRE_CODE.finditer(page))
    if len(matches) >= len(blocks):
        return _splice(page, matches[: len(blocks)], blocks)
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




def validate_measurement_html(page: str, campaign: dict[str, Any]) -> None:
    """Check the existing harness result table against its canonical JSON twin."""
    import math
    from html.parser import HTMLParser

    class Table(HTMLParser):
        def __init__(self) -> None:
            super().__init__(convert_charrefs=True)
            self.stack: list[str] = []
            self.rows: dict[str, list[str]] = {}
            self.row: str | None = None
            self.cell: list[str] | None = None
            self.seen: set[str] = set()

        def handle_starttag(self, tag, attrs):
            self.seen.add(tag)
            if tag not in {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}:
                self.stack.append(tag)
            if tag == "tr" and dict(attrs).get("id") is not None:
                self.row = dict(attrs)["id"]
                if self.row in self.rows:
                    raise ReportError("duplicate HTML measurement row")
                self.rows[self.row] = []
            if tag == "td" and self.row is not None:
                self.cell = []

        def handle_startendtag(self, tag, attrs):
            self.handle_starttag(tag, attrs)
            if tag not in {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"}:
                self.handle_endtag(tag)

        def handle_data(self, data):
            if self.cell is not None:
                self.cell.append(data)

        def handle_endtag(self, tag):
            if not self.stack or self.stack.pop() != tag:
                raise ReportError("malformed report HTML structure")
            if tag == "td" and self.cell is not None:
                self.rows[self.row].append(" ".join(" ".join(self.cell).split()))
                self.cell = None
            if tag == "tr":
                self.row = None

    table = Table()
    table.feed(page)
    table.close()
    if table.stack or not {"html", "body", "table", "tbody"}.issubset(table.seen):
        raise ReportError("report HTML must contain a complete measurement document")
    expected = {case["id"]: case for case in campaign["cases"]}
    if set(table.rows) != set(expected):
        raise ReportError("HTML measurement cases do not match campaign JSON")
    for case_id, case in expected.items():
        cells = table.rows[case_id]
        if len(cells) != 5:
            raise ReportError(f"HTML {case_id}: expected five measurement cells")
        label = " ".join((case["label"] + " " + case["strategy"]).split())
        if cells[0] != label or cells[1] != f"{case['ms_per_run']:.3f} ms":
            raise ReportError(f"HTML {case_id}: label/strategy/mean contradict campaign JSON")
        duration = re.fullmatch(r"([0-9]+(?:\.[0-9]+)?)(ns|µs|μs|ms|s) total", cells[2])
        factors = {"ns": 1e-6, "µs": 1e-3, "μs": 1e-3, "ms": 1, "s": 1000}
        if duration is None or not math.isclose(float(duration[1]) * factors[duration[2]], case["total_ms"], rel_tol=1e-9, abs_tol=1e-6):
            raise ReportError(f"HTML {case_id}: total contradicts campaign JSON")
        if cells[3] != f"{case['timed_iters']} + 1 warmup":
            raise ReportError(f"HTML {case_id}: iterations contradict campaign JSON")
        rss = []
        sub = case.get("subprocess_rss_bytes")
        parent = case.get("in_process_rss_bytes")
        if sub is not None:
            rss.extend((["subprocess"] if parent is not None else []) + [f"{sub / 1048576:.2f} MiB", case["subprocess_rss_note"]])
        if parent is not None:
            rss.extend((["parent"] if sub is not None else []) + [f"{parent / 1048576:.2f} MiB" + (" (in-process)" if sub is None else ""), case["in_process_rss_note"]])
        memory = " ".join(" ".join(rss).split()) if rss else "Not measured"
        if cells[4] != memory:
            raise ReportError(f"HTML {case_id}: memory contradicts campaign JSON")


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, json.JSONDecodeError, ReportError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
