# Intent — phase-d

Written 2026-11-11 by randlee (plan-anchoring capture). Jev judges locked anchors,
not this page. Anchors live in `anchors.json` once locked; sprint delivery is in
[phase-d-plan.md](phase-d-plan.md).

## What this plan is for

Phase C shipped **0.2.0**. Maintenance **0.2.1** must clear **security and advisory
debt** (notably **rusqlite** / RUSTSEC-2026-0459 via [#132](https://github.com/randlee/sc-runtime/issues/132))
and **dependency truth** ([#133](https://github.com/randlee/sc-runtime/issues/133)):
tested `>=` floors, explicit caps when crates.io “latest” cannot land, and CI that
proves **low** and **high** resolve without surprising consumers.

A separate failure mode is **supply chain outside the workspace**: pinned **registry
crates** and **external CLIs** (for example `sc-compose`) must not flip CI red when
an unrelated upstream publish moves on crates.io. Downstream generated applications
may hold older floors (they could not move rusqlite when blocked by sqlx coexistence
or security trade-offs) while this repository upgrades on its own schedule. Phase d
must make both models **explicit and durable**—not ad hoc pin bumps after every
crates.io drift event.

Plan anchoring for phase d exists to **lock those constraints** as testable anchors
and to judge sprints d-1 / d-1b / d-2 against them, alongside REQ/ADR citations
already on the sprint docs.

## What success looks like

- **Registry consumers** get honest manifests: `>=` tested floors, documented
  `<next-major` caps where latest fails compile or ADR (rusqlite/sqlx ruling in d-1;
  sweep inventory in [issue-133-dependency-sweep.md](issue-133-dependency-sweep.md)),
  and a **minimal-versions** (or equivalent) recipe that fails when a bare caret
  collapses too far.
- **This repo’s CI** installs the **sc-compose CLI** at **≥ 1.8.0** (see
  `.sc/generation-tools.json`) from **immutable release binaries**, not
  `cargo install`, matching plan-anchoring 0.11.0. Unrelated `sc-composer`
  publishes must not break that pin until maintainers bump the JSON.
- **Generated / downstream consumers** can set `SC_COMPOSE` or PATH to another
  compatible `sc-compose` release without this repository being forced to the same
  pin for its own generation jobs.
- **sc-compose distribution** is tracked upstream:
  [sc-compose#1041](https://github.com/randlee/sc-compose/issues/1041) (immutable
  release installs vs `cargo install` drift; multi-consumer pins).

## What is deliberately out of scope

- sqlx major bump and sqlx 0.9 + rusqlite 0.41 same-process writer (ruled in d-1;
  not 0.2.1).
- Forcing every dependency to crates.io latest in one release (sweep §1 blockers).
- Rewriting sc-compose or sc-composer inside sc-runtime (upstream + CI contract only).

## Anchors

Anchors were captured on 2026-11-11 (ANC-001–ANC-015). **ANC-008** is **retired**
(sqlx/rusqlite same-process is already in [sprint d-1](sprint-d-1-rusqlite-0-41.md),
not a user anchor). The record of truth is [`anchors.json`](anchors.json); the
register is [`anchors.md`](anchors.md). Jev gates judge beads against locked
statements, not this intent page.
