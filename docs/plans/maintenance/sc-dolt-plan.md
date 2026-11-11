---
id: sc-dolt-plan
status: on_hold
branch: plan/sc-dolt
worktree: /Users/randlee/github/sc-runtime-worktrees/plan/sc-dolt
closure_type: plan
target_boundary: docs-only
---

# sc-dolt planning sprint (on hold)

## Hold policy

This plan **does not authorize implementation, workspace changes, crate
publication, or registry release.** It stays **on hold** until `quality-mgr`
reports **PASS** on plan QA-1 at a commit that includes these remediations and
is merged forward with current `develop`. Beads epic `run-sc-dolt` remains
planning-only until hold is lifted.

## Requirements and decisions

- **Requirements:** REQ-DOLT-0001 through REQ-DOLT-0006;
  REQ-RUN-0001, REQ-RUN-0002, REQ-RUN-0005, NFR-RUN-0004, NFR-RUN-0009
  (amendments listed below and in [sc-dolt architecture](../../sc-dolt/architecture.md));
  REQ-RT-0003, NFR-RT-0002 (daemon remains store-agnostic).
- **ADRs:** ADR-DOLT-0001 through ADR-DOLT-0004 (proposed);
  amending ADR-RUN-0001 point 4 (public session surface), ADR-RUN-0003
  (workspace edge row for `sc-dolt`), ADR-RUN-0004 (`sc-observability-types`
  only), ADR-RUN-0301 point 6 and ADR-RUN-0305 point 9 (native Dolt remote ops
  exception).
- **Evidence:** [Dolt v2.6.2 research pack](../../research/dolt-v2.6.2/README.md).

## Deliverables

1. [docs/sc-dolt/requirements.md](../../sc-dolt/requirements.md) with every
   REQ-DOLT id using the shared template (Statement, Rationale, Success
   Criteria).
2. [docs/sc-dolt/architecture.md](../../sc-dolt/architecture.md) with proposed
   ADRs (Status, Decision Date, Source), binding root ADR amendments, workspace
   edge table, public type sketches (no sqlx re-exports), and `sc-observability-types`
   injection only.
3. Evidence pack corrections (checkout vs probe, initial branch-operation list,
   consumer-owned SQLite export).
4. This sprint document and an updated [project-plan](../../project-plan.md)
   index with **on hold** status.

## Acceptance criteria (plan QA-1)

1. **req:** Each REQ-DOLT-0001…0006 has verifiable success criteria traceable
   from this sprint's requirements list.
2. **req:** REQ-RUN-0001/0002/0005 and NFR-RUN-0009 amendments are written in
   architecture; NFR-RUN-0004 is upheld (library reduces duplicated Dolt session
   code, not application HTTP policy).
3. **arch:** ADR-RUN-0003 row and ADR-RUN-0004 edge for `sc-dolt` are explicit
   before implementation; ADR-RUN-0001 documents opaque session handles with
   no public sqlx types.
4. **req:** Branch checkout evidence matches `probes/probe.md` (dirty-main probe).
5. **req:** One initial branch-operation list: create, list, select, merge,
   remote fetch/push/pull; delete/rename/`DOLT_CHECKOUT` deferred.
6. **req:** SQLite snapshot export is consumer-owned; excluded from `sc-dolt`
   acceptance.
7. CI `just lint and just test` green on the plan branch after merge with
   `develop`.
8. Plan remains **on hold** in `project-plan.md` until quality-mgr PASS.

## Sprint requirements list

| Requirement | Acceptance root in this doc |
| --- | --- |
| REQ-DOLT-0001 | Criterion 2, 4 |
| REQ-DOLT-0002 | Criteria 4, 5 |
| REQ-DOLT-0003 | Architecture credential resolver + success criteria in requirements |
| REQ-DOLT-0004 | Architecture ADR-DOLT-0004 + ADR-RUN-0301/0305 amendments |
| REQ-DOLT-0005 | REQ-DOLT-0002 merge diagnostics |
| REQ-DOLT-0006 | Criteria 6, 8 |
| REQ-RUN-0001 amendment | Criterion 2 (`sc-dolt` workspace member) |
| REQ-RUN-0002 amendment | Independent crate test/README when implemented |
| REQ-RUN-0005 amendment | `boundaries/sc-dolt/sc-dolt.toml` at implementation |
| NFR-RUN-0009 amendment | `DoltError` enum in architecture sketch |
| REQ-RT-0003 / NFR-RT-0002 | Daemon does not depend on `sc-dolt` |

## Beads delivery cut (unchanged scope, still blocked)

| Beads child | Scope when hold lifts |
| --- | --- |
| `.1` branch sessions | Crate skeleton, validated targets, branch lease, create/list/select |
| `.2` native operations | Merge, conflict diagnostics, remote fetch/push/pull |
| `.3` two-consumer integration | Pinned SQLx + two table models |

## Owned paths

- `docs/plans/maintenance/sc-dolt-plan.md`
- `docs/sc-dolt/requirements.md`
- `docs/sc-dolt/architecture.md`
- `docs/project-plan.md` (sc-dolt section only)
- `docs/research/dolt-v2.6.2/` (evidence corrections only)

## This sprint does not close

Rust implementation, `Cargo.toml` workspace membership, boundary TOML on disk,
crate publication, or changes to generated templates.
