#!/usr/bin/env python3
"""Package sc_compose_bench output into site/reports/benchmark-monty/.

The harness writes HTML and JSON under ``target/sc-compose-bench/`` (or
``$CARGO_TARGET_DIR/sc-compose-bench``). This script copies that evidence
into an immutable campaign and regenerates index.html from every
``*.campaign.json`` file. Default CI does not invoke it.

Campaign JSON schema: ``sc-runtime/benchmark-monty-campaign/v1``.
Harness JSON schema: ``sc-runtime/benchmark-monty-harness/v1``.
"""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

_BENCH = Path(__file__).resolve().parent.parent
if str(_BENCH) not in sys.path:
    sys.path.insert(0, str(_BENCH))
from check_gate import GateHostError, assert_gate_host  # noqa: E402
from check_gate import host_label as gate_host_label  # noqa: E402
from render_report import (  # noqa: E402
    ReportError,
    apply_examples_to_html,
    example_blocks_html,
    require_examples,
)

SUBSYSTEM_DIR = Path(__file__).resolve().parent
ROOT = SUBSYSTEM_DIR.parents[2]
REPORT_DIR = ROOT / "site" / "reports" / "benchmark-monty"
BASELINES_PATH = SUBSYSTEM_DIR / "baselines.json"
HARNESS_HTML_NAME = "sc-compose-hook-comparison.report.html"
HARNESS_JSON_NAME = "sc-compose-hook-comparison.report.json"
HARNESS_SCHEMA = "sc-runtime/benchmark-monty-harness/v1"
CAMPAIGN_SCHEMA = "sc-runtime/benchmark-monty-campaign/v1"
BASELINE_SCHEMA = "sc-runtime/benchmark-monty-baselines/v1"


class CampaignError(Exception):
    """The campaign could not be measured or published."""


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def stamp(when: datetime) -> str:
    return when.astimezone(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def harness_output_dir() -> Path:
    override = os.environ.get("CARGO_TARGET_DIR")
    base = Path(override) if override else ROOT / "target"
    return base / "sc-compose-bench"


def harness_html_path() -> Path:
    return harness_output_dir() / HARNESS_HTML_NAME


def harness_json_path() -> Path:
    return harness_output_dir() / HARNESS_JSON_NAME


def host_label() -> str:
    try:
        return gate_host_label()
    except GateHostError as exc:
        raise CampaignError(str(exc)) from exc


def run(cmd: list[str], *, cwd: Path = ROOT, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, env=env, text=True, capture_output=True, check=False)


def git_sha() -> str:
    completed = run(["git", "rev-parse", "HEAD"])
    if completed.returncode != 0:
        raise CampaignError(completed.stderr.strip() or "git rev-parse failed")
    return completed.stdout.strip()


def toolchain_rustc(prefix: list[str]) -> str | None:
    completed = run([*prefix, "--version"])
    if completed.returncode != 0:
        return None
    text = (completed.stdout or completed.stderr).strip()
    return text or None


def monty_cargo() -> tuple[list[str], str]:
    """Return a cargo argv prefix and the rustc version string.

    Rust 1.96.1 is required for the Monty feature. Some machines expose that
    compiler as the ``1.96`` channel rather than a ``1.96.1`` toolchain name.
    """
    probes: list[tuple[list[str], list[str]]] = [
        (["rustup", "run", "1.96.1", "rustc"], ["rustup", "run", "1.96.1", "cargo"]),
        (["rustup", "run", "1.96", "rustc"], ["rustup", "run", "1.96", "cargo"]),
    ]
    for rustc_cmd, cargo_cmd in probes:
        version = toolchain_rustc(rustc_cmd)
        if version and version.startswith("rustc 1.96.1"):
            return cargo_cmd, version
    raise CampaignError(
        "Rust 1.96.1 is required for the Monty benchmark. "
        "Install it with `rustup toolchain install 1.96.1`."
    )


def load_json(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise CampaignError(f"{path}: invalid JSON") from exc
    if not isinstance(payload, dict):
        raise CampaignError(f"{path}: JSON root must be an object")
    return payload


def load_baselines() -> dict:
    path = BASELINES_PATH
    payload = load_json(path)
    if payload.get("schema") != BASELINE_SCHEMA:
        raise CampaignError(f"{path}: schema must be {BASELINE_SCHEMA}")
    cases = payload.get("cases")
    if not isinstance(cases, dict):
        raise CampaignError(f"{path}: cases must be an object")
    return payload


def floor_for(baselines: dict, case_id: str) -> float | None:
    spec = baselines.get("cases", {}).get(case_id)
    if not isinstance(spec, dict):
        return None
    floor = spec.get("max_ms_per_run")
    if floor is None:
        return None
    if isinstance(floor, bool) or not isinstance(floor, (int, float)):
        raise CampaignError(f"baseline {case_id}: max_ms_per_run must be a number or null")
    return float(floor)


def case_status(case: dict, floor: float | None) -> str:
    if case.get("skipped"):
        return "SKIPPED"
    ms = case.get("ms_per_run")
    if not isinstance(ms, (int, float)) or isinstance(ms, bool):
        return "INCOMPLETE"
    if floor is None:
        return "MEASURED"
    if float(ms) > floor:
        return "FAIL"
    return "PASS"


def campaign_status(cases: list[dict]) -> str:
    statuses = [str(case.get("status")) for case in cases]
    measured = [status for status in statuses if status != "SKIPPED"]
    if not measured or any(status == "INCOMPLETE" for status in measured):
        if not any(status in {"PASS", "MEASURED", "FAIL"} for status in statuses):
            return "INCOMPLETE"
    if "FAIL" in statuses:
        return "FAIL"
    if "INCOMPLETE" in statuses:
        return "INCOMPLETE"
    if "MEASURED" in statuses:
        return "MEASURED"
    if measured and all(status == "PASS" for status in measured):
        return "PASS"
    return "INCOMPLETE"


def annotate_cases(harness_cases: list, baselines: dict) -> list[dict]:
    annotated: list[dict] = []
    for case in harness_cases:
        if not isinstance(case, dict) or not isinstance(case.get("id"), str):
            raise CampaignError("harness case is missing a string id")
        floor = floor_for(baselines, case["id"])
        annotated.append({**case, "max_ms_per_run": floor, "status": case_status(case, floor)})
    return annotated


def assert_examples(payload: dict, *, label: str) -> None:
    try:
        require_examples(payload, label=label)
    except ReportError as exc:
        raise CampaignError(str(exc)) from exc


def require_production_resume_loop(examples: dict) -> None:
    """The published resume loop is the production ``execute_feed_with_compose_program`` excerpt.

    The lab ``feed_start`` harness stays a separate constant. Packaging
    rejects a resume loop that is not labelled as the production path.
    """
    resume = examples.get("resume_loop")
    if not isinstance(resume, str):
        raise CampaignError(
            "examples.resume_loop must be the production "
            "execute_feed_with_compose_program excerpt"
        )
    lowered = resume.lower()
    if "production" not in lowered:
        raise CampaignError(
            "examples.resume_loop must be labelled as the production path "
            "(keep the lab feed_start loop separate)"
        )
    if not any(name in lowered for name in ("execute_feed_with_compose_program", "execute_feed_with_logger", "drive_feed")):
        raise CampaignError(
            "examples.resume_loop must name execute_feed_with_compose_program or drive_feed "
            "(cfg(test) execute_feed is not the production path)"
        )
    if not any(name in lowered for name in ("execute_feed_with_compose_program", "execute_feed_with_logger")) and "execute_feed" in lowered:
        raise CampaignError(
            "examples.resume_loop must not label production as execute_feed only; "
            "name execute_feed_with_compose_program or drive_feed"
        )


def floor_table(baselines: dict) -> str:
    rows: list[str] = []
    cases = baselines.get("cases")
    if isinstance(cases, dict):
        for case_id in sorted(cases):
            spec = cases[case_id]
            floor = spec.get("max_ms_per_run") if isinstance(spec, dict) else None
            shown = "none" if floor is None else html.escape(str(floor))
            rows.append(
                f"<tr><td><code>{html.escape(str(case_id))}</code></td><td>{shown}</td></tr>"
            )
    if not rows:
        rows.append('<tr><td colspan="2" class="muted">No floors configured.</td></tr>')
    return "\n".join(rows)


def campaign_examples_html(campaigns: list[dict]) -> str:
    blocks: list[str] = []
    for campaign in campaigns:
        examples = campaign.get("examples")
        if not isinstance(examples, dict):
            continue
        try:
            rendered = example_blocks_html(examples)
        except ReportError:
            continue
        campaign_id = html.escape(str(campaign.get("campaign_id", "campaign")))
        blocks.append(f"<h3>{campaign_id}</h3>\n{rendered}")
    if not blocks:
        return '<p class="muted">No campaign code examples yet.</p>'
    return "\n".join(blocks)


def render_index(campaigns: list[dict], baselines: dict) -> str:
    rows: list[str] = []
    if not campaigns:
        rows.append('<tr><td colspan="6" class="muted">No published campaigns yet.</td></tr>')
    for campaign in campaigns:
        campaign_id = html.escape(str(campaign["campaign_id"]))
        report = html.escape(str(campaign["report_html"]))
        json_name = html.escape(f"{campaign['campaign_id']}.campaign.json")
        rustc = html.escape(str(campaign.get("rustc", "")))
        rows.append(
            "<tr>"
            f"<td><a href=\"{json_name}\">{campaign_id}</a></td>"
            f"<td>{html.escape(str(campaign['host']))}</td>"
            f"<td><code>{html.escape(str(campaign['git_sha'])[:12])}</code></td>"
            f"<td>{html.escape(str(campaign['status']))}</td>"
            f"<td>{rustc}</td>"
            f"<td><a href=\"{report}\">{report}</a></td>"
            "</tr>"
        )
    body = "\n".join(rows)
    floors = floor_table(baselines)
    examples = campaign_examples_html(campaigns)
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>sc-runtime · Monty benchmark reports</title>
  <style>
    body {{ font: 16px system-ui, sans-serif; max-width: 52rem; margin: 2rem auto; padding: 0 1rem; line-height: 1.6; }}
    nav a {{ margin-right: 1rem; }}
    table {{ border-collapse: collapse; width: 100%; margin-top: 1rem; }}
    th, td {{ border: 1px solid #cbd5e1; padding: 0.45rem; text-align: left; vertical-align: top; }}
    .muted {{ color: #64748b; }}
    pre {{ background: #f8fafc; border: 1px solid #cbd5e1; border-radius: 6px; padding: 0.75rem; overflow-x: auto; }}
    code {{ font-family: ui-monospace, Menlo, monospace; font-size: 0.85em; }}
  </style>
</head>
<body>
  <nav>
    <a href="../../">sc-runtime site</a>
    <a href="../../announcements/">Announcements</a>
  </nav>
  <h1>Monty compose hook benchmarks</h1>
  <p class="muted">
    Campaigns compare Python hooks with the Monty lab path. Run and publish
    via <code>.claude/skills/benchmark-run/SKILL.md</code>. This page is
    regenerated from <code>*.campaign.json</code>.
  </p>
  <h2>Campaigns</h2>
  <table>
    <thead><tr><th>Campaign</th><th>Host</th><th>Git</th><th>Status</th><th>rustc</th><th>Report</th></tr></thead>
    <tbody>
{body}
    </tbody>
  </table>
  <h2>Baseline floors</h2>
  <p class="muted">From <code>scripts/benchmark/monty/baselines.json</code>. <code>none</code> means the case is measured and does not fail the campaign.</p>
  <table>
    <thead><tr><th>Case</th><th>max_ms_per_run</th></tr></thead>
    <tbody>
{floors}
    </tbody>
  </table>
  <h2>Code examples</h2>
  <p class="muted">Copied from each campaign JSON (<code>examples.hook_source</code>, <code>examples.resume_loop</code>, <code>examples.cli_argv</code>).</p>
{examples}
</body>
</html>
"""


def load_campaigns(report_dir: Path) -> list[dict]:
    campaigns: list[dict] = []
    for path in sorted(report_dir.glob("*.campaign.json")):
        payload = load_json(path)
        if payload.get("schema") != CAMPAIGN_SCHEMA:
            raise CampaignError(f"{path}: schema must be {CAMPAIGN_SCHEMA}")
        for key in ("campaign_id", "host", "git_sha", "utc", "status", "report_html"):
            if not isinstance(payload.get(key), str) or not payload[key]:
                raise CampaignError(f"{path}: missing {key}")
        expected = f"{payload['campaign_id']}.campaign.json"
        if path.name != expected:
            raise CampaignError(f"{path}: filename must be {expected}")
        html_path = report_dir / payload["report_html"]
        if not html_path.is_file():
            raise CampaignError(f"{path}: missing report {html_path.name}")
        campaigns.append(payload)
    campaigns.sort(key=lambda item: (item["utc"], item["campaign_id"]), reverse=True)
    return campaigns


def write_index(report_dir: Path) -> str:
    campaigns = load_campaigns(report_dir)
    for campaign in campaigns:
        assert_examples(campaign, label=str(campaign.get("campaign_id", "campaign")))
    text = render_index(campaigns, load_baselines())
    path = report_dir / "index.html"
    path.write_text(text, encoding="utf-8")
    return text


def check_index(report_dir: Path) -> None:
    baselines = load_baselines()
    campaigns = load_campaigns(report_dir)
    for campaign in campaigns:
        assert_examples(campaign, label=str(campaign.get("campaign_id", "campaign")))
    expected = render_index(campaigns, baselines)
    path = report_dir / "index.html"
    actual = path.read_text(encoding="utf-8") if path.is_file() else ""
    if actual != expected:
        raise CampaignError(f"{path} is stale; regenerate with --index-only")


def run_bench() -> str:
    cargo, rustc_version = monty_cargo()
    env = os.environ.copy()
    env["SC_CRON_BENCH"] = "1"
    completed = subprocess.run(
        [
            *cargo,
            "test",
            "-p",
            "sc-cron",
            "--features",
            "monty",
            "--release",
            "--test",
            "sc_compose_bench",
            "--",
            "--ignored",
            "--nocapture",
        ],
        cwd=ROOT,
        env=env,
        check=False,
    )
    if completed.returncode != 0:
        raise CampaignError(
            f"sc_compose_bench exited {completed.returncode} before a campaign was written"
        )
    if not harness_json_path().is_file() or not harness_html_path().is_file():
        raise CampaignError(
            "benchmark finished without harness output under target/sc-compose-bench; "
            "not publishing a campaign"
        )
    return rustc_version


def package_campaign(
    *,
    report_dir: Path,
    harness: dict,
    html_path: Path,
    host: str,
    git_revision: str,
    rustc_version: str,
    when: datetime | None = None,
    baselines: dict | None = None,
) -> dict:
    if harness.get("schema") != HARNESS_SCHEMA:
        raise CampaignError(f"harness schema must be {HARNESS_SCHEMA}")
    raw_cases = harness.get("cases")
    if not isinstance(raw_cases, list) or not raw_cases:
        raise CampaignError("harness JSON has no cases")
    baseline_doc = baselines if baselines is not None else load_baselines()
    cases = annotate_cases(raw_cases, baseline_doc)
    when = when or utc_now()
    campaign_id = f"{stamp(when)}-{host}"
    json_path = report_dir / f"{campaign_id}.campaign.json"
    html_name = f"{campaign_id}.html"
    dest_html = report_dir / html_name
    if json_path.exists() or dest_html.exists():
        raise CampaignError(f"refusing to overwrite campaign {campaign_id}")
    examples = harness.get("examples")
    assert_examples({"examples": examples}, label="harness")
    if not isinstance(examples, dict):
        raise CampaignError("harness examples must be an object")
    require_production_resume_loop(examples)
    report_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(html_path, dest_html)
    payload = {
        "schema": CAMPAIGN_SCHEMA,
        "campaign_id": campaign_id,
        "host": host,
        "git_sha": git_revision,
        "rustc": rustc_version,
        "utc": when.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "status": campaign_status(cases),
        "report_html": html_name,
        "harness": "crates/sc-cron/tests/sc_compose_bench.rs",
        "cargo_profile": harness.get("cargo_profile"),
        "sc_compose_cli_version": harness.get("sc_compose_cli_version"),
        "python_sc_compose": harness.get("python_sc_compose"),
        "cases": cases,
        "examples": examples,
    }
    assert_examples(payload, label=campaign_id)
    dest_html.write_text(
        stamp_report_html(dest_html.read_text(encoding="utf-8"), payload),
        encoding="utf-8",
    )
    json_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    write_index(report_dir)
    return payload


_META_RE = re.compile(r"<!-- campaign-meta --><p class=\"subtitle\">.*?</p>", re.DOTALL)


def stamp_report_html(page: str, campaign: dict) -> str:
    """Record host, git SHA, and rustc on the published campaign HTML."""
    page = apply_examples_to_html(page, campaign["examples"])
    meta = (
        "<!-- campaign-meta --><p class=\"subtitle\">"
        f"Campaign {html.escape(str(campaign['campaign_id']))}"
        f" · host {html.escape(str(campaign['host']))}"
        f" · git <code>{html.escape(str(campaign['git_sha']))}</code>"
        f" · {html.escape(str(campaign['rustc']))}</p>"
    )
    if "<!-- campaign-meta -->" in page:
        return _META_RE.sub(meta, page, count=1)
    if "<body>" in page:
        return page.replace("<body>", "<body>\n" + meta, 1)
    return meta + "\n" + page


def index_only(report_dir: Path) -> None:
    """Regenerate index and campaign HTML from JSON. Does not run cargo."""
    paths = sorted(report_dir.glob("*.campaign.json"))
    for path in paths:
        payload = load_json(path)
        assert_examples(payload, label=str(path))
        html_name = payload.get("report_html")
        if not isinstance(html_name, str) or not html_name:
            raise CampaignError(f"{path}: missing report_html")
        html_path = report_dir / html_name
        if not html_path.is_file():
            raise CampaignError(f"{path}: missing report {html_name}")
        html_path.write_text(
            stamp_report_html(html_path.read_text(encoding="utf-8"), payload),
            encoding="utf-8",
        )
    write_index(report_dir)


def latest_report(report_dir: Path) -> Path | None:
    campaigns = load_campaigns(report_dir)
    if not campaigns:
        return None
    return report_dir / campaigns[0]["report_html"]


def open_report(path: Path) -> None:
    opener = shutil.which("open") or shutil.which("xdg-open")
    if opener is None:
        print(f"no desktop opener; report is {path}")
        return
    subprocess.run([opener, str(path)], check=False)


def show(report_dir: Path) -> None:
    write_index(report_dir)
    latest = latest_report(report_dir)
    index = report_dir / "index.html"
    print(f"Index: {index.relative_to(ROOT)}")
    if latest is None:
        raise CampaignError(f"no campaign HTML under {report_dir.relative_to(ROOT)}/")
    print(f"Latest: {latest.relative_to(ROOT)}")
    open_report(latest)


def publish(report_dir: Path) -> None:
    staged = run(["git", "diff", "--cached", "--name-only"])
    if staged.returncode != 0:
        raise CampaignError(staged.stderr.strip() or "git diff --cached failed")
    if staged.stdout.strip():
        raise CampaignError("refusing to publish: the index already has staged files")
    index_only(report_dir)
    check_index(report_dir)
    added = run(["git", "add", "--", str(report_dir.relative_to(ROOT))])
    if added.returncode != 0:
        raise CampaignError(added.stderr.strip() or "git add failed")
    names = run(["git", "diff", "--cached", "--name-only"])
    if names.returncode != 0:
        raise CampaignError(names.stderr.strip() or "git diff --cached failed")
    prefix = f"{report_dir.relative_to(ROOT).as_posix()}/"
    for line in names.stdout.splitlines():
        if line and not line.startswith(prefix):
            raise CampaignError(f"publish staged a path outside the report dir: {line}")
    print(names.stdout, end="" if names.stdout.endswith("\n") or not names.stdout else "\n")
    print("staged site/reports/benchmark-monty/ only; commit the evidence yourself")


def run_campaign() -> dict:
    rustc_version = run_bench()
    harness = load_json(harness_json_path())
    return package_campaign(
        report_dir=REPORT_DIR,
        harness=harness,
        html_path=harness_html_path(),
        host=host_label(),
        git_revision=git_sha(),
        rustc_version=rustc_version,
    )


def self_check() -> None:
    import tempfile

    when = datetime(2026, 10, 27, 18, 0, tzinfo=timezone.utc)
    harness = {
        "schema": HARNESS_SCHEMA,
        "cargo_profile": "release",
        "sc_compose_cli_version": "sc-compose 0.0.0",
        "python_sc_compose": True,
        "examples": {
            "hook_source": "def cron_tick(hook):\n    return sc_compose_run('render', hook['payload']['compose_args'])\n",
            "resume_loop": (
                "// production path: execute_feed_with_compose_program (not the lab feed_start harness)\n"
                "let mut progress = repl.feed_start(phase_snippet(phase), snippet_inputs(hook), PrintWriter::Disabled)?;\n"
                "resume_host(call, stage, phase, hook, hosts, deadline, logger)?;\n"
            ),
            "cli_argv": "sc-compose render --root <temp> --mode file --file t.j2 --var-file v.json\n",
        },
        "cases": [
            {"id": "cli", "label": "cli", "skipped": False, "ms_per_run": 3.0},
            {"id": "py_hook", "label": "py", "skipped": False, "ms_per_run": 30.0},
            {
                "id": "production_bridge",
                "label": "production",
                "skipped": True,
                "skip_reason": "b-10",
                "ms_per_run": None,
            },
        ],
    }
    with tempfile.TemporaryDirectory() as tmp:
        report_dir = Path(tmp)
        shutil.copyfile(BASELINES_PATH, report_dir / "baselines.json")
        html_path = report_dir / "source.html"
        html_path.write_text("<p>report</p>\n", encoding="utf-8")
        payload = package_campaign(
            report_dir=report_dir,
            harness=harness,
            html_path=html_path,
            host="self-check",
            git_revision="a" * 40,
            rustc_version="rustc 1.96.1 (self-check)",
            when=when,
        )
        if payload["status"] != "MEASURED":
            raise CampaignError(f"expected MEASURED with null floors, got {payload['status']}")
        if payload["campaign_id"] != "20261027T180000Z-self-check":
            raise CampaignError(payload["campaign_id"])
        check_index(report_dir)
        index = report_dir / "index.html"
        original = index.read_text(encoding="utf-8")
        index.write_text(original + "\n", encoding="utf-8")
        try:
            check_index(report_dir)
        except CampaignError:
            pass
        else:
            raise CampaignError("stale index was accepted")
        index.write_text(original, encoding="utf-8")
        failing = json.loads((report_dir / "baselines.json").read_text(encoding="utf-8"))
        failing["cases"]["cli"]["max_ms_per_run"] = 1
        (report_dir / "baselines.json").write_text(json.dumps(failing), encoding="utf-8")
        again = package_campaign(
            report_dir=report_dir,
            harness=harness,
            html_path=html_path,
            host="self-check",
            git_revision="b" * 40,
            rustc_version="rustc 1.96.1 (self-check)",
            when=when.replace(minute=1),
            baselines=failing,
        )
        if again["status"] != "FAIL":
            raise CampaignError(f"expected FAIL against a 1 ms floor, got {again['status']}")
        try:
            package_campaign(
                report_dir=report_dir,
                harness=harness,
                html_path=html_path,
                host="self-check",
                git_revision="b" * 40,
                rustc_version="rustc 1.96.1 (self-check)",
                when=when.replace(minute=1),
            )
        except CampaignError:
            pass
        else:
            raise CampaignError("overwrite was allowed")
        index_only(report_dir)
        published = (report_dir / f"{payload['campaign_id']}.html").read_text(encoding="utf-8")
        if "<pre><code>" not in published or "cron_tick" not in published:
            raise CampaignError("index-only did not refresh campaign HTML examples")
        if "git" not in published or payload["git_sha"] not in published:
            raise CampaignError("campaign HTML is missing the git SHA")
        index_text = (report_dir / "index.html").read_text(encoding="utf-8")
        for needle in (
            "cron_tick",
            "feed_start",
            "execute_feed_with_compose_program",
            "production",
            "sc-compose render --root",
            "max_ms_per_run",
        ):
            if needle not in index_text:
                raise CampaignError(f"index.html is missing {needle}")
        lab_only = json.loads(json.dumps(harness))
        lab_only["examples"]["resume_loop"] = (
            "let mut progress = repl.feed_start(snippet, inputs, PrintWriter::Disabled)?;\n"
            "call.resume(result)?;\n"
        )
        try:
            package_campaign(
                report_dir=report_dir,
                harness=lab_only,
                html_path=html_path,
                host="self-check",
                git_revision="c" * 40,
                rustc_version="rustc 1.96.1 (self-check)",
                when=when.replace(minute=2),
            )
        except CampaignError as exc:
            if "examples.resume_loop" not in str(exc):
                raise
        else:
            raise CampaignError("lab resume_loop was published as examples.resume_loop")
        check_gate_host_allowlist()
        broken_path = report_dir / f"{payload['campaign_id']}.campaign.json"
        broken = json.loads(broken_path.read_text(encoding="utf-8"))
        del broken["examples"]["hook_source"]
        broken_path.write_text(json.dumps(broken), encoding="utf-8")
        try:
            index_only(report_dir)
        except CampaignError as exc:
            if "hook_source" not in str(exc):
                raise
        else:
            raise CampaignError("missing examples.hook_source was accepted")
        try:
            validate_gate(payload)
        except CampaignError as exc:
            if "production_bridge" not in str(exc):
                raise
        else:
            raise CampaignError("gate accepted a skipped production_bridge")
    print("self-check ok")


def check_gate_host_allowlist() -> None:
    """The gate rejects a host outside rand-m5 and baselines.json unless bypassed."""
    import tempfile

    keys = ("SC_BENCH_HOST_LABEL", "SC_BENCH_GATE_ALLOW_ANY_HOST")
    saved = {key: os.environ.get(key) for key in keys}
    try:
        os.environ.pop("SC_BENCH_GATE_ALLOW_ANY_HOST", None)
        os.environ["SC_BENCH_HOST_LABEL"] = "not-a-controlled-host"
        try:
            assert_gate_host(BASELINES_PATH)
        except GateHostError as exc:
            text = str(exc)
            if "not-a-controlled-host" not in text or "rand-m5" not in text:
                raise CampaignError(text) from exc
        else:
            raise CampaignError("gate accepted a disallowed host label")
        os.environ["SC_BENCH_HOST_LABEL"] = "rand-m5"
        if assert_gate_host(BASELINES_PATH) != "rand-m5":
            raise CampaignError("rand-m5 was not allowed")
        os.environ["SC_BENCH_GATE_ALLOW_ANY_HOST"] = "1"
        os.environ["SC_BENCH_HOST_LABEL"] = "not-a-controlled-host"
        if assert_gate_host(BASELINES_PATH) != "not-a-controlled-host":
            raise CampaignError("SC_BENCH_GATE_ALLOW_ANY_HOST=1 did not bypass the allowlist")
        os.environ["SC_BENCH_GATE_ALLOW_ANY_HOST"] = "0"
        with tempfile.TemporaryDirectory() as tmp:
            recorded = Path(tmp) / "baselines.json"
            recorded.write_text(
                json.dumps({"allowed_hosts": ["lab-host"]}),
                encoding="utf-8",
            )
            os.environ.pop("SC_BENCH_GATE_ALLOW_ANY_HOST", None)
            os.environ["SC_BENCH_HOST_LABEL"] = "lab-host"
            if assert_gate_host(recorded) != "lab-host":
                raise CampaignError("a baselines.json host label was rejected")
            os.environ["SC_BENCH_HOST_LABEL"] = "other-host"
            try:
                assert_gate_host(recorded)
            except GateHostError:
                pass
            else:
                raise CampaignError("gate accepted a host missing from baselines.json")
    finally:
        for key, value in saved.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def validate_gate(payload: dict) -> None:
    assert_examples(payload, label="gate")
    examples = payload.get("examples")
    require_production_resume_loop(examples if isinstance(examples, dict) else {})
    cases = {case["id"]: case for case in payload.get("cases", []) if isinstance(case, dict)}
    bridge = cases.get("production_bridge")
    if bridge is None or bridge.get("skipped"):
        raise CampaignError("gate requires production_bridge to run (not skipped)")
    status = str(payload.get("status"))
    if status in {"FAIL", "INCOMPLETE", "SKIPPED"}:
        raise CampaignError(f"gate rejected campaign status {status}")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--index-only", action="store_true", help="regenerate index.html and exit")
    mode.add_argument("--check", action="store_true", help="fail when index.html is stale")
    mode.add_argument("--show", action="store_true", help="rebuild the index and open the newest report")
    mode.add_argument("--publish", action="store_true", help="stage site/reports/benchmark-monty only")
    mode.add_argument("--self-check", action="store_true", help="run the packager against a temp directory")
    mode.add_argument(
        "--gate",
        action="store_true",
        help="run a fresh campaign and fail unless production_bridge passes baselines",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        if args.self_check:
            self_check()
        elif args.index_only:
            index_only(REPORT_DIR)
            print(f"wrote {REPORT_DIR.relative_to(ROOT) / 'index.html'}")
        elif args.check:
            check_index(REPORT_DIR)
            print("index.html matches *.campaign.json")
        elif args.show:
            show(REPORT_DIR)
        elif args.publish:
            publish(REPORT_DIR)
        elif args.gate:
            try:
                assert_gate_host(BASELINES_PATH)
            except GateHostError as exc:
                raise CampaignError(str(exc)) from exc
            payload = run_campaign()
            validate_gate(payload)
            print(f"gate ok: {payload['campaign_id']} status={payload['status']}")
        else:
            payload = run_campaign()
            print(f"wrote {payload['campaign_id']} status={payload['status']}")
    except CampaignError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
