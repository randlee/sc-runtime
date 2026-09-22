---
id: a-4
phase: a
status: planned
closure_type: boundary
target_boundary: sc-command
track: core
wave: 1
branch: sprint/a-4-command
worktree: sprint/a-4-command
recommended_agent: arch-ctm
recommended_model: terra
must_follow: [a-1]
parallel_safe: [a-2, a-3]
requirements:
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0004
  - REQ-CMD-0005
  - REQ-CMD-0006
  - NFR-CMD-0001
  - NFR-CMD-0002
  - NFR-CMD-0003
  - REQ-RUN-0002
  - REQ-RUN-0005
  - NFR-RUN-0003
  - NFR-RUN-0004
adrs:
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RUN-0001
  - ADR-RUN-0003
  - ADR-RUN-0006
owned_paths:
  - "crates/sc-command/**"
  - "boundaries/sc-command/**"
  - "docs/sc-command/requirements.md"
  - "docs/sc-command/architecture.md"
---

# Sprint a-4 — sc-command

## Outcome and closure

Create one independently usable crate for a lossless response envelope and
standard REST/MCP conversions. It is a wire helper, not a command registry,
handler framework, CLI generator, or test-generation system.

## Decision prerequisites

Resolve the envelope version/representation, error-kind/status mapping,
details and suggestion forms, concrete rmcp carrier, and serialization-failure
behavior needed for interoperability. Record those choices in the owned
sc-command requirement/ADR sources before implementation proceeds beyond the
facade. Leave application command shape open.

## Public contract produced

This sprint owns the sc-command types and conversions in the canonical
[minimal public contract handoff](phase-a-plan.md#minimal-public-contract-handoff):
`Envelope`, `OpError`, `ErrorKind`, result conversion, and server-feature
Axum/rmcp conversions. The crate-local requirement and ADR sources are the
decision record; application command shape remains outside this boundary.

## Dependencies and parallel safety

- `must_follow a-1`: consumes demonstrated Axum/rmcp types and versions.
- `parallel_safe a-2` and `a-3`: owned paths and boundaries are disjoint.

## Deliverables

1. **REQ-CMD-0001 / REQ-CMD-0003 / REQ-CMD-0006** — A generic, lossless
   envelope that converts results without discarding received versions or
   unknown JSON payload content.
2. **REQ-CMD-0002** — Typed operation errors reuse published observability
   error-code/remediation types rather than duplicating them.
3. **REQ-CMD-0004 / REQ-CMD-0005** — Server-feature conversions use ordinary
   Axum and rmcp response types and do not panic on serialization failure.
4. **NFR-CMD-0001 / NFR-CMD-0002** — Minimal default/server feature graphs,
   focused tests, and a compiling standalone README example.
5. **REQ-RUN-0005** — Boundary manifest for the actual facade and edges.

## Acceptance criteria

1. `boundary:sc-command` — canonical success/failure/null/unknown-field vectors
   round-trip under default features.
2. `boundary:sc-command` — server conversions produce the decided HTTP and MCP
   carriers with the same envelope semantics.
3. `ADR-CMD-0003` — inspection finds no registry, handler macro, attribute
   macro, transport client, runtime dependency, or generated-test framework.

## This sprint does not close

Application command declarations, CLI rendering policy, attribute generation,
and end-to-end surface parity close outside Phase A or in the minimal template
proof only.

## Paths to delete

None.

## Required validation

Run default/server tests and doctests, dependency-tree checks, workspace build
once registered, and boundary validation.
