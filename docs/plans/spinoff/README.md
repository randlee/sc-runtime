# Spinoff planning outlines (Codex handoff)

**Branch:** `plan/spinoff-monty-observability` (worktree off `develop`)  
**Purpose:** Input for a Codex agent to produce full hardened sprint plans (plan-hardening skill), not implementation.

| Outline | Target sprint shape |
|---------|---------------------|
| [monty-hook-completion-outline.md](monty-hook-completion-outline.md) | Post–Track B Monty hook hardening and productization |
| [sc-observability-integration-outline.md](sc-observability-integration-outline.md) | Injectable logger across template daemon and libraries |

**Baseline:** `develop` at worktree creation. Track B Monty extension (`integrate/phase-b-monty`, PR #84) may land after these plans are written; the Monty outline references both **what `develop` already has (b-9)** and **what the integration branch adds (b-10–b-12)** so the planner can set `must_follow` correctly.

**Beads (optional):** `run-nwk` (observability); no epic yet for Monty completion—planner may create one.

**Planner should:** read linked sprint docs, ADRs, QA triage under `.triage/phase-b/findings/` on the integration branch, and `docs/sc-runtime/observability-injection.md`; emit sprint markdown under `docs/plans/phase-b/` or a new phase folder with REQ/NFR/ADR ids, owned_paths, acceptance criteria, and validation commands.
