---
id: a-5
phase: a
status: planned
closure_type: integration
target_boundary: runtime-composition
vertical_rationale: "sc-runtime is the one intended assembly boundary; its greenfield implementation and rewritten-spike consumer form one cohesive vertical closure after the three real leaf APIs exist."
track: core
wave: 2
branch: sprint/a-5-runtime-core
worktree: sprint/a-5-runtime-core
recommended_agent: solar
recommended_model: sol
must_follow: [a-2, a-3, a-4]
parallel_safe: []
requirements:
  - REQ-RT-0001
  - REQ-RT-0002
  - REQ-RT-0003
  - REQ-RT-0004
  - REQ-RT-0005
  - REQ-RT-0006
  - REQ-RT-0007
  - REQ-RT-0008
  - NFR-RT-0001
  - NFR-RT-0002
  - NFR-RT-0003
  - NFR-RT-0004
  - REQ-RUN-0001
  - REQ-RUN-0002
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0103
  - REQ-RUN-0204
  - REQ-RUN-0205
  - NFR-RUN-0002
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
adrs:
  - ADR-RT-0001
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-RT-0004
  - ADR-RUN-0001
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0009
  - ADR-RUN-0201
  - ADR-RUN-0202
owned_paths:
  - "Cargo.toml"
  - "Cargo.lock"
  - "justfile"
  - "crates/sc-runtime/**"
  - "boundaries/sc-runtime/**"
  - "examples/spike/**"
  - ".github/workflows/crates.yml"
  - "docs/validation/phase-a-runtime.md"
---

# Sprint a-5 — sc-runtime core

## Outcome and closure

Complete the reusable core milestone: implement the runtime assembly crate,
compose all four real libraries in the spike, and prove that the runtime owns
lifecycle rather than application policy.

The sprint spans one production boundary and its immediate composition root.
That is the vertical product boundary described by the original milestone;
splitting a shape-only runtime sprint from its only proof would add a serial PR
without increasing width.

## Decision prerequisites

Resolve only the runtime API choices needed for builder inputs, router/service
types, lock behavior, health/OpenAPI mounting, shutdown, fixture cleanup, and
typed errors. Do not resolve CLI auto-start, generated command declarations,
or application test architecture in this sprint.

## Dependencies and parallel safety

- `must_follow a-2`: consumes the real config facade if runtime configuration
  loading is retained as a dependency; otherwise records that applications load
  config before calling runtime.
- `must_follow a-3`: consumes endpoint/listener types and binding behavior.
- `must_follow a-4`: consumes envelope/server conversions only where the
  runtime contract truly needs them; avoid a dependency if ordinary Axum/rmcp
  types suffice.

The three parent implementations may run in parallel. A dependency discovered
to be unnecessary is removed rather than preserved for symmetry.

## Deliverables

1. **REQ-RT-0001 / REQ-RT-0003 / REQ-RT-0008** — A small builder accepts
   application-created stores and ordinary routers/services, resolves runtime
   resources, and serves without owning a command or store registry.
2. **REQ-RT-0002** — A retained nonblocking singleton lock protects one daemon
   instance without PID/probe machinery.
3. **REQ-RT-0004 / REQ-RT-0005** — One router/listener composition and graceful
   shutdown use standard Axum/Tokio/rmcp facilities.
4. **REQ-RT-0006** — An in-process `DaemonFixture` uses the production assembly
   path and a fresh temporary instance root; it never launches a daemon process.
5. **REQ-RT-0007** — Typed runtime failures preserve their sources and never
   call `process::exit`.
6. **REQ-RUN-0103 / REQ-RUN-0204 / REQ-RUN-0205** — Rewrite the spike against
   the four crates, keep its server `main.rs` small, and prove one-listener
   REST/OpenAPI/MCP composition.
7. **REQ-RUN-0001 / REQ-RUN-0004 / REQ-RUN-0005** — Create the four-member
   workspace, root core recipes/CI, and runtime boundary manifest without
   registering template packages as workspace members.

## Acceptance criteria

1. `req:REQ-RUN-0204` — the rewritten spike loads configuration outside or at
   the explicit runtime seam, constructs the builder, and reaches the same
   service function through the proved surfaces with a server `main.rs` under
   the source requirement's limit.
2. `req:REQ-RUN-0205` — REST, OpenAPI, health, and optional stateless MCP share
   the intended process/listener without wrappers around ordinary framework
   types.
3. `req:REQ-RT-0006` — two in-process fixtures run concurrently on independent
   temporary roots, stop independently, and clean up their runtime resources.
4. `boundary:sc-runtime` — crate tests, README example, default/server feature
   checks, workspace build, and all four boundary manifests are green.
5. `ADR-RUN-0004` — inspection finds no application command registry, generated
   CLI testing abstraction, service-manager launcher, SQL schema, or
   observability wrapper in the reusable libraries.

## This sprint does not close

Generated application structure, JSON-driven generation, registry publication,
CLI auto-start, process-level daemon tests, attribute-generated tests, and the
wizard remain open.

## Paths to delete

None. a-7 removes the rewritten spike after release evidence is preserved.

## Required validation

Run root lint/test recipes, workspace build, all default/server package
selections, doctests, boundary validation, feature-specific dependency-tree
checks, and the rewritten spike proofs. Host validation uses only in-process
fixtures. Record results in `docs/validation/phase-a-runtime.md`.

