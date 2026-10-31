#!/usr/bin/env python3
"""Verify committed benchmark evidence without running a controlled-host campaign.

Measurements are produced and reviewed separately. This check verifies their
recorded provenance and release equivalence, not the honesty of their author;
independent quality-manager review remains required.
"""
from __future__ import annotations

import importlib.util
import json
import math
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CASES = frozenset({"cli", "py_render", "py_hook", "monty", "production_bridge"})


class EvidenceError(Exception):
    """Committed evidence does not establish release readiness."""


def git(root: Path, *args: str) -> str:
    try:
        return subprocess.run(["git", *args], cwd=root, check=True, text=True,
                              capture_output=True, timeout=30).stdout
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
        raise EvidenceError(f"git evidence check failed: {' '.join(args)}") from exc


def report_change(line: str, directories: set[str]) -> bool:
    status, _, path = line.partition("\t")
    parent, _, name = path.rpartition("/")
    if parent not in directories:
        return False
    if name == "index.html":
        return status in {"A", "M"}
    return status == "A" and bool(re.fullmatch(r"\d{8}T\d{6}Z-[A-Za-z0-9._-]+\.(?:campaign\.json|html)", name))


def positive_number(value: object) -> bool:
    return (not isinstance(value, bool) and isinstance(value, (int, float))
            and math.isfinite(value) and value > 0)


def verify_campaign(root: Path, payload: dict, path: Path, module, directories: set[str]) -> None:
    campaign_id = payload.get("campaign_id")
    if payload.get("schema") != module.CAMPAIGN_SCHEMA:
        raise EvidenceError("invalid campaign schema")
    if not isinstance(campaign_id, str) or not re.fullmatch(r"\d{8}T\d{6}Z-rand-m5", campaign_id):
        raise EvidenceError("invalid controlled-host campaign id")
    if path.name != f"{campaign_id}.campaign.json" or payload.get("report_html") != f"{campaign_id}.html":
        raise EvidenceError("campaign filenames do not match their id")
    try:
        when = datetime.strptime(payload["utc"], "%Y-%m-%dT%H:%M:%SZ")
    except (KeyError, TypeError, ValueError) as exc:
        raise EvidenceError("invalid campaign UTC") from exc
    if campaign_id != when.strftime("%Y%m%dT%H%M%SZ-rand-m5"):
        raise EvidenceError("campaign id does not match UTC")
    if (payload.get("host") != "rand-m5" or payload.get("cargo_profile") != "release"
            or payload.get("harness") != "crates/sc-cron/tests/sc_compose_bench.rs"
            or not str(payload.get("rustc", "")).startswith("rustc 1.96.1 ")
            or payload.get("python_sc_compose") is not True
            or not str(payload.get("sc_compose_cli_version", "")).startswith("sc-compose ")):
        raise EvidenceError("wrong host, profile, harness, or tool metadata")
    sha = payload.get("git_sha")
    if not isinstance(sha, str) or not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise EvidenceError("invalid measured source SHA")
    try:
        resolved = git(root, "rev-parse", "--verify", f"{sha}^{{commit}}").strip()
    except EvidenceError:
        # Squash merges may leave the measured commit outside branch ancestry.
        # Fetch only that immutable GitHub object; no benchmark host is contacted.
        git(root, "fetch", "--no-tags", "origin", sha)
        resolved = git(root, "rev-parse", "--verify", f"{sha}^{{commit}}").strip()
    if resolved != sha:
        raise EvidenceError("measured SHA is not a commit")
    changes = git(root, "diff", "--name-status", "--no-renames", sha, "HEAD").splitlines()
    if any(not report_change(line, directories) for line in changes):
        raise EvidenceError("release source differs from measured source outside added reports/index")
    raw_cases = payload.get("cases")
    if (not isinstance(raw_cases, list) or len(raw_cases) != len(CASES)
            or any(not isinstance(case, dict) for case in raw_cases)
            or {case.get("id") for case in raw_cases} != CASES):
        raise EvidenceError("campaign must contain all five unique required cases")
    baselines = module.load_baselines()
    expected = module.annotate_cases(raw_cases, baselines)
    for recorded, recomputed in zip(raw_cases, expected):
        if (recorded.get("skipped") is not False or recorded.get("skip_reason") is not None
                or type(recorded.get("timed_iters")) is not int or recorded["timed_iters"] <= 0
                or not positive_number(recorded.get("ms_per_run"))
                or not positive_number(recorded.get("total_ms"))):
            raise EvidenceError("case skipped or has invalid numeric measurements")
        if not math.isclose(recorded["total_ms"] / recorded["timed_iters"], recorded["ms_per_run"], rel_tol=1e-5, abs_tol=1e-5):
            raise EvidenceError("case totals do not match per-run measurements")
        floor = recomputed["max_ms_per_run"]
        if floor is not None and not positive_number(floor):
            raise EvidenceError("invalid current baseline")
        if recorded.get("max_ms_per_run") != floor or recorded.get("status") != recomputed["status"]:
            raise EvidenceError("case status does not match current baselines")
    if payload.get("status") != module.campaign_status(expected) or payload.get("status") not in {"PASS", "MEASURED"}:
        raise EvidenceError("campaign is not complete and accepted by current baselines")
    module.validate_gate(payload)
    html = (path.parent / payload["report_html"]).read_text(encoding="utf-8")
    if any(value not in html for value in (campaign_id, sha, payload["host"], payload["rustc"])):
        raise EvidenceError("HTML report metadata does not match campaign")


def verify(root: Path = ROOT) -> list[str]:
    if git(root, "status", "--porcelain", "--untracked-files=no").strip():
        raise EvidenceError("tracked working tree/index must match committed HEAD")
    subsystems = sorted((root / "scripts/benchmark").glob("*/campaign.py"))
    if not subsystems:
        raise EvidenceError("no benchmark subsystems registered")
    directories = {f"site/reports/benchmark-{script.parent.name}" for script in subsystems}
    accepted = []
    for script in subsystems:
        if script.parent.name != "monty":
            raise EvidenceError(f"release evidence contract required for subsystem {script.parent.name}")
        spec = importlib.util.spec_from_file_location("release_evidence_campaign", script)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report_dir = root / "site/reports" / f"benchmark-{script.parent.name}"
        tracked = set(git(root, "ls-files", "--", str(report_dir.relative_to(root))).splitlines())
        if not report_dir.is_dir() or not tracked:
            raise EvidenceError("committed benchmark reports are missing")
        if any(str(path.relative_to(root)) not in tracked for path in report_dir.iterdir() if path.is_file()):
            raise EvidenceError("untracked benchmark report evidence must be committed")
        if any(path.is_symlink() for path in report_dir.iterdir()):
            raise EvidenceError("report evidence must be regular files")
        try:
            module.check_index(report_dir)
        except (module.CampaignError, OSError, ValueError, TypeError, KeyError) as exc:
            raise EvidenceError(str(exc)) from exc
        failures = []
        for path in sorted(report_dir.glob("*.campaign.json"), reverse=True):
            try:
                payload = module.load_json(path)
                verify_campaign(root, payload, path, module, directories)
                accepted.append(f"{path.relative_to(root)} source={payload['git_sha']} status={payload['status']}")
                break
            except (EvidenceError, module.CampaignError, OSError, ValueError, TypeError, KeyError) as exc:
                failures.append(f"{path.name}: {exc}")
        else:
            raise EvidenceError("no applicable reviewed-build evidence; " + "; ".join(failures))
    return accepted


def main() -> int:
    try:
        for receipt in verify():
            print(f"benchmark evidence verified: {receipt}")
        return 0
    except Exception as exc:
        print(f"benchmark evidence: FAIL: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
