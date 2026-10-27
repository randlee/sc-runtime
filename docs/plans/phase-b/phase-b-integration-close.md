# Phase B integration close and parallel Monty extension

Two tracks. They do not share an integration branch, and the Monty extension
does not gate phase close.

## Track A — Phase close on `integrate/phase-b`

Waves 1–3 (`b-1` through `b-9`). Metrics: [phase-b-plan.md](phase-b-plan.md) wave table.

**Trigger:** `integrate/phase-b` contains the landed stack (sc-compose
reference **#73**, then phase-close **#72**, bottom first) and CI on that
branch is green.

**Goal:** phase-end QA per `.claude/project/quality-policy.md` (the phase-end
reviewer set, including `flaky-test-qa`), then merge `integrate/phase-b` to
`develop`.

1. Open the PR `integrate/phase-b` → `develop`.
2. `quality-mgr` runs the phase-end set on that PR.
3. A finding stays open until the reviewer type that filed it records PASS on
   the fix commit. Fix branches cut from `integrate/phase-b`.
4. Merge only when `quality-mgr` reports PASS and CI is green.

Monty extension work is not a substitute for this gate. Phase-end QA has not
run on `integrate/phase-b` as of the **#73 / #72** stack.

## Track B — Monty extension on `integrate/phase-b-monty`

**Branch:** `integrate/phase-b-monty`, cut from `integrate/phase-b` once `b-9`
is on that branch. Cut it without waiting for Track A to merge. Sprint and
fix branches for `b-10` and `b-11` target `integrate/phase-b-monty`. Only the
later extension PR targets `develop`.

| Sprint | Doc | Wave | Closes |
|---|---|---|---|
| `b-10` | [sprint-b-10-sc-cron-monty-host-bridge.md](sprint-b-10-sc-cron-monty-host-bridge.md) | 4 | `render_template` resumed inside `monty_run::execute_feed` |
| `b-11` | [sprint-b-11-benchmark-monty-reports.md](sprint-b-11-benchmark-monty-reports.md) | 5 | campaign publish, including the `production_bridge` case |

Extension metrics and the `b-10` → `b-11` edge rationale: [phase-b-plan.md](phase-b-plan.md).
Track B does not gate Track A and stays off `integrate/phase-b`.

## gh stack on Track A

Stack **#74** on `integrate/phase-b`: **#73** (bottom) → **#72** (top). Merge
bottom first.
