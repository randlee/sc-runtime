# Spinoff planning outlines (Codex handoff)

**Branch:** `plan/spinoff-monty-observability` (worktree off `develop`)  
**Purpose:** Source outlines retained for provenance. Their hardened Phase B
continuation is [b-13 through b-17](../phase-b/phase-b-plan.md#sprint-index);
the outlines are not implementation authority.

| Outline | Target sprint shape |
|---------|---------------------|
| [monty-hook-completion-outline.md](monty-hook-completion-outline.md) | Post–Track B Monty hook hardening and productization |
| [sc-observability-integration-outline.md](sc-observability-integration-outline.md) | Injectable logger across template daemon and libraries |

**Baseline:** `develop` at worktree creation. Track B Monty extension (`integrate/phase-b-monty`, PR #84) may land after these plans are written; the Monty outline references both **what `develop` already has (b-9)** and **what the integration branch adds (b-10–b-12)** so the planner can set `must_follow` correctly.

**Beads (optional):** `run-nwk` (observability); no epic yet for Monty completion—planner may create one.

**Hardened result:** b-13 through b-17 include REQ/NFR/ADR ids,
`owned_paths`, dependencies, agent recommendations, acceptance criteria, and
exact validation commands. PR #84 merge is an entry prerequisite; its b-10
through b-12 work is verified rather than replanned.
