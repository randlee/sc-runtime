# Intent - phase-current

Written 2026-10-25 by randlee. This page frames the anchors. Jev judges the anchors, not this page.

## What this plan is for

sc-runtime is a set of crates and a template for deploying a new application that can use its database immediately. General database commands are the bootstrap. The consumer disables that public surface and adds application commands that call the same backend operations.

The storage plan is ADR-RUN-0305. SQLite and Postgres meet in an adaptor crate that `just new` generates into the consumer repository. That plan stays a requirement when a sprint list changes.

## What success looks like

- A generated project exposes general database commands on the CLI, on MCP, and over HTTP, including `delete row <table> <id>`. The consumer can turn that surface off, and an application command calls the same backend operation.
- A new application is created with no database, SQLite only, one sqlx SQL engine only, or SQLite plus one sqlx SQL engine. Postgres and MySQL are the SQL engines sqlx 0.9 ships.
- The copy cache and the host-aggregate cache are both generated when the application has SQLite and a SQL engine. The consumer defines the host rollup. Tables can be skipped on either side.
- Paired stores use the same schema or a similar schema, and one shared trait or a different trait per store. Each engine's SQL stays in that engine's crate.
- `sc-cron` is an optional crate with one SQLite table and normal cron strings. A sync method is Rust or a Python hook. The consuming application schedules the sync.

## What is deliberately out of scope

- A command registry inside the four library crates.
- Store-to-store cache sync inside the sc-runtime crates. That forwarding lives in the consumer adaptor.

## Anchors

The judged anchors are the REQ and ADR ids in the register. `anchors.json` records those ids. An ANC id is only a retired pointer at one of them.
