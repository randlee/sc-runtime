---
id: a-2
phase: a
status: planned
closure_type: boundary
target_boundary: sc-config
track: core
wave: 1
branch: sprint/a-2-config
worktree: sprint/a-2-config
recommended_agent: arch-ctm
recommended_model: terra
must_follow: [a-1]
parallel_safe: [a-3, a-4]
requirements:
  - REQ-CFG-0001
  - REQ-CFG-0002
  - REQ-CFG-0003
  - REQ-CFG-0004
  - NFR-CFG-0001
  - NFR-CFG-0002
  - NFR-CFG-0003
  - REQ-RUN-0002
  - REQ-RUN-0005
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
adrs:
  - ADR-CFG-0001
  - ADR-CFG-0002
  - ADR-CFG-0003
  - ADR-RUN-0003
  - ADR-RUN-0006
  - ADR-RUN-0007
owned_paths:
  - "crates/sc-config/**"
  - "boundaries/sc-config/**"
---

# Sprint a-2 — sc-config

## Outcome and closure

Create one independently usable synchronous configuration crate. Its public
surface is the smallest contract needed to load typed JSON configuration with
environment overrides; it does not prescribe generated application structure.

## Decision prerequisites

Resolve the source requirements' OPEN choices for environment ordering,
invalid names, non-Unicode values, public constructor signatures, and error
variant names. Record only decisions necessary to make the crate coherent.

## Dependencies and parallel safety

- `must_follow a-1`: consumes verified dependency versions.
- `parallel_safe a-3` and `a-4`: owned paths and public boundaries are disjoint.

## Deliverables

1. **REQ-CFG-0001 / REQ-CFG-0002** — JSON defaults/local overrides and
   app-prefixed environment overrides with deterministic merge behavior.
2. **REQ-CFG-0003** — An explicit-input loader suitable for parallel tests,
   plus a convenience entry point for normal process use.
3. **REQ-CFG-0004 / NFR-CFG-0001** — Typed, source-preserving errors and no
   reachable panic path for malformed external input.
4. **NFR-CFG-0002 / NFR-CFG-0003** — Minimal dependencies, no async/reload
   layer, focused tests, and a compiling standalone README example.
5. **REQ-RUN-0005** — The sc-config boundary manifest describing only its
   facade and actual dependency restrictions.

## Acceptance criteria

1. `boundary:sc-config` — crate tests cover file, merge, environment, malformed
   input, and explicit-input parallel behavior; the README example compiles.
2. `boundary:sc-config` — the crate builds independently with only its approved
   normal dependencies and no sibling SC crate.
3. `ADR-CFG-0003` — inspection finds no reload, watcher, async runtime, global
   mutable registry, or generated-application policy.

## This sprint does not close

Daemon assembly, template configuration layout, application defaults, and
generated-project behavior close in a-5 or a-6.

## Paths to delete

None.

## Required validation

Run crate tests, doctests, formatting/linting, workspace build once registered,
and configured boundary validation. Check the default dependency tree.

