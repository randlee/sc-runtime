# Repository Quality Policy

This file contains repository-specific QA policy. Reusable agents and skills
must read this file rather than embedding repository names, commands,
interfaces, approval authorities, or temporary architectural exceptions in
their own prompts.

## Repository Baseline

- Default comparison branch: `develop`
- Requirements index: `docs/requirements.md`
- Architecture index: `docs/architecture.md`
- Framework charter (multi-consumer posture): `docs/framework-charter.md`
- Project plan: `docs/project-plan.md` when present (not yet created)
- Team protocol: `docs/team-protocol.md`
- Crate docs: `docs/<crate>/requirements.md` and
  `docs/<crate>/architecture.md`
- Developer roster: `docs/development/developer-roster.md`

## Reviewer Policy

- Initial implementation review: `req-qa`, `arch-qa`,
  `extensibility-qa`, `ruthless-boundary-qa`, `rust-qa-agent`,
  `rust-best-practices-agent`, and `rust-service-hardening-agent`
- Fix verification: `req-qa`, `arch-qa`, and `rust-qa-agent`; dispatch an
  additional reviewer only when rechecking a carry-forward finding owned by
  that reviewer
- Plan review QA-1: `req-qa`, `arch-qa`, `extensibility-qa`,
  `ruthless-boundary-qa`, `rust-best-practices-agent`,
  `rust-service-hardening-agent`, and `ceremony-qa`
- Plan review QA-2 and later: `req-qa` and `arch-qa` scoped to the
  dispatched findings; dispatch another reviewer only when rechecking a
  carry-forward finding it owns. Plan QA is capped at 3 rounds
  (`plan_qa_cycle_limit`); a round that leaves only minor findings reports
  `PASS — minor fixes required, no re-QA`
- `ceremony-finding-screen`: every sprint or plan QA round that has
  findings, over all of them, before the report is posted
- Phase-end review: the initial implementation set plus `flaky-test-qa`
- Run `flaky-test-qa` earlier when tests changed or instability is suspected
- Run `schema-reviewer` only when the governed-interface section below names
  an interface relevant to the assigned scope
- Every QA round, plan or sprint, requires an open PR; the rendered report
  is posted to it every round

## Release and publishing

- **Kit:** [sc-publish](https://github.com/randlee/sc-publish), vendored per
  `release/sc-publish-pin.toml` and `docs/development/sc-publish.md`.
- **Channels (Phase A):** GitHub Releases + **crates.io** for the four library
  crates; no optional package-manager channels until declared in
  `release/sc-publish-consumer-input.json`.
- **Do not** maintain a hand-written root `release.yml` that diverges from the
  kit; sync via `install.py` after input or pin changes.
- **Cursor phase-end:** `/cursor-publish` after the last sprint merges to
  `main` (see `.cursor/skills/cursor-orchestration/SKILL.md`).

## Validation Policy

No repository-wide phase-end artifact command is configured yet. An
implementation phase must add its authoritative command here before phase-end
QA can pass. Reviewer assignments may provide narrower commands for earlier
rounds.

## Repository layout (reviewer expectations)

Reviewers must read this section before treating missing paths as defects.
Early Phase A work may exist only as plans and bootstrap lint files.

| Path | When required |
| --- | --- |
| `docs/requirements.md`, `docs/architecture.md` | Every review |
| `docs/project-plan.md` | When present; not required before first phase plan lands |
| `docs/plans/phase-<phase>/` | Plan and sprint reviews for that phase |
| `docs/<crate>/requirements.md`, `docs/<crate>/architecture.md` | Sprint owns or references that crate |
| `crates/<crate>/` | Sprint deliverable includes that crate |
| `boundaries/<crate>/*.toml` | Sprint deliverable includes boundary manifests (see below) |
| Root `Cargo.toml` workspace | Maintainer workspace sprints (Phase A a-5 onward) |

Absent paths that this table marks as not yet required are **not** Blocking
in `doc_review` or pre-implementation sprints.

## Boundary Enforcement

**Product:** sc-lint **0.8.0** (`minimum_version` in root `sc-lint.toml`).

**Local integration:** `sc-lint init --just` manages `justfile`, `sc-lint.toml`,
and `.sc-lint/bootstrap.py`. First use on a clean checkout: `just setup`.

| Check | Command |
| --- | --- |
| Full maintainer lint (after workspace exists) | `just lint full` or `just lint` |
| Boundary + line counts only (early sprints) | `just lint gates` |
| Boundary analyzer directly | `sc-lint lint sc-boundary --root . --config sc-lint.toml` |

**Manifest paths:** `boundaries/<crate>/*.toml` for `sc-config`, `sc-transport`,
`sc-command`, and `sc-runtime` (see `boundaries/README.md`).

**Generated projects:** template ships the consumer contract (`sc-lint.toml`,
`.sc-lint/bootstrap.py`, public `setup`/`lint`/`test`/`upgrade`); fixture CI
runs `just setup && just lint` (sprint a-6).

Boundary relaxation until manifests exist: lead ruling -> ADR -> manifest update.

## Plan Naming

One convention for every phase. Everything is lower case.

| Thing | Form | Example |
|---|---|---|
| Phase id | next unused letter pair, lower case | `bc` |
| Sprint id | `<phase>-<n>`, `n` from 1 | `bc-4` |
| Plan directory | `docs/plans/phase-<phase>/` | `docs/plans/phase-bc/` |
| Phase plan | `docs/plans/phase-<phase>/phase-<phase>-plan.md` | `phase-bc-plan.md` |
| Sprint doc | `docs/plans/phase-<phase>/sprint-<phase>-<n>-<slug>.md` | `sprint-bc-4-task-ledger-writer.md` |
| Plan branch | `plan/phase-<phase>`, PR to `develop` | `plan/phase-bc` |
| Phase branch | `integrate/phase-<phase>`, cut from `develop` | `integrate/phase-bc` |
| Sprint branch | `sprint/<phase>-<n>-<slug>` | `sprint/bc-4-task-ledger-writer` |
| Fix layer | `fix/<phase>-<n>-<slug>`; phase-level: `fix/<phase>-<slug>` | `fix/bc-4-qa1` |

- Phase plans never live outside `docs/plans/phase-<phase>/`.
- Every sprint declares its branch in `.sprints/` (`triage:branch`). Tools
  read the declared branch and never infer it from a file name.

## Governed Interfaces

No governed interface policy is currently defined. Until this section names
an interface, its compatibility rule, evidence paths, and approval authority,
`schema-reviewer` returns `SKIPPED`.

## Repository Exceptions

Phase ids `a` and `b` are single letters. Later phases use the letter-pair form in Plan Naming.
