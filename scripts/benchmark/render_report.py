#!/usr/bin/env python3
"""Render complete deterministic campaign HTML and shared example blocks.

Producers write the complete JSON rendering; release verification compares its
exact bytes. Index refreshes preserve immutable historical report twins.
"""

from __future__ import annotations

import html
import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_EXAMPLE_KEYS = ("hook_source", "resume_loop", "cli_argv")
_EXAMPLE_TITLES = {
    "hook_source": "Hook snippet",
    "resume_loop": "production drive_feed resume loop",
    "cli_argv": "sc-compose CLI argv",
}


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


def example_blocks_html(examples: dict[str, Any]) -> str:
    """Render the three required example blocks as ``<pre><code>``."""
    require_examples({"examples": examples}, label="examples")
    return "\n".join(_example_titled(examples, key) for key in REQUIRED_EXAMPLE_KEYS)


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




def render_campaign_html(campaign: dict[str, Any]) -> str:
    """Render the complete published report from its immutable JSON twin.

    Both producer and release verifier use this exact rendering. No raw harness
    HTML, ambient timestamp, file content, or unreviewed prose is interpolated.
    """
    require_examples(campaign, label="campaign")
    def esc(value: Any) -> str:
        return html.escape(str(value))

    def number(value: Any) -> str:
        return "not measured" if value is None else f"{value:.6f}"

    rows = []
    for case in campaign["cases"]:
        memory = []
        for key, title in (("subprocess", "Subprocess"), ("in_process", "In-process")):
            size = case.get(f"{key}_rss_bytes")
            if size is not None:
                memory.append(f"{title}: {size / 1048576:.2f} MiB — {case.get(f'{key}_rss_note')}")
        limit = case.get("max_ms_per_run")
        ceiling = "none (unratified)" if limit is None else f"{number(limit)} ms"
        rows.append(
            f'<tr id="{esc(case["id"])}"><td><strong>{esc(case.get("label", case["id"]))}</strong>'
            f'<p class="strategy">{esc(case.get("strategy", ""))}</p></td>'
            f'<td>{esc(number(case.get("ms_per_run")))} ms</td>'
            f'<td>{esc(number(case.get("total_ms")))} ms total</td>'
            f'<td>{esc(case.get("timed_iters"))} + 1 warmup</td>'
            f'<td>{esc(case.get("status"))}<br>{esc(case.get("skip_reason") or "")}</td>'
            f'<td>{esc(ceiling)}</td><td>{esc("; ".join(memory) or "Not measured")}</td></tr>'
        )
    examples = example_blocks_html(campaign["examples"])
    for key, title in (("compose_run", "sc_compose_run hook"),
                       ("lab_resume_loop", "Lab feed_start / resume loop")):
        if key in campaign["examples"]:
            examples += f'\n<h3>{title}</h3>\n<pre><code>{esc(campaign["examples"][key])}</code></pre>'
    metadata = "\n".join(
        f'<dt>{esc(key)}</dt><dd><code>{esc(campaign.get(key))}</code></dd>'
        for key in ("campaign_id", "utc", "host", "git_sha", "rustc", "cargo_profile",
                    "harness", "sc_compose_cli_version", "python_sc_compose")
    )
    body = "\n".join(rows)
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>sc-runtime Monty benchmark — {esc(campaign['campaign_id'])}</title>
<style>
:root {{ color-scheme: dark; }}
body {{ font: 16px system-ui, sans-serif; max-width: 72rem; margin: 2rem auto; padding: 0 1rem; line-height: 1.6; background: #0f1419; color: #e7ecf3; }}
section {{ background: #1a2332; border: 1px solid #30363d; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; overflow-x: auto; }}
th, td {{ padding: .5rem; text-align: left; vertical-align: top; border-bottom: 1px solid #30363d; }}
table {{ width: 100%; border-collapse: collapse; }}
pre {{ background: #0b0f14; padding: .75rem; overflow-x: auto; }}
.strategy, dt {{ color: #9aa8bc; }}
dd {{ overflow-wrap: anywhere; }}
</style>
</head>
<body>
<h1>Monty compose hook benchmark</h1>
<section><h2>Campaign</h2><dl>
{metadata}
</dl><p>Campaign status: <strong>{esc(campaign['status'])}</strong>.</p>
<p>MEASURED means complete execution with one or more unratified (null) numerical ceilings; it is not a numerical performance PASS.</p></section>
<section><h2>Methodology</h2>
<p>Release-profile Rust harness. Each case has one untimed warmup before its recorded timed iterations. Wall-clock measurements use Rust Instant over the timed block only.</p>
<p>Subprocess RSS uses a separate one-shot macOS /usr/bin/time -l invocation. In-process RSS is getrusage(RUSAGE_SELF) high-water after the timed block and includes the test binary and Monty VM; it is not an isolated daemon measurement. Per-case memory notes identify the measurement method.</p></section>
<section><h2>Results</h2><table>
<thead><tr><th>Case and strategy</th><th>Mean / run</th><th>Total</th><th>Iterations</th><th>Status / reason</th><th>Ceiling</th><th>Memory</th></tr></thead>
<tbody>
{body}
</tbody></table></section>
<section><h2>Code examples</h2>
{examples}
</section>
<section><h2>Reproduce</h2><pre>just --justfile scripts/benchmark/justfile benchmark monty</pre>
<p>Run on the controlled host from the recorded source revision. Raw Rust harness JSON/HTML remain local diagnostics; this complete published HTML is rendered from the campaign JSON.</p></section>
</body>
</html>
"""


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, json.JSONDecodeError, ReportError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
