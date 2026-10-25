# Phase b — rusqlite writer and host lint profile

## Phase id

`b`. The owner named this phase b. That is an exception to the letter-pair default in `.claude/project/quality-policy.md`, consistent with the grandfathered phase id `a`.

Entry state is the workspace at tag `v0.1.0`: four library crates, template `store-sqlite` on sqlx 0.9 with no write actor, `just lint` running boundary, line-counts, fmt, and clippy. This phase does not retag `v0.1.0` and does not publish.

## Objective

Land `sc-runtime-db-sqlite`, the rusqlite write actor, as its own crate. Turn on the installed host analyzers and the Windows cross-check. The template sqlx store does not change.

## Boundary map

| Boundary | Sprint | Owns | Depends on |
|---|---|---|---|
| `sc-runtime-db-sqlite` | `b-1` | the writer crate, its manifest, the membership amendments already recorded in [ADR-RUN-0012](../../architecture.md) | none |
| host lint profile | `b-2` | `sc-lint.toml`, the hot-path grep, the xwin install declaration | none |

## Sprint index

| Sprint | Doc | Closure | Track | Wave |
|---|---|---|---|---|
| `b-1` | [sprint-b-1-sqlite-writer.md](sprint-b-1-sqlite-writer.md) | `boundary` | writer | 1 |
| `b-2` | [sprint-b-2-lint-profile.md](sprint-b-2-lint-profile.md) | `boundary` | lint | 1 |

## Wave table

| Wave | Sprints | Critical path | Width | Sprint count |
|---|---|---|---|---|
| 1 | `b-1`, `b-2` | 1 | 2 | 2 |

## Requirements and ADRs

- [REQ-RUN-0001](../../requirements.md), [REQ-RUN-0004](../../requirements.md), [REQ-RUN-0005](../../requirements.md), [REQ-RUN-0308](../../requirements.md), [NFR-RUN-0002](../../requirements.md), [NFR-RUN-0005](../../requirements.md), [NFR-RUN-0008](../../requirements.md), [NFR-RUN-0009](../../requirements.md), [ADR-RUN-0003](../../architecture.md), [ADR-RUN-0005](../../architecture.md), [ADR-RUN-0006](../../architecture.md), [ADR-RUN-0011](../../architecture.md), [ADR-RUN-0012](../../architecture.md), [ADR-RUN-0301](../../architecture.md) points 1–7.
- Acceptance for the writer contract roots at [ADR-RUN-0012](../../architecture.md) and [docs/sc-runtime-db-sqlite/architecture.md](../../sc-runtime-db-sqlite/architecture.md). `req:` roots only where a requirement statement is the subject.

## Out of this phase

- Implementing sc-lint-runtime rules or raising `minimum_version`. That work is [sc-lint#640](https://github.com/randlee/sc-lint/issues/640).
- A phase-wide checkpoint. Each sprint closes on its own boundary.

The old narrative at `docs/plan/phase-B/phase-B-plan.md` is a pointer to this file.
