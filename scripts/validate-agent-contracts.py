#!/usr/bin/env python3
"""Validate repository-neutral, revision-pinned QA orchestration contracts."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from xml.etree import ElementTree


ROOT = Path(__file__).resolve().parents[1]

REVIEWER_PROMPTS = (
    ".claude/agents/req-qa.md",
    ".claude/agents/arch-qa.md",
    ".claude/agents/flaky-test-qa.md",
    ".claude/agents/ruthless-boundary-qa.md",
    ".claude/agents/rust-qa-agent.md",
    ".claude/agents/rust-best-practices-agent.md",
    ".claude/agents/rust-code-reviewer.md",
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
    "synaptic canvas",
    "herdr",
    "rand's",
    "randlee",
    "just validate",
    "just lint",
    "just test",
)


def fail(message: str, failures: list[str]) -> None:
    failures.append(message)


def render_qa_strict(variables: dict[str, object], failures: list[str]) -> ElementTree.Element | None:
    executable = shutil.which("sc-compose")
    if executable is None:
        return None
    template = ROOT / ".claude/skills/codex-orchestration/qa-template.xml.j2"
    with tempfile.NamedTemporaryFile("w", suffix=".json", encoding="utf-8") as handle:
        json.dump(variables, handle)
        handle.flush()
        result = subprocess.run(
            [
                executable,
                "render",
                "--root",
                str(ROOT),
                "--strict",
                "--check-render",
                "--json",
                "--file",
                str(template),
                "--var-file",
                handle.name,
            ],
            capture_output=True,
            text=True,
            check=False,
        )
    if result.returncode != 0:
        fail(f"qa-template.xml.j2: strict render failed: {result.stderr.strip()}", failures)
        return None
    envelope = json.loads(result.stdout)
    errors = [
        diagnostic
        for diagnostic in envelope["diagnostics"]
        if diagnostic["severity"] == "error"
    ]
    if errors:
        fail(f"qa-template.xml.j2: strict validation errors: {errors}", failures)
        return None
    try:
        return ElementTree.fromstring(envelope["payload"]["body"])
    except ElementTree.ParseError as error:
        fail(f"qa-template.xml.j2: rendered invalid XML: {error}", failures)
        return None


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

    for relative in (
        ".claude/skills/codex-orchestration/ruthless-boundary-qa-assignment.json.j2",
        ".claude/assets/sc-rust/quality-mgr/templates/rust-best-practices-assignment.json.j2",
        ".claude/assets/sc-rust/quality-mgr/templates/rust-service-hardening-assignment.json.j2",
    ):
        if '"findings_scope_locked"' not in (ROOT / relative).read_text():
            fail(f"{relative}: missing fix-round scope lock", failures)

    qa_template = (ROOT / ".claude/skills/codex-orchestration/qa-template.xml.j2").read_text()
    for fragment in (
        "  - commit",
        "<commit><![CDATA[{{ commit | string | cdata_escape }}]]></commit>",
        "bd close {{ task_id | string | cdata_escape }}",
        "<![CDATA[",
    ):
        if fragment not in qa_template:
            fail(f"qa-template.xml.j2: missing {fragment}", failures)
    for forbidden in ("{% autoescape false", "{% macro", "&lt;", "&gt;", "&amp;", "&quot;"):
        if forbidden in qa_template:
            fail(f"qa-template.xml.j2: forbidden blanket/raw entity workaround {forbidden}", failures)

    adversarial = 'quoted "value" \\ path\nline <tag>& snowman ☃ ]]> twice ]]>'
    attribute_adversarial = adversarial.replace("\n", " ")
    qa_variables: dict[str, object] = {
        "task_id": attribute_adversarial,
        "sprint": attribute_adversarial,
        "sprint_doc": adversarial,
        "review_mode": adversarial,
        "description": adversarial,
        "pr_number": 8,
        "branch": adversarial,
        "commit": adversarial,
        "worktree_path": adversarial,
        "commits": [adversarial, "def456"],
        "review_targets": [adversarial],
        "references": [adversarial],
        "lead": adversarial,
        "cc": adversarial,
        "changed_files": [adversarial],
        "triage_records": [adversarial],
    }
    root = render_qa_strict(qa_variables, failures)
    if root is not None:
        for field, variable in (
            ("pr-number", "pr_number"),
            ("commits", "commits"),
            ("review-targets", "review_targets"),
            ("changed-files", "changed_files"),
            ("triage-records", "triage_records"),
            ("references", "references"),
        ):
            if json.loads(root.findtext(field, default="null")) != qa_variables[variable]:
                fail(f"qa-template.xml.j2: {field} did not round-trip", failures)

    string_variables = {
        **qa_variables,
        "pr_number": "",
        "commits": "HEAD",
        "review_targets": "- src/",
        "changed_files": "",
        "triage_records": "",
        "references": "- plan",
    }
    string_root = render_qa_strict(string_variables, failures)
    if string_root is not None:
        for field, expected in (
            ("pr-number", ""),
            ("commits", "HEAD"),
            ("review-targets", "\n- src/\n  "),
            ("changed-files", "\n\n  "),
            ("triage-records", "\n\n  "),
            ("references", "\n- plan\n  "),
        ):
            if string_root.findtext(field) != expected:
                fail(f"qa-template.xml.j2: {field} string semantics changed", failures)

    quality_manager = (ROOT / ".claude/agents/quality-mgr.md").read_text()
    for fragment in (
        ".claude/project/quality-policy.md",
        "exact `branch`, `commit`, and `worktree_path`",
        "git show <commit>:<path>",
        "git diff <integration_branch>...<commit> --name-only",
        "every carried finding remains part of the merge gate",
        "bd update <task-id> --claim",
        "bd close <task-id>",
    ):
        if fragment not in quality_manager:
            fail(f"quality-mgr.md: missing {fragment}", failures)
    if "...HEAD --name-only" in quality_manager:
        fail("quality-mgr.md: moving HEAD used for review target discovery", failures)

    review_template = (
        ROOT / ".claude/skills/codex-orchestration/review-template.xml.j2"
    ).read_text()
    for fragment in (
        "HEAD` exactly `{{ commit | string | cdata_escape }}",
        "git show {{ commit | string | cdata_escape }}:<path>",
        "never at a newer branch tip or moving worktree state",
    ):
        if fragment not in review_template:
            fail(f"review-template.xml.j2: missing pinned-review invariant {fragment}", failures)

    for relative, text in (
        (".claude/agents/quality-mgr.md", quality_manager),
        (".claude/skills/codex-orchestration/qa-template.xml.j2", qa_template),
        (".claude/skills/codex-orchestration/review-template.xml.j2", review_template),
    ):
        for forbidden in ("current branch/worktree", "current branch and worktree"):
            if forbidden in text:
                fail(f"{relative}: moving-checkout evidence instruction {forbidden!r}", failures)

    for fragment in (
        "`RBQA-*`, `RBP-*`, or `RSH-*`",
        "Every carried finding remains in the merge gate",
    ):
        if fragment not in qa_template:
            fail(f"qa-template.xml.j2: missing fix-round invariant {fragment}", failures)

    for relative in (
        ".claude/agents/quality-mgr.md",
        ".claude/skills/codex-orchestration/SKILL.md",
        ".claude/skills/quality-management-gh/SKILL.md",
    ):
        text = (ROOT / relative).read_text()
        for forbidden in ("unconditionally omit", "carry to the next phase backlog"):
            if forbidden in text:
                fail(f"{relative}: contradictory fix-round policy {forbidden!r}", failures)

    report_json_fields = {
        ".claude/skills/quality-management-gh/findings-report.md.j2": (
            "sprint_id",
            "task_id",
            "branch",
            "commit",
            "verdict",
            "merge_readiness",
            "merge_reason",
            "next_action",
            "action_owner",
        ),
        ".claude/skills/quality-management-gh/quality-report.md.j2": (
            "sprint_id",
            "task_id",
            "branch",
            "commit",
            "verdict",
            "merge_readiness",
            "merge_reason",
            "recommendation",
        ),
    }
    for relative, fields in report_json_fields.items():
        text = (ROOT / relative).read_text()
        for field in fields:
            fragment = f"{{{{ {field} | tojson }}}}"
            if fragment not in text:
                fail(f"{relative}: machine-status string {field} must use tojson", failures)
        if "{{ blocking_ids_json | safe }}" not in text:
            fail(f"{relative}: blocking_ids_json must remain typed JSON", failures)

    schema_assignment = (
        ROOT / ".claude/skills/codex-orchestration/schema-reviewer-assignment.json.j2"
    ).read_text()
    for fragment in (
        "  policy_path: .claude/project/quality-policy.md",
        '  notes: ""',
    ):
        if fragment not in schema_assignment:
            fail(f"schema-reviewer-assignment.json.j2: invalid default {fragment}", failures)

    rust_best_practices_skill = (
        ROOT / ".claude/skills/rust-best-practices/SKILL.md"
    ).read_text()
    for field in ('"branch"', '"commit"', '"carry_forward_findings"', '"findings_scope_locked"'):
        if field not in rust_best_practices_skill:
            fail(f"rust-best-practices/SKILL.md: assignment example missing {field}", failures)
    if "never dispatch it against implicit unstaged changes" not in rust_best_practices_skill:
        fail("rust-best-practices/SKILL.md: unpinned rust-code-reviewer delegation", failures)

    rust_service_skill = (
        ROOT / ".claude/skills/rust-service-hardening/SKILL.md"
    ).read_text()
    for field, minimum_count in (
        ('"branch"', 2),
        ('"commit"', 2),
        ('"carry_forward_findings"', 1),
        ('"findings_scope_locked"', 1),
    ):
        if rust_service_skill.count(field) < minimum_count:
            fail(
                f"rust-service-hardening/SKILL.md: assignment examples missing {field}",
                failures,
            )
    if "never dispatch it against implicit unstaged changes" not in rust_service_skill:
        fail("rust-service-hardening/SKILL.md: unpinned rust-code-reviewer delegation", failures)

    rust_code_reviewer = (ROOT / ".claude/agents/rust-code-reviewer.md").read_text()
    for fragment in (
        "git show <commit>:<path>",
        '"reviewed_branch"',
        '"reviewed_commit"',
    ):
        if fragment not in rust_code_reviewer:
            fail(f"rust-code-reviewer.md: missing pinned-review invariant {fragment}", failures)
    if "review unstaged changes from `git diff`" in rust_code_reviewer:
        fail("rust-code-reviewer.md: moving-checkout default remains", failures)

    quality_management_skill = (
        ROOT / ".claude/skills/quality-management-gh/SKILL.md"
    ).read_text()
    if "`commit` unchanged from the QA assignment" not in quality_management_skill:
        fail("quality-management-gh/SKILL.md: report commit is not assignment-pinned", failures)
    if "`commit` from `git rev-parse`" in quality_management_skill:
        fail("quality-management-gh/SKILL.md: report commit uses moving checkout", failures)

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
