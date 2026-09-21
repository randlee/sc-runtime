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
3. `req:REQ-RUN-0303` — MCP on/off and project-licence fixtures generate the
   expected files; unsupported database choices fail before generation.
4. `req:REQ-RUN-0306` — plain cargo-generate creates a buildable default project
   without Python or Wyvern; Jinja document handling is documented honestly.
5. `req:REQ-RUN-0701` — fixture CI generates, lints, tests, and builds without
   modifying the result between generation and validation.
6. `ADR-RUN-0303` — inspection confirms that reusable runtime behavior remains
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

