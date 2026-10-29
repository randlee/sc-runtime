# Spinoff planning outlines (Codex handoff)

**Branch:** `plan/spinoff-monty-observability` (worktree off `develop`)  
**Purpose:** Source outlines retained for provenance. Their hardened Phase B
continuation is [b-13 through b-18](../phase-b/phase-b-plan.md#sprint-index);
the outlines are not implementation authority.

| Outline | Target sprint shape |
|---------|---------------------|
| [monty-hook-completion-outline.md](monty-hook-completion-outline.md) | Post–Track B Monty hook hardening and productization |
| [sc-observability-integration-outline.md](sc-observability-integration-outline.md) | Injectable logger across template daemon and libraries |

**Baseline:** `develop` after Track B Monty merged at `240e445`. The outlines
preserve their pre-merge discovery notes, but the landed b-10 through b-12
sprint documents and binding ADR/REQ text are authoritative.

**Beads (optional):** `run-nwk` (observability); no epic yet for Monty completion—planner may create one.

**Hardened result:** b-13 through b-18 include REQ/NFR/ADR ids,
`owned_paths`, dependencies, agent recommendations, acceptance criteria, and
exact validation commands. b-10 through b-12 are baseline artifacts that the
continuation consumes and verifies rather than replans.
The hardened cut assigns cron component logger propagation/events to b-13,
worker/due-run events to b-15, generated Monty product work to b-16, and the
shared Python/Monty hook-environment contract to b-17. It requires the
upstream non-blocking `try_log` adapter contract, records continuation width
3, and blocks b-18 until a later exact sc-publish kit pin excludes
`docs/sprints/**` from its install package.
