# Phase B integration close and parallel Monty extension

Two tracks. They do not share an integration branch, and the Monty extension
does not gate phase close.

## Track A — Phase close (complete)

Waves 1–3 (`b-1` through `b-9`). Metrics: [phase-b-plan.md](phase-b-plan.md) wave table.

**Status: complete.** `develop` is at `a448385` ("Phase B: integrate waves 1–3
on develop"), which contains the landed stack (sc-compose reference **#73**
and phase-close **#72**). Nothing remains to run on `integrate/phase-b` for
waves 1–3. The Monty extension below is independent of that merge and is not a
substitute for it.

For reference, the gate that closed Track A was phase-end QA per
`.claude/project/quality-policy.md` (the phase-end reviewer set, including
`flaky-test-qa`), with fix branches cut from `integrate/phase-b` and a merge
only on `quality-mgr` PASS and green CI.

## Track B — Monty extension on `integrate/phase-b-monty`

**Branch:** `integrate/phase-b-monty`, cut from `develop` (at or after
`a448385`, which contains `b-9`). Sprint and fix branches for `b-10` and `b-11`
target `integrate/phase-b-monty`. Only the later extension PR targets `develop`.

| Sprint | Doc | Wave | Closes |
|---|---|---|---|
| `b-10` | [sprint-b-10-sc-cron-monty-host-bridge.md](sprint-b-10-sc-cron-monty-host-bridge.md) | 4 | `render_template` resumed inside `monty_run::execute_feed` |
| `b-11` | [sprint-b-11-benchmark-monty-reports.md](sprint-b-11-benchmark-monty-reports.md) | 5 | multi-subsystem benchmark platform, HTML code examples, publish-preflight `benchmark-pass` gate, and the `monty` campaign including `production_bridge` |

Extension metrics and the `b-10` → `b-11` edge rationale: [phase-b-plan.md](phase-b-plan.md).
Track B does not reopen Track A. Its `b-11` benchmark gate does apply to later binary publishes through `release/publish-preflight.toml`.

## gh stack on Track A (historical)

Stack **#74** on `integrate/phase-b`: **#73** (bottom) → **#72** (top). Merged to `develop` at `a448385`.
