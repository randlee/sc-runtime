---
id: a-1
phase: a
status: planned
closure_type: contract
target_boundary: compatibility-evidence
vertical_rationale: "The production boundaries do not exist; one disposable composition proves the external interfaces they will use."
track: evidence
wave: 0
branch: sprint/a-1-compatibility-spike
worktree: sprint/a-1-compatibility-spike
recommended_agent: solar
recommended_model: sol
must_follow: []
parallel_safe: []
requirements:
  - REQ-RUN-0101
  - REQ-RUN-0102
  - NFR-RUN-0001
  - NFR-RUN-0007
  - REQ-RUN-0306
adrs:
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0302
owned_paths:
  - "examples/spike/**"
  - "docs/architecture.md"
---

# Sprint a-1 — compatibility spike

## Outcome and closure

Prove only the external facts that the core libraries and generator need.
The spike is disposable evidence, not an early framework or template.

## Decision prerequisites

Choose the smallest operation, route/tool name, and MCP proof client needed to
exercise the stack. Do not treat a proposed library signature as verified until
the spike compiles and runs it. Record exact versions rather than introducing
compatibility wrappers.

## Dependencies and parallel safety

There is no predecessor. a-2, a-3, and a-4 consume its recorded versions and
demonstrated third-party signatures.

## Deliverables

1. **REQ-RUN-0101** — One throwaway server combines Axum, utoipa-axum,
   stateless rmcp, SQLite, a UDS listener, and one service function; a Clap
   client reaches it using reqwest's supported UDS facility.
2. **REQ-RUN-0102** — Record the actual versions and results for the design's
   Verify items, including cargo-generate conditional/exclude/value-file syntax.
3. **NFR-RUN-0001** — Measure whether the selected feature layout keeps Axum,
   rmcp, and sqlx out of the default CLI dependency graph.
4. **REQ-RUN-0306** — Prove plain cargo-generate can consume the proposed
   template mechanics; do not build the product template here.

## Acceptance criteria

1. `boundary:compatibility-evidence` — curl over UDS, the Clap client, and an
   MCP tools call all reach the same service function in one process.
2. `boundary:compatibility-evidence` — every recorded version and signature is
   taken from the running spike or tool output, not memory or proposed prose.
3. `ADR-RUN-0003` — the dependency measurement records default and server
   selections without inventing a policy that remains an owner decision.

## This sprint does not close

No public SC crate API, daemon lifecycle, generated application, CLI policy,
release artifact, or wizard is closed here.

## Paths to delete

None in this sprint. a-7 deletes `examples/spike/**` before the release tag.

## Required validation

Build and run each proof with the exact recorded versions. Save commands,
platform, commit, and results in `docs/architecture.md`. If a Verify item fails,
stop dependent planning rather than adding an adapter automatically.

