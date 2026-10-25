# Anchors - phase-current

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `doc/plan/phase-current/intent.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | locked | Bootstrap database commands the consumer can disable | 2026-10-25 |
| ANC-002 | locked | Copy cache matches the remote schema | 2026-10-25 |
| ANC-003 | locked | Host cache aggregates many hosts | 2026-10-25 |
| ANC-004 | locked | Consumer defines the host rollup | 2026-10-25 |
| ANC-005 | locked | just new generates the adaptor | 2026-10-25 |
| ANC-006 | locked | Tables may be skipped either way | 2026-10-25 |
| ANC-007 | locked | Flags name server-added columns | 2026-10-25 |
| ANC-008 | locked | One implementation per common pattern | 2026-10-25 |
| ANC-009 | locked | Different code per database model | 2026-10-25 |
| ANC-010 | locked | Advanced adaptors are consumer code | 2026-10-25 |
| ANC-011 | locked | The app schedules store sync | 2026-10-25 |
| ANC-012 | locked | sc-cron is an optional crate | 2026-10-25 |
| ANC-013 | locked | sc-cron config is one SQLite table | 2026-10-25 |
| ANC-014 | locked | sc-cron schedules with cron strings | 2026-10-25 |
| ANC-015 | locked | Richer scheduling API is the consumer's | 2026-10-25 |
| ANC-016 | retired | Sync method is Rust or python3 | 2026-10-25 |
| ANC-017 | locked | beads-cache uses SQLite only | 2026-10-25 |
| ANC-018 | locked | beads-cache syncs to Dolt through beads | 2026-10-25 |
| ANC-019 | locked | beads-cache does not use Postgres | 2026-10-25 |
| ANC-020 | locked | Adaptor crate defines the store relationship | 2026-10-25 |
| ANC-021 | locked | A cache forwards data between stores | 2026-10-25 |
| ANC-022 | locked | sc-runtime is crates plus a template | 2026-10-25 |
| ANC-023 | locked | A generated application is ready to run | 2026-10-25 |
| ANC-024 | locked | sc-cron is a phase-b sprint | 2026-10-25 |
| ANC-025 | locked | sc-cron decisions stay in the ADR or reqs | 2026-10-25 |
| ANC-026 | locked | SQLite cron runs a Rust job or python3 | 2026-10-25 |

## ANC-001

Anchor: The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface. An application command calls the same backend operation through its own CLI command, MCP tool, and HTTP route.

> the 'general database commands' would be expected to be disabled by the customer. they are simply a bootstrap tool and could easily be 'replaced' w/ a command that has better ux and does a similar call on the back-end. i.e. delete row <table> <id> would be a general command which could be used 'under the hood' w/ a different cli/mcp/http call.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Bootstrap database commands the consumer can disable

Scope: a bead that edits template CLI, MCP, or HTTP commands

Deterministic check to pair with the development gate: The generated template registers a general delete-row command on the CLI, on MCP, and over HTTP, and a generation flag omits those three registrations while the backend operation remains callable.

Beads: anchor `run-8bp`, plan questions `run-8bp.1`, dev questions `run-8bp.2`

## ANC-002

Anchor: The simple local cache is a copy of the remote Postgres or SQL database with the same schema, for faster or offline access.

> if local cache is a) simply a 'copy' of the remote postgres/sql database with the same schema to allow faster/offline access to data, that is the simple case.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Copy cache matches the remote schema

Scope: a bead that defines the local cache

Deterministic check to pair with the development gate: The generated copy adaptor creates SQLite tables for the shared remote schema.

Beads: anchor `run-72g`, plan questions `run-72g.1`, dev questions `run-72g.2`

## ANC-003

Anchor: The host-specific cache is a cache whose Postgres or SQL database aggregates information from many hosts.

> if the cache is b) host specific and the postgres/sql aggregates info from many hosts

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Host cache aggregates many hosts

Scope: a bead that defines the host-specific cache

Deterministic check to pair with the development gate: The host-aggregate adaptor documents one SQLite database per host and a Postgres aggregate.

Beads: anchor `run-egh`, plan questions `run-egh.1`, dev questions `run-egh.2`

## ANC-004

Anchor: The host-aggregate cache requires logic in the consumer repository to define the aggregate.

> that would be a bit more complicated and require some consumer repo logic to define.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Consumer defines the host rollup

Scope: a bead that defines the host-aggregate cache

Deterministic check to pair with the development gate: The host-aggregate adaptor has a consumer-owned function that maps host rows into the Postgres aggregate.

Beads: anchor `run-s7f`, plan questions `run-s7f.1`, dev questions `run-s7f.2`

## ANC-005

Anchor: Adaptor Rust code and models in a new repository are generated by the generation driver, the command just new (scripts/new_project.py). The library crate named sc-runtime does not generate them.

> sc-runtime could generate a new repo w/ the adaptor rust code and models

Said by randlee, planning session, 2026-10-25. Status: locked. Version 3.

Summary: just new generates the adaptor

Scope: a bead that edits scripts/new_project.py or the template

Deterministic check to pair with the development gate: just new writes the adaptor crate and its models into the new repository. crates/sc-runtime contains no generator.

Beads: anchor `run-n0i`, plan questions `run-n0i.1`, dev questions `run-n0i.2`

## ANC-006

Anchor: Table skipping is allowed for an extra SQLite table and for an extra SQL-server table.

> skipping tables may occur both ways (extra tables may be sqlite, may be sql/postgres server).

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Tables may be skipped either way

Scope: a bead that chooses which tables to sync

Deterministic check to pair with the development gate: Generation input can name a SQLite-only table and a SQL-server-only table.

Beads: anchor `run-goz`, plan questions `run-goz.1`, dev questions `run-goz.2`

## ANC-007

Anchor: Columns added on the SQL or Postgres server are indicated by a consumer-repository flag.

> consumer repo flags indicating what tables to skip, what columns are added for sql/postgres server

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Flags name server-added columns

Scope: a bead that adds generation flags

Deterministic check to pair with the development gate: The generation input names columns that exist on the SQL server and not in SQLite.

Beads: anchor `run-ey3`, plan questions `run-ey3.1`, dev questions `run-ey3.2`

## ANC-008

Anchor: No consumer repository uses a different implementation of one common pattern from the others.

> What I want to avoid is each consuming repo making different choices about how to implement common patterns.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: One implementation per common pattern

Scope: a bead that implements a common pattern in a consumer repository

Deterministic check to pair with the development gate: Two generated repositories that select the same pattern contain the same adaptor implementation.

Beads: anchor `run-00l`, plan questions `run-00l.1`, dev questions `run-00l.2`

## ANC-009

Anchor: Different code for different models is allowed on the consuming repository.

> Supporting multiple models may require dropping different code on the consuming repo.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Different code per database model

Scope: a bead that generates store code for more than one database

Deterministic check to pair with the development gate: Generated SQLite code and generated Postgres code are separate files.

Beads: anchor `run-apv`, plan questions `run-apv.1`, dev questions `run-apv.2`

## ANC-010

Anchor: A more advanced adaptor is written by the consumer from the building blocks sc-runtime provides.

> more advanced senarios would be completely on the consumer to write the adaptor from the building blocks we provide.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Advanced adaptors are consumer code

Scope: a bead that adds a store adaptor

Deterministic check to pair with the development gate: An adaptor that is neither the copy pattern nor the host-aggregate pattern is a file the consumer wrote.

Beads: anchor `run-xau`, plan questions `run-xau.1`, dev questions `run-xau.2`

## ANC-011

Anchor: Store synchronization is scheduled by the consuming application.

> scheduling of sync is on the consuming app

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: The app schedules store sync

Scope: a bead that generates a store adaptor or a sync schedule

Deterministic check to pair with the development gate: The adaptor crate starts no timer; a schedule is registered by application code.

Beads: anchor `run-ebh`, plan questions `run-ebh.1`, dev questions `run-ebh.2`

## ANC-012

Anchor: sc-cron is an optional drop-in crate.

> I would expect sc-cron to be a simple optional drop in

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: sc-cron is an optional crate

Scope: a bead that adds crates/sc-cron

Deterministic check to pair with the development gate: The template builds with sc-cron absent, and a flag adds the crate.

Beads: anchor `run-443`, plan questions `run-443.1`, dev questions `run-443.2`

## ANC-013

Anchor: sc-cron configuration is one SQLite table.

> that needs a sqlite table for configuration

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: sc-cron config is one SQLite table

Scope: a bead that adds the sc-cron configuration table

Deterministic check to pair with the development gate: crates/sc-cron creates one schedule table.

Beads: anchor `run-bmj`, plan questions `run-bmj.1`, dev questions `run-bmj.2`

## ANC-014

Anchor: sc-cron scheduling uses normal cron strings.

> scheduling would be done w/ cron strings

Said by randlee, planning session, 2026-10-25. Status: locked. Version 2.

Summary: sc-cron schedules with cron strings

Scope: a bead that edits sc-cron scheduling

Deterministic check to pair with the development gate: A schedule row stores a cron string and the parser accepts it.

Beads: anchor `run-8dy`, plan questions `run-8dy.1`, dev questions `run-8dy.2`

## ANC-015

Anchor: A more sophisticated scheduling API is created by the consumer.

> consumer can create more sophisticated api if they desire.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Richer scheduling API is the consumer's

Scope: a bead that adds a scheduling API

Deterministic check to pair with the development gate: crates/sc-cron has no HTTP route, MCP tool, or binary; a consumer project may add one.

Beads: anchor `run-ezk`, plan questions `run-ezk.1`, dev questions `run-ezk.2`

## ANC-016

Anchor: A cache-sync method is either a Rust function or a Python program run as python3 <module> cron.tick or cron.stop with one JSON object on stdin.

> A cache sync adaptor would expose cron sync methods which could be rust code, or could be python hook (this is what atm-core is doing today w/ atm-cron).

Said by randlee, planning session, 2026-10-25. Status: retired. Version 3.

Summary: Sync method is Rust or python3

Scope: a bead that adds a cache-sync method

Deterministic check to pair with the development gate: A schedule row names a Rust function, or a Python module invoked as python3 <module> cron.tick or cron.stop with one JSON object on stdin.

Beads: anchor `run-50w`, plan questions `run-50w.1`, dev questions `run-50w.2`

## ANC-017

Anchor: beads-cache uses SQLite only.

> beads-cache would be a sqlite ONLY design

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: beads-cache uses SQLite only

Scope: a bead that wires beads-cache

Deterministic check to pair with the development gate: The beads-cache crate depends on SQLite and has no Postgres driver.

Beads: anchor `run-854`, plan questions `run-854.1`, dev questions `run-854.2`

## ANC-018

Anchor: beads-cache synchronization to Dolt is through the beads CLI or API.

> which syncronizes w/ dolt database through beads cli/api.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: beads-cache syncs to Dolt through beads

Scope: a bead that wires beads-cache

Deterministic check to pair with the development gate: The beads-cache sync job calls the beads CLI or API.

Beads: anchor `run-rgm`, plan questions `run-rgm.1`, dev questions `run-rgm.2`

## ANC-019

Anchor: beads-cache use of sc-runtime does not include SQL or Postgres.

> it's use of sc-runtime would not include SQL/postgres which are out of scope (beads owns beads/dolt database).

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: beads-cache does not use Postgres

Scope: a bead that wires beads-cache

Deterministic check to pair with the development gate: beads-cache Cargo.toml has no Postgres driver and no store-postgres dependency.

Beads: anchor `run-0ms`, plan questions `run-0ms.1`, dev questions `run-0ms.2`

## ANC-020

Anchor: The store relationship is defined in an adaptor crate on the consumer repository.

> there needs to be a place (i.e. schema, adaptor crate) on consumer repo that defines the relationship

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: Adaptor crate defines the store relationship

Scope: a bead that adds an adaptor crate

Deterministic check to pair with the development gate: The generated consumer repository contains an adaptor crate that names the SQLite and SQL-server relationship.

Beads: anchor `run-awh`, plan questions `run-awh.1`, dev questions `run-awh.2`

## ANC-021

Anchor: A cache is forwarding of data between stores.

> caching would be forwarding between stores.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: A cache forwards data between stores

Scope: a bead that adds a cache between stores

Deterministic check to pair with the development gate: The consumer adaptor copies rows from one store to the other.

Beads: anchor `run-82j`, plan questions `run-82j.1`, dev questions `run-82j.2`

## ANC-022

Anchor: sc-runtime is a set of crates and a templating system.

> sc-runtime is a set of crates and a templating system for rapidly deploying new ready to run applications.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: sc-runtime is crates plus a template

Scope: every plan bead

Deterministic check to pair with the development gate: The repository contains crates/ and template/.

Beads: anchor `run-10g`, plan questions `run-10g.1`, dev questions `run-10g.2`

## ANC-023

Anchor: An application deployed from sc-runtime is ready to run.

> for rapidly deploying new ready to run applications.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: A generated application is ready to run

Scope: a bead that changes generation or the template

Deterministic check to pair with the development gate: just new produces a project that passes just test with no file edited.

Beads: anchor `run-v0f`, plan questions `run-v0f.1`, dev questions `run-v0f.2`

## ANC-024

Anchor: sc-cron is ported as part of the phase-b sprints.

> we should probably port sc-cron as part of phase-b sprints

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: sc-cron is a phase-b sprint

Scope: the phase-b plan

Deterministic check to pair with the development gate: docs/plans/phase-b contains a sprint whose deliverable is crates/sc-cron.

Beads: anchor `run-q75`, plan questions `run-q75.1`, dev questions `run-q75.2`

## ANC-025

Anchor: The sc-cron discussion is a permanent record in the ADR or requirements documents.

> so any discussion becomes perment record in the adr/req documents.

Said by randlee, planning session, 2026-10-25. Status: locked. Version 1.

Summary: sc-cron decisions stay in the ADR or reqs

Scope: a bead that edits docs/architecture.md or docs/requirements.md

Deterministic check to pair with the development gate: docs/architecture.md or docs/requirements.md states the sc-cron contract.

Beads: anchor `run-af2`, plan questions `run-af2.1`, dev questions `run-af2.2`

## ANC-026

Anchor: sc-cron is a SQLite scheduler. The caller starts a schedule, stops it, and runs one worker. A due row runs as an in-process Rust job with run and should_stop, or as python3 <module> cron.tick or cron.stop with one JSON object on stdin.

> A cache sync adaptor would expose cron sync methods which could be rust code, or could be python hook

Said by randlee, planning session, 2026-10-25. Status: locked. Version 2.

Summary: SQLite cron runs a Rust job or python3

Scope: a bead that adds or edits crates/sc-cron

Deterministic check to pair with the development gate: crates/sc-cron exposes start_async, stop_async, spawn_worker, Job::run, and Job::should_stop, and can run python3 <module> cron.tick or cron.stop with JSON on stdin.

Beads: anchor `run-oez`, plan questions `run-oez.1`, dev questions `run-oez.2`
