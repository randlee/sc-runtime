# Agent charter — Track A only (phase B close)

**Owner bead:** `run-7sg`  
**Human name:** Phase B close agent (Track A)

This agent **must not** implement or edit Monty extension planning or product
work. That is Track B (`run-u64`, `run-47q`, PR #75, PR #76, `b-10`, `b-11`).

## In scope

1. Land stack **#74**: **#73** → **#72** → **`integrate/phase-b`** (`gh stack merge 74 --yes --merge` when CI green).
2. Open and maintain PR **`integrate/phase-b` → `develop`**.
3. Run **phase-end quality-mgr** (full reviewer set per `.claude/project/quality-policy.md`).
4. Fix findings on branches cut from **`integrate/phase-b`** (Grok 4.7 high fast implementation; filing QA agent re-review).
5. Merge to **`develop`** only when quality-mgr PASS and CI green.
6. Append status to [phase-b-close-agent-log.md](phase-b-close-agent-log.md) after each major step.

## Out of scope (hard stop)

| Item | Track B owner |
|------|----------------|
| PR #75 `plan/phase-b-monty-extension` → `develop` | Plan / Monty docs |
| PR #76 `sprint/b-11-benchmark-monty-reports` | Implementation spike (do not merge for phase close) |
| `integrate/phase-b-monty`, b-10, b-11 sprint docs | Extension execution |
| `.claude/skills/benchmark-run/`, `site/reports/benchmark-monty/` | b-11 |

Read [phase-b-integration-close.md](phase-b-integration-close.md) Track A only.

## Communication

- **Status log:** `docs/plans/phase-b/phase-b-close-agent-log.md` (this agent appends; humans read).
- **Issue tracker:** `bd show run-7sg` / `bd update run-7sg`.
- **New chat handoff:** paste this file path + “You are Track A only per AGENT-CHARTER-track-a-phase-close.md.”

## Current prerequisites

- **#73** `just lint and just test`: must be PASS.
- **#72** `just lint and just test`: must be PASS before stack merge.
- **`integrate/phase-b` tip** must advance past `d17fc8c` after stack merge before phase-end QA.
