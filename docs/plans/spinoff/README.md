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
preserve their pre-merge discovery notes. The landed b-10 through b-12
documents describe the implementation baseline; continuation amendments in
the binding ADR/REQ text and b-13 through b-18 are the planned product
authority. Findings inside an outline are annotated with their binding owner;
historical candidate splits are provenance and must not be turned into
additional sprints.

**Beads provenance:** `run-nwk` supplied the observability outline. Do not
create another epic or sprint from either retained outline; the hardened phase
plan is the work queue.

**Hardened result:** b-13 through b-18 include REQ/NFR/ADR ids,
`owned_paths`, dependencies, agent recommendations, acceptance criteria, and
exact validation commands. b-10 through b-12 are baseline artifacts. The
continuation consumes their resume loop and process limits; b-16 explicitly
replaces the pre-release fixed host policy with application extensions and
both CLI adapters.
The hardened cut assigns cron component logger propagation/events to b-13,
worker/due-run events to b-15, the Monty registry/CLI adapters and generated
product work to b-16, and the
shared Python/Monty hook-environment contract to b-17. It requires the
upstream non-blocking `try_log` adapter contract, records continuation width
3, and blocks b-18 until a later exact sc-publish kit pin excludes
`docs/sprints/**` from its install package.
