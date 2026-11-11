---
id: d-4
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-generation-tooling
track: generation-tooling
wave: 1
branch: sprint/d-4-generation-tooling
worktree: sprint/d-4-generation-tooling
pr_target: integrate/phase-d
vertical_rationale: One executable-selection defect crosses the Python generator and its existing CI installation action; it changes no crate or Cargo manifest.
recommended_agent: cipher
recommended_model: luna
requirements:
  - REQ-RUN-0005
  - REQ-RUN-0305
  - REQ-RUN-0501
  - REQ-RUN-0502
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0401
github_issues: []
parallel_safe: [d-1, d-2]
parallel_safe_rationale: Owns only generator tooling, its action and dedicated new tests. d-1 alone owns Justfile and workflow files; existing action install-generation/compose-version inputs are preserved.
must_follow: []
owned_paths:
  - .sc/generation-tools.json
  - .github/actions/rust-build-cache/action.yml
  - scripts/new_project.py
  - .claude/skills/plan-anchoring/scripts/change_report.py
  - .codex/skills/plan-anchoring/scripts/change_report.py
  - .cursor/skills/plan-anchoring/scripts/change_report.py
  - tests/unit/generation/test_generation_tool*.py
  - docs/development/generation-tools.md
paths_to_delete: []
---

# Sprint d-4 — generation tooling

## Goal and deliverables

Keep consumers free to select a compatible generation CLI independently of the
repository CI pin. This sprint owns the whole existing rust-build-cache action,
including its installation logic; d-1 calls its unchanged public inputs and
makes no edit to that file. No new shared workflow or Cargo change is needed.

| Deliverable | Requirement served |
| --- | --- |
| Declare CLI minimum >=1.8.0, qualify selected GitHub Release archive installation and preserve action input contract | REQ-RUN-0005; ANC-005/020 |
| Verify SC_COMPOSE before PATH and real generation with compatible1.8.0, independently of CI's chosen version | REQ-RUN-0305/0501/0502; ANC-003 |
| Keep the three report/version-check copies aligned; enforce a lower bound with deterministic comparison tests | REQ-RUN-0005/0305; ANC-015 |

## Acceptance and validation

1. **req:REQ-RUN-0005/install** — The selected CLI comes from GitHub Release
   archives with checksum and exact selected-version verification. The declared
   minimum is at least1.8.0. Preserve existing Rust cache/toolchain behavior and
   action inputs. Existing workflow callers need no edit to use this action.
2. **req:REQ-RUN-0502/selection** — A real generation run succeeds on compatible
   sc-compose1.8.0; SC_COMPOSE selects an independent executable ahead of PATH
   without changing the CI pin. Preserve fail-before-generation on missing tools.
3. **req:REQ-RUN-0305/floor** — Controlled inputs prove equal/newer versions
   accepted and older/malformed versions rejected; real generation and report
   rendering prove the commands actually execute. No timing/sleep assertions.
4. Run `python3 -m unittest discover -s tests/unit/generation -p 'test_generation_tool*.py'`
   and the real generator/report smoke procedures documented in the owned tool
   guide. Run `git diff --check`. Do not present stub executables as live proof.

## This sprint does not close

Dependency ranges and shared workflow edits are d-1; READMEs are d-2; released
consumer security/upgrade proof is d-3. d-3 consumes the checked CLI action and
executable-selection behavior for kit generated-consumer preflight, so that
artifact joins its existing wave2 inputs. No additional serial wave is added.
