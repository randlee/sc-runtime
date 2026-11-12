# Phase D intent

[PRD v2.0](../../prd/sc-runtime-prd.md) is the product intent and top authority.
Its storage and writer requirements (PRD-STO-1–5), cron requirements
(PRD-CRON-1–5), command pattern, engineering rules (including PRD-ENG-5/6)
and deferred decisions apply directly. This document adds only the Phase D
delta; it does not restate or override those requirements.

The executable plan is Beads epic `run-phase-d`, indexed by
[phase-d.jsonl](../phase-d.jsonl). [Anchors](anchors.md) retain approved
constraints and their verbatim provenance. The earlier Markdown plan and the
three-sprint/eight-anchor status are historical.

## Phase D delta

**Adoption.** Unblock beads-cache's SQLite security upgrade and make the fixed
SDK usable from the registry. Acceptance includes the actual upstream migration
PR open with CI green at a recorded revision against the published release,
without local path, patch or Git SDK substitutes. Generated application
acceptance follows the PRD and live sprint criteria; the superseded vulnerable
SQLx-SQLite configuration is not an accepted Phase D outcome (PRD-STO-2/3;
PRD-CRON-3).

**Dependency choice and recommendations.** Published/template registry requirements
use tested floors with no upper bound (ANC-037). Consumers may pin what they want.
The template ships a recommended Cargo.lock at the newest tested combination;
.sc-runtime/tested-dependencies.json records those versions for reference and
future revert/upgrade tools (ANC-038). The normal build and test suites check the selected versions.

**Release.** The recorded owner decision is coordinated **0.3.0** across nine
crates: `sc-storage`, `sc-runtime-db-sqlite`, `sc-config`, `sc-transport`,
`sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands` and `sc-monty` (owner
2026-11-12 requirement change item 1 adds the ninth crate; original version message
01M76S6D7T58E4CMKHJRCXFJ1J; `run-phase-d` release metadata). This supersedes
the earlier 0.2.1 target. The live plan owns package, advisory, integration and
consumer proof; this document claims no implementation or publication pass.

## Planning handover provenance

The user stated on 2026-11-11:

> The intention of phase-d is to make sc-runtime easy for consumers to use.
> today beads-cache is blocked because of the security warning on rusqlite.
> we need to be following 'best practices' for a scaffolding framework which does not block consumers from building or upgrading dependencies w/ non-breaking changes/upgrades.

Final owner approval uses the HTML diff after independent review. Historical
calibration packets and run evidence retain their original inputs.
