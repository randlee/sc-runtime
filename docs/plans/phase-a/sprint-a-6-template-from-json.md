---
id: a-6
phase: a
status: planned
closure_type: integration
target_boundary: template-from-json
vertical_rationale: "The generator and editable example do not exist yet; one thin vertical sprint creates the single JSON-to-project artifact and proves it without introducing a reusable application framework."
track: generation
wave: 3
branch: sprint/a-6-template-from-json
worktree: sprint/a-6-template-from-json
recommended_agent: solar
recommended_model: sol
must_follow: [a-5]
parallel_safe: []
requirements:
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0201
  - REQ-RUN-0202
  - REQ-RUN-0203
  - REQ-RUN-0301
  - REQ-RUN-0302
  - REQ-RUN-0303
  - REQ-RUN-0304
  - REQ-RUN-0305
  - REQ-RUN-0306
  - REQ-RUN-0307
  - REQ-RUN-0308
  - REQ-RUN-0309
  - REQ-RUN-0310
  - REQ-RUN-0401
  - REQ-RUN-0402
  - REQ-RUN-0403
  - REQ-RUN-0501
  - REQ-RUN-0502
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RUN-0001
  - NFR-RUN-0002
  - NFR-RUN-0003
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0006
  - NFR-RUN-0007
  - NFR-RUN-0008
  - NFR-RUN-0009
  - NFR-RUN-0010
adrs:
  - ADR-RUN-0001
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0004
  - ADR-RUN-0005
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0009
  - ADR-RUN-0201
  - ADR-RUN-0202
  - ADR-RUN-0301
  - ADR-RUN-0302
  - ADR-RUN-0303
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - "template/**"
  - "wizard/answers.schema.json"
  - "wizard/fixtures/**"
  - "tests/fixtures/refused/**"
  - "scripts/new_project.py"
  - "justfile"
  - "tests/unit/generation/**"
  - "tests/integration/generation/**"
  - ".github/workflows/template.yml"
  - "docs/generation.md"
  - "docs/validation/phase-a-generation.md"
---

# Sprint a-6 — template from JSON

## Outcome and closure

Generate one minimal, editable SQLite application from a validated answers JSON
file. The generated project proves the four libraries are useful together; it
is not a framework whose internal organization is imposed on future projects.

## Decision prerequisites

Reconcile the deferred auto-start and prescribed CLI-test requirements before
dispatch. Settle only the answers keys/defaults, library version requirements,
sc-compose invocation, destination/name mapping, and minimal example operation
needed for this milestone.

## Dependencies and parallel safety

- `must_follow a-5`: consumes the real published-surface candidates and the
  rewritten spike's ordinary composition pattern.

No separate driver, template-tree, or phase-wide integration sprint is
created: those slices would serialize one small greenfield artifact and defer
its only meaningful proof.

## Deliverables

1. **REQ-RUN-0401 / REQ-RUN-0402 / REQ-RUN-0403** — One versioned schema and
   small valid/refused fixture set; invalid or unsupported input fails before
   destination creation.
2. **REQ-RUN-0501 / REQ-RUN-0502** — A small `--var-file` driver sequences
   validation, cargo-generate, sc-compose, and the generated project's own
   lint/test commands. It does not include wizard logic yet.
   In development/PR mode the driver writes a generated-root
   `[patch.crates-io]` table pointing at the four crates in the checkout before
   its first Cargo invocation; release mode writes no patch and requires
   registry sources. The template always retains version requirements.
3. **REQ-RUN-0301 / REQ-RUN-0302** — An editable five-crate example with one
   create/get operation and thin REST, optional MCP, and CLI transport adapters.
4. **REQ-RUN-0303 through REQ-RUN-0310** — Minimal conditional generation,
   SQLite store/config/observability wiring, agent-document rendering, and
   standard generated recipes needed for the example to compile and run.
   Concrete example choices are documented as examples, not extension rules.
5. **REQ-RUN-0201 / REQ-RUN-0203** — One host-safe in-process smoke proves the
   example service through REST, optional MCP, and the CLI's transport path.
6. **REQ-RUN-0701** — Fixture CI discovers supported answers files and verifies
   no-edit generation, lint, test, and build.
7. **REQ-RUN-0306** — Plain cargo-generate remains usable for the template tree
   with documented limitations around separately rendered Jinja documents.

## Acceptance criteria

1. `req:REQ-RUN-0501` — `just new <dest> --var-file <fixture>` generates into
   an absent destination and stops at the first named failing standard tool.
2. `req:REQ-RUN-0201` — the generated example creates and gets one widget via
   the shared service path from REST, optional MCP, and a real transport client.
   This proves connectivity, not permanent application architecture.
3. `req:REQ-RUN-0202` — report-only `--json` against an unreachable endpoint
   returns a nonzero status and the exact `DAEMON.NOT_RUNNING` envelope without
   creating daemon, lock, socket, or database artifacts. The real CLI-binary
   proof runs only on an isolated CI runner; a host-safe proof exercises the
   same reporting path without prescribing helper functions.
4. `req:REQ-RUN-0203` — success and operation-failure envelopes are equal as
   JSON across REST, MCP and CLI `--json`; unknown `data` and `error.details`
   content and the received version survive CLI output unchanged.
5. `req:REQ-RUN-0301` / `req:REQ-RUN-0302` — generation produces the minimal
   editable api-types/store-sqlite/service/daemon/cli example, and the create/get
   operation reaches one service path through all enabled surfaces.
6. `req:REQ-RUN-0310` — CLI and daemon accept the same endpoint text and use
   only `sc-transport` resolution; an override points both at the same fixture
   endpoint and no template code computes its own default.
7. `req:REQ-RUN-0303` — MCP on/off and project-licence fixtures generate the
   expected files; unsupported database choices fail before generation.
8. `req:REQ-RUN-0306` — plain cargo-generate creates the default project
   without Python or Wyvern. Before publication, its separate CI smoke applies
   the same checkout patch immediately after generation and before the first
   Cargo command; after publication it uses registry mode. Jinja document
   handling is documented honestly.
9. `req:REQ-RUN-0701` — fixture CI generates, lints, tests, and builds without
   modifying the result between generation and validation.
   PR-mode lockfiles resolve all four libraries to the checkout paths; release
   lockfiles resolve all four to the crates.io registry.
10. `req:NFR-RUN-0001` — `cargo build -p cli` and its normal dependency tree
    contain none of Axum, rmcp, sqlx, daemon, service, store, or sc-runtime;
    the a-1 measurement decides and documents the separate workspace-wide
    feature-unification policy.
11. `ADR-RUN-0303` — inspection confirms that reusable runtime behavior remains
   in the published crates while routes, schema, queries, and application
   composition remain editable template code.

## This sprint does not close

It does not implement CLI auto-start, service-manager definitions, prescribed
CLI helper/test shapes, attribute macros, attribute-generated tests, process
daemon tests, Postgres, or the wizard. It does not claim the example's internal
file/crate organization is mandatory for generated applications.

## Paths to delete

Generated test destinations and temporary values files are removed by their
tests. No repository source path is deleted.

## Required validation

Run root lint/test, all fixture generations, generated workspace build/test,
plain cargo-generate smoke, schema/refused-input tests, and static checks for
unrendered Jinja/Liquid. No daemon child process runs on the host. Record the
supported variants and results in `docs/validation/phase-a-generation.md`.
