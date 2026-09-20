#!/usr/bin/env python3
"""Validate repository-neutral, revision-pinned QA orchestration contracts."""

from __future__ import annotations

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]

REVIEWER_PROMPTS = (
    ".claude/agents/req-qa.md",
    ".claude/agents/arch-qa.md",
    ".claude/agents/flaky-test-qa.md",
    ".claude/agents/ruthless-boundary-qa.md",
    ".claude/agents/rust-qa-agent.md",
    ".claude/agents/rust-best-practices-agent.md",
    ".claude/agents/rust-service-hardening-agent.md",
    ".claude/agents/schema-reviewer.md",
)

ASSIGNMENT_TEMPLATES = (
    ".claude/skills/codex-orchestration/req-qa-assignment.json.j2",
    ".claude/skills/codex-orchestration/arch-qa-assignment.json.j2",
    ".claude/skills/codex-orchestration/flaky-test-qa-assignment.json.j2",
    ".claude/skills/codex-orchestration/ruthless-boundary-qa-assignment.json.j2",
    ".claude/assets/sc-rust/quality-mgr/templates/rust-qa-assignment.json.j2",
    ".claude/assets/sc-rust/quality-mgr/templates/rust-best-practices-assignment.json.j2",
    ".claude/assets/sc-rust/quality-mgr/templates/rust-service-hardening-assignment.json.j2",
    ".claude/skills/codex-orchestration/schema-reviewer-assignment.json.j2",
)

SHARED_ROOTS = (
    ".claude/agents",
    ".claude/assets/sc-rust",
    ".claude/skills/codex-orchestration",
    ".claude/skills/plan-hardening",
    ".claude/skills/quality-management-gh",
    ".claude/skills/rust-best-practices",
    ".claude/skills/rust-service-hardening",
)

FORBIDDEN = (
    "sc-runtime",
    "sc-lint",
    "atm-core",
    "herdr",
    "rand's",
    "randlee",
    "just validate",
    "just lint",
    "just test",
)


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def main() -> int:
    failures: list[str] = []

    for relative in REVIEWER_PROMPTS:
        path = ROOT / relative
        text = path.read_text()
        for field in ('"branch"', '"commit"', '"worktree_path"'):
            if field not in text:
                fail(f"{relative}: missing pinned input field {field}", failures)

    for relative in ASSIGNMENT_TEMPLATES:
        path = ROOT / relative
        text = path.read_text()
        for required in ("  - branch", "  - commit", "  - worktree_path"):
            if required not in text:
                fail(f"{relative}: missing required variable {required.strip()}", failures)
        for field in ('"branch"', '"commit"', '"worktree_path"'):
            if field not in text:
                fail(f"{relative}: missing rendered field {field}", failures)

    qa_template = (ROOT / ".claude/skills/codex-orchestration/qa-template.xml.j2").read_text()
    for fragment in (
        "  - commit",
        "<commit><![CDATA[{{ cdata_value(commit) }}]]></commit>",
        "bd close {{ cdata_value(task_id) }}",
        "<![CDATA[",
    ):
        if fragment not in qa_template:
            fail(f"qa-template.xml.j2: missing {fragment}", failures)
    for forbidden in ("{% autoescape false", "&lt;", "&gt;", "&amp;", "&quot;"):
        if forbidden in qa_template:
            fail(f"qa-template.xml.j2: forbidden blanket/raw entity workaround {forbidden}", failures)

    quality_manager = (ROOT / ".claude/agents/quality-mgr.md").read_text()
    for fragment in (
        ".claude/project/quality-policy.md",
        "exact `branch`, `commit`, and `worktree_path`",
        "bd update <task-id> --claim",
        "bd close <task-id>",
    ):
        if fragment not in quality_manager:
            fail(f"quality-mgr.md: missing {fragment}", failures)

    for root in SHARED_ROOTS:
        for path in sorted((ROOT / root).rglob("*")):
            if not path.is_file():
                continue
            text = path.read_text(errors="replace").lower()
            relative = path.relative_to(ROOT)
            for token in FORBIDDEN:
                if token in text:
                    fail(f"{relative}: repository-specific token {token!r}", failures)

    if failures:
        for failure in failures:
            print(f"ERROR: {failure}", file=sys.stderr)
        return 1

    print("agent contracts: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
