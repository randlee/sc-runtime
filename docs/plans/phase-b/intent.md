# Intent - phase-b

Written 2026-10-25 by randlee. This page frames the anchors. Jev judges the anchors, not this page.

## What this plan is for

sc-runtime is a set of crates and a template for deploying a new application that can use its database immediately. General database commands are the bootstrap. The consumer disables that public surface and adds application commands that call the same backend operations.

The storage plan is ADR-RUN-0305. SQLite and Postgres meet in an adaptor crate that `just new` generates into the consumer repository. That plan stays a requirement when a sprint list changes.

## What success looks like

- A generated project exposes general database commands on the CLI, on MCP, and over HTTP, including `delete row <table> <id>`. The consumer can turn that surface off, and an application command calls the same backend operation.
- A new application is created with no database, SQLite only, one sqlx SQL engine only, or SQLite plus one sqlx SQL engine. Postgres and MySQL are the SQL engines sqlx 0.9 ships.
- The copy cache and the host-aggregate cache are both generated when the application has SQLite and a SQL engine. The consumer defines the host rollup. Tables can be skipped on either side.
- Paired stores use the same schema or a similar schema, and one shared trait or a different trait per store. Each engine's SQL stays in that engine's crate.
- `sc-cron` is an optional general-purpose crate with one SQLite table and normal cron strings. Application jobs may use Rust, Python, or application-supplied Monty source. The consuming application schedules the work.

## Phase B continuation intent approved 2026-10-29

Applications can extend Monty host calls without changing sc-cron. Phase B
provides sc_compose_run and atm_run through that shared interface, supporting
any command/subcommand of their external executables. Neither tool's Rust
crates enter any production release crate; boundary lint enforces that rule,
with test and benchmark references permitted. render_template is not a product
requirement. MVP has no script signing or separate approval mechanism.

Python and Monty share explicit environment configuration. Daemon inheritance
is allowed by name; schedules can set or clear values, use application-resolved
references, and load absolute dotenv files. Schedule writes never implicitly
capture ambient values. Deliberate plaintext set values are valid registry
data. Monty can read the configured environment during source initialization.
The first release establishes one schedule-schema baseline.

## What is deliberately out of scope

- A command registry inside the four library crates.
- Store-to-store cache sync inside the sc-runtime crates. That forwarding lives in the consumer adaptor.

## Anchors

The judged anchors include approved ANC statements alongside REQ and ADR ids.
ANC-027 through ANC-042 preserve the approved Phase B continuation intent.
Earlier retired ANC records remain provenance; existing REQ/ADR records retain
their own version history and must be reconciled explicitly with amended contracts.
