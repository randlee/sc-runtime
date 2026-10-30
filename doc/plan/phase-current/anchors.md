# Anchors - phase-current

Constraints captured during planning. Each anchor is a testable statement written by the orchestrator and approved by the user; the user's verbatim words are kept beneath it as provenance. Every anchor is judged at the planning gate and again at the development gate. Edit `anchors.json`, not this file; this file is rendered.

Intent: `doc/plan/phase-current/intent.md`

| ID | Status | Summary | Stated |
| --- | --- | --- | --- |
| ANC-001 | retired | Bootstrap database commands the consumer can disable | 2026-10-25 |
| ANC-002 | retired | Copy cache matches the remote schema | 2026-10-25 |
| ANC-003 | retired | Host cache aggregates many hosts | 2026-10-25 |
| ANC-004 | retired | Consumer defines the host rollup | 2026-10-25 |
| ANC-005 | retired | just new generates the adaptor | 2026-10-25 |
| ANC-006 | retired | Tables may be skipped either way | 2026-10-25 |
| ANC-007 | retired | Flags name server-added columns | 2026-10-25 |
| ANC-008 | retired | One implementation per common pattern | 2026-10-25 |
| ANC-009 | retired | Different code per database model | 2026-10-25 |
| ANC-010 | retired | Advanced adaptors are consumer code | 2026-10-25 |
| ANC-011 | retired | The app schedules store sync | 2026-10-25 |
| ANC-012 | retired | sc-cron is an optional crate | 2026-10-25 |
| ANC-013 | retired | sc-cron config is one SQLite table | 2026-10-25 |
| ANC-014 | retired | sc-cron schedules with cron strings | 2026-10-25 |
| ANC-015 | retired | Richer scheduling API is the consumer's | 2026-10-25 |
| ANC-016 | retired | Sync method is Rust or python3 | 2026-10-25 |
| ANC-017 | retired | beads-cache uses SQLite only | 2026-10-25 |
| ANC-018 | retired | beads-cache syncs to Dolt through beads | 2026-10-25 |
| ANC-019 | retired | beads-cache does not use Postgres | 2026-10-25 |
| ANC-020 | retired | Adaptor crate defines the store relationship | 2026-10-25 |
| ANC-021 | retired | A cache forwards data between stores | 2026-10-25 |
| ANC-022 | retired | sc-runtime is crates plus a template | 2026-10-25 |
| ANC-023 | retired | A generated application is ready to run | 2026-10-25 |
| ANC-024 | retired | sc-cron is a phase-b sprint | 2026-10-25 |
| ANC-025 | retired | sc-cron decisions stay in the ADR or reqs | 2026-10-25 |
| ANC-026 | retired | SQLite cron runs a Rust job or python3 | 2026-10-25 |
| REQ-CRON-0001 | locked | Optional reusable cron dependency | 2026-10-25 |
| REQ-CRON-0002 | locked | Normal five-field crontab | 2026-10-25 |
| REQ-CRON-0003 | locked | Application-defined jobs and extensible Monty hosts | 2026-10-25 |
| REQ-CRON-0004 | locked | One schedule table | 2026-10-25 |
| REQ-CRON-0005 | locked | One CronError enum | 2026-10-25 |
| ADR-RUN-0013 | locked | General-purpose cron and application host capabilities | 2026-10-25 |
| ADR-RUN-0305 | locked | Generated consumer store adaptor | 2026-10-25 |
| REQ-RUN-0313 | locked | Adaptor crate defines the stores | 2026-10-25 |
| REQ-RUN-0314 | locked | Copy adaptor matches the remote schema | 2026-10-25 |
| REQ-RUN-0315 | locked | Host aggregate collects many hosts | 2026-10-25 |
| REQ-RUN-0316 | locked | Consumer defines the host rollup | 2026-10-25 |
| REQ-RUN-0317 | locked | just new generates the adaptor | 2026-10-25 |
| REQ-RUN-0318 | locked | Library crate generates nothing | 2026-10-25 |
| REQ-RUN-0319 | locked | Flags name one-sided tables and columns | 2026-10-25 |
| REQ-RUN-0320 | locked | One implementation per common pattern | 2026-10-25 |
| REQ-RUN-0321 | locked | SQL stays in that engine's crate | 2026-10-25 |
| REQ-RUN-0322 | locked | Advanced adaptor is consumer code | 2026-10-25 |
| REQ-RUN-0324 | locked | Adaptor moves rows between stores | 2026-10-25 |
| REQ-RUN-0325 | locked | Framework crates move no rows | 2026-10-25 |
| REQ-RUN-0323 | locked | The application schedules sync | 2026-10-25 |
| REQ-RUN-0326 | locked | Database layouts of a new application | 2026-10-25 |
| REQ-RUN-0327 | locked | Same schema or a similar schema | 2026-10-25 |
| REQ-RUN-0328 | locked | Shared trait or a trait per store | 2026-10-25 |
| REQ-RUN-0329 | locked | Package name carries the project name | 2026-10-25 |
| REQ-RUN-0330 | locked | Row structs are in the schema crate | 2026-10-25 |
| REQ-RUN-0331 | locked | Disableable database commands | 2026-10-25 |
| ADR-RUN-0002 | locked | Template plus published crates | 2026-10-25 |
| REQ-RUN-0503 | locked | Generated project is green | 2026-10-25 |
| ANC-027 | locked | General-purpose scheduler | 2026-10-29 |
| ANC-028 | locked | Application host extensions | 2026-10-29 |
| ANC-029 | locked | Compose command coverage | 2026-10-29 |
| ANC-030 | locked | ATM host integration | 2026-10-29 |
| ANC-031 | locked | Production tool crate boundary | 2026-10-29 |
| ANC-032 | locked | No mandatory render helper | 2026-10-29 |
| ANC-033 | locked | No MVP signing gate | 2026-10-29 |
| ANC-034 | locked | Shared hook environment | 2026-10-29 |
| ANC-035 | locked | Explicit daemon inheritance | 2026-10-29 |
| ANC-036 | locked | Schedule environment overrides | 2026-10-29 |
| ANC-037 | locked | No implicit environment capture | 2026-10-29 |
| ANC-038 | locked | Explicit plaintext values | 2026-10-29 |
| ANC-039 | locked | Application secret references | 2026-10-29 |
| ANC-040 | locked | Absolute dotenv sources | 2026-10-29 |
| ANC-041 | locked | Environment before initialization | 2026-10-29 |
| ANC-042 | locked | First release schema baseline | 2026-10-29 |

## ANC-001

Anchor: The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface. An application command calls the same backend operation through its own CLI command, MCP tool, and HTTP route.

> the 'general database commands' would be expected to be disabled by the customer. they are simply a bootstrap tool and could easily be 'replaced' w/ a command that has better ux and does a similar call on the back-end. i.e. delete row <table> <id> would be a general command which could be used 'under the hood' w/ a different cli/mcp/http call.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Bootstrap database commands the consumer can disable

Scope: a bead that edits template CLI, MCP, or HTTP commands

Deterministic check to pair with the development gate: The generated template registers a general delete-row command on the CLI, on MCP, and over HTTP, and a generation flag omits those three registrations while the backend operation remains callable.

Beads: anchor `run-8bp`, plan questions `run-8bp.1`, dev questions `run-8bp.2`

## ANC-002

Anchor: The simple local cache is a copy of the remote Postgres or SQL database with the same schema, for faster or offline access.

> if local cache is a) simply a 'copy' of the remote postgres/sql database with the same schema to allow faster/offline access to data, that is the simple case.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Copy cache matches the remote schema

Scope: a bead that defines the local cache

Deterministic check to pair with the development gate: The generated copy adaptor creates SQLite tables for the shared remote schema.

Beads: anchor `run-72g`, plan questions `run-72g.1`, dev questions `run-72g.2`

## ANC-003

Anchor: The host-specific cache is a cache whose Postgres or SQL database aggregates information from many hosts.

> if the cache is b) host specific and the postgres/sql aggregates info from many hosts

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Host cache aggregates many hosts

Scope: a bead that defines the host-specific cache

Deterministic check to pair with the development gate: The host-aggregate adaptor documents one SQLite database per host and a Postgres aggregate.

Beads: anchor `run-egh`, plan questions `run-egh.1`, dev questions `run-egh.2`

## ANC-004

Anchor: The host-aggregate cache requires logic in the consumer repository to define the aggregate.

> that would be a bit more complicated and require some consumer repo logic to define.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Consumer defines the host rollup

Scope: a bead that defines the host-aggregate cache

Deterministic check to pair with the development gate: The host-aggregate adaptor has a consumer-owned function that maps host rows into the Postgres aggregate.

Beads: anchor `run-s7f`, plan questions `run-s7f.1`, dev questions `run-s7f.2`

## ANC-005

Anchor: Adaptor Rust code and models in a new repository are generated by the generation driver, the command just new (scripts/new_project.py). The library crate named sc-runtime does not generate them.

> sc-runtime could generate a new repo w/ the adaptor rust code and models

Said by randlee, planning session, 2026-10-25. Status: retired. Version 4.

Summary: just new generates the adaptor

Scope: a bead that edits scripts/new_project.py or the template

Deterministic check to pair with the development gate: just new writes the adaptor crate and its models into the new repository. crates/sc-runtime contains no generator.

Beads: anchor `run-n0i`, plan questions `run-n0i.1`, dev questions `run-n0i.2`

## ANC-006

Anchor: Table skipping is allowed for an extra SQLite table and for an extra SQL-server table.

> skipping tables may occur both ways (extra tables may be sqlite, may be sql/postgres server).

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Tables may be skipped either way

Scope: a bead that chooses which tables to sync

Deterministic check to pair with the development gate: Generation input can name a SQLite-only table and a SQL-server-only table.

Beads: anchor `run-goz`, plan questions `run-goz.1`, dev questions `run-goz.2`

## ANC-007

Anchor: Columns added on the SQL or Postgres server are indicated by a consumer-repository flag.

> consumer repo flags indicating what tables to skip, what columns are added for sql/postgres server

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Flags name server-added columns

Scope: a bead that adds generation flags

Deterministic check to pair with the development gate: The generation input names columns that exist on the SQL server and not in SQLite.

Beads: anchor `run-ey3`, plan questions `run-ey3.1`, dev questions `run-ey3.2`

## ANC-008

Anchor: No consumer repository uses a different implementation of one common pattern from the others.

> What I want to avoid is each consuming repo making different choices about how to implement common patterns.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: One implementation per common pattern

Scope: a bead that implements a common pattern in a consumer repository

Deterministic check to pair with the development gate: Two generated repositories that select the same pattern contain the same adaptor implementation.

Beads: anchor `run-00l`, plan questions `run-00l.1`, dev questions `run-00l.2`

## ANC-009

Anchor: Different code for different models is allowed on the consuming repository.

> Supporting multiple models may require dropping different code on the consuming repo.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Different code per database model

Scope: a bead that generates store code for more than one database

Deterministic check to pair with the development gate: Generated SQLite code and generated Postgres code are separate files.

Beads: anchor `run-apv`, plan questions `run-apv.1`, dev questions `run-apv.2`

## ANC-010

Anchor: A more advanced adaptor is written by the consumer from the building blocks sc-runtime provides.

> more advanced senarios would be completely on the consumer to write the adaptor from the building blocks we provide.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Advanced adaptors are consumer code

Scope: a bead that adds a store adaptor

Deterministic check to pair with the development gate: An adaptor that is neither the copy pattern nor the host-aggregate pattern is a file the consumer wrote.

Beads: anchor `run-xau`, plan questions `run-xau.1`, dev questions `run-xau.2`

## ANC-011

Anchor: Store synchronization is scheduled by the consuming application.

> scheduling of sync is on the consuming app

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: The app schedules store sync

Scope: a bead that generates a store adaptor or a sync schedule

Deterministic check to pair with the development gate: The adaptor crate starts no timer; a schedule is registered by application code.

Beads: anchor `run-ebh`, plan questions `run-ebh.1`, dev questions `run-ebh.2`

## ANC-012

Anchor: sc-cron is an optional drop-in crate.

> I would expect sc-cron to be a simple optional drop in

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: sc-cron is an optional crate

Scope: a bead that adds crates/sc-cron

Deterministic check to pair with the development gate: The template builds with sc-cron absent, and a flag adds the crate.

Beads: anchor `run-443`, plan questions `run-443.1`, dev questions `run-443.2`

## ANC-013

Anchor: sc-cron configuration is one SQLite table.

> that needs a sqlite table for configuration

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: sc-cron config is one SQLite table

Scope: a bead that adds the sc-cron configuration table

Deterministic check to pair with the development gate: crates/sc-cron creates one schedule table.

Beads: anchor `run-bmj`, plan questions `run-bmj.1`, dev questions `run-bmj.2`

## ANC-014

Anchor: sc-cron scheduling uses normal cron strings.

> scheduling would be done w/ cron strings

Said by randlee, planning session, 2026-10-25. Status: retired. Version 3.

Summary: sc-cron schedules with cron strings

Scope: a bead that edits sc-cron scheduling

Deterministic check to pair with the development gate: A schedule row stores a cron string and the parser accepts it.

Beads: anchor `run-8dy`, plan questions `run-8dy.1`, dev questions `run-8dy.2`

## ANC-015

Anchor: A more sophisticated scheduling API is created by the consumer.

> consumer can create more sophisticated api if they desire.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

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

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: beads-cache uses SQLite only

Scope: a bead that wires beads-cache

Deterministic check to pair with the development gate: The beads-cache crate depends on SQLite and has no Postgres driver.

Beads: anchor `run-854`, plan questions `run-854.1`, dev questions `run-854.2`

## ANC-018

Anchor: beads-cache synchronization to Dolt is through the beads CLI or API.

> which syncronizes w/ dolt database through beads cli/api.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: beads-cache syncs to Dolt through beads

Scope: a bead that wires beads-cache

Deterministic check to pair with the development gate: The beads-cache sync job calls the beads CLI or API.

Beads: anchor `run-rgm`, plan questions `run-rgm.1`, dev questions `run-rgm.2`

## ANC-019

Anchor: beads-cache use of sc-runtime does not include SQL or Postgres.

> it's use of sc-runtime would not include SQL/postgres which are out of scope (beads owns beads/dolt database).

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: beads-cache does not use Postgres

Scope: a bead that wires beads-cache

Deterministic check to pair with the development gate: beads-cache Cargo.toml has no Postgres driver and no store-postgres dependency.

Beads: anchor `run-0ms`, plan questions `run-0ms.1`, dev questions `run-0ms.2`

## ANC-020

Anchor: The store relationship is defined in an adaptor crate on the consumer repository.

> there needs to be a place (i.e. schema, adaptor crate) on consumer repo that defines the relationship

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: Adaptor crate defines the store relationship

Scope: a bead that adds an adaptor crate

Deterministic check to pair with the development gate: The generated consumer repository contains an adaptor crate that names the SQLite and SQL-server relationship.

Beads: anchor `run-awh`, plan questions `run-awh.1`, dev questions `run-awh.2`

## ANC-021

Anchor: A cache is forwarding of data between stores.

> caching would be forwarding between stores.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: A cache forwards data between stores

Scope: a bead that adds a cache between stores

Deterministic check to pair with the development gate: The consumer adaptor copies rows from one store to the other.

Beads: anchor `run-82j`, plan questions `run-82j.1`, dev questions `run-82j.2`

## ANC-022

Anchor: sc-runtime is a set of crates and a templating system.

> sc-runtime is a set of crates and a templating system for rapidly deploying new ready to run applications.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: sc-runtime is crates plus a template

Scope: every plan bead

Deterministic check to pair with the development gate: The repository contains crates/ and template/.

Beads: anchor `run-10g`, plan questions `run-10g.1`, dev questions `run-10g.2`

## ANC-023

Anchor: An application deployed from sc-runtime is ready to run.

> for rapidly deploying new ready to run applications.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: A generated application is ready to run

Scope: a bead that changes generation or the template

Deterministic check to pair with the development gate: just new produces a project that passes just test with no file edited.

Beads: anchor `run-v0f`, plan questions `run-v0f.1`, dev questions `run-v0f.2`

## ANC-024

Anchor: sc-cron is ported as part of the phase-b sprints.

> we should probably port sc-cron as part of phase-b sprints

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: sc-cron is a phase-b sprint

Scope: the phase-b plan

Deterministic check to pair with the development gate: docs/plans/phase-b contains a sprint whose deliverable is crates/sc-cron.

Beads: anchor `run-q75`, plan questions `run-q75.1`, dev questions `run-q75.2`

## ANC-025

Anchor: The sc-cron discussion is a permanent record in the ADR or requirements documents.

> so any discussion becomes perment record in the adr/req documents.

Said by randlee, planning session, 2026-10-25. Status: retired. Version 2.

Summary: sc-cron decisions stay in the ADR or reqs

Scope: a bead that edits docs/architecture.md or docs/requirements.md

Deterministic check to pair with the development gate: docs/architecture.md or docs/requirements.md states the sc-cron contract.

Beads: anchor `run-af2`, plan questions `run-af2.1`, dev questions `run-af2.2`

## ANC-026

Anchor: sc-cron is a SQLite scheduler. The caller starts a schedule, stops it, and runs one worker. A due row runs as an in-process Rust job with run and should_stop, or as python3 <module> cron.tick or cron.stop with one JSON object on stdin.

> A cache sync adaptor would expose cron sync methods which could be rust code, or could be python hook

Said by randlee, planning session, 2026-10-25. Status: retired. Version 3.

Summary: SQLite cron runs a Rust job or python3

Scope: a bead that adds or edits crates/sc-cron

Deterministic check to pair with the development gate: crates/sc-cron exposes start_async, stop_async, spawn_worker, Job::run, and Job::should_stop, and can run python3 <module> cron.tick or cron.stop with JSON on stdin.

Beads: anchor `run-oez`, plan questions `run-oez.1`, dev questions `run-oez.2`

## REQ-CRON-0001

Anchor: sc-cron must remain an optional workspace crate; the generated daemon may enable it through cron/monty features while sc-runtime remains independent of sc-cron.

> crates/sc-cron is a workspace member. It depends on no other workspace crate. The template does not name it. The sc-runtime library crate does not name it.

Said by randlee, Historical register quote retained as immutable provenance. Current statement follows the amended REQ-CRON-0001 authority and owner-approved Phase B continuation ANC-027 through ANC-042 (2026-10-29); the historical quote is not the current normative contract., 2026-10-25. Status: locked. Version 5.

Summary: Optional reusable cron dependency

Scope: a bead that edits Cargo.toml or crates/sc-cron

Deterministic check to pair with the development gate: Cargo manifests keep sc-cron optional in generated daemons and absent from sc-runtime dependencies.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-4x5`, plan questions `run-4x5.1`, dev questions `run-4x5.2`

Fixtures:
- pass: The generated daemon optionally enables sc-cron through cron/monty features, and sc-runtime remains independent.
- fail: Every generated application must link sc-cron.

## REQ-CRON-0002

Anchor: A sc-cron schedule is a normal crontab string of five fields: minute, hour, day of month, month, and day of week.

> CronExpr::parse of */5 * * * * succeeds and means every fifth minute. CronExpr::parse of 0 */5 * * * * succeeds and means second 0 of every fifth minute. CronExpr::parse of @hourly succeeds.

Said by randlee, requirements register, 2026-10-25. Status: locked. Version 3.

Summary: Normal five-field crontab

Scope: a bead that edits the sc-cron expression parser

Deterministic check to pair with the development gate: A test expects parsing */5 * * * * to mean every fifth minute.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-8n1`, plan questions `run-8n1.1`, dev questions `run-8n1.2`

Fixtures:
- pass: A sc-cron schedule is a normal crontab string of five fields: minute, hour, day of month, month, and day of week.
- fail: A sc-cron schedule is a six-field string with seconds first, and a five-field crontab is invalid.

## REQ-CRON-0003

Anchor: sc-cron must run application-defined Rust, Python, and optional Monty jobs; applications can extend Monty host calls, including sc_compose_run and atm_run CLI adapters, without changing sc-cron.

> A schedule row names either an in-process Rust job or a Python module. For Python, the crate runs python3 <module> cron.tick or python3 <module> cron.stop and writes one JSON object to stdin. The crate exposes no HTTP route, MCP tool, or binary.

Said by randlee, Historical register quote retained as immutable provenance. Current statement follows the amended REQ-CRON-0003 authority and owner-approved Phase B continuation ANC-027 through ANC-042 (2026-10-29); the historical quote is not the current normative contract., 2026-10-25. Status: locked. Version 5.

Summary: Application-defined jobs and extensible Monty hosts

Scope: a bead that adds or edits crates/sc-cron

Deterministic check to pair with the development gate: Host registry integration tests invoke custom, sc-compose, and ATM functions through one extension interface.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-anr`, plan questions `run-anr.1`, dev questions `run-anr.2`

Fixtures:
- pass: A consumer registers its own Monty function beside sc_compose_run and atm_run while Rust/Python jobs remain supported.
- fail: Monty exposes only the fixed render_template function and cannot register application calls.

## REQ-CRON-0004

Anchor: The sc-cron crate creates one schedule table. A test writes a row without constructing SqliteWriter. The crate does not migrate the consumer's other tables.

> The sc-cron crate creates one schedule table. A test writes a row without constructing SqliteWriter. The crate does not migrate the consumer's other tables.

Said by randlee, requirements register, 2026-10-25. Status: locked. Version 2.

Summary: One schedule table

Scope: a bead that adds the sc-cron configuration table

Deterministic check to pair with the development gate: A test writes a schedule row without constructing SqliteWriter.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-ec5`, plan questions `run-ec5.1`, dev questions `run-ec5.2`

Fixtures:
- pass: The sc-cron crate creates one schedule table. A test writes a row without constructing SqliteWriter. The crate does not migrate the consumer's other tables.
- fail: The sc-cron crate migrates the consumer's other tables, and a schedule row is written by constructing SqliteWriter.

## REQ-CRON-0005

Anchor: Fallible public functions in sc-cron return CronError.

> Fallible public functions in sc-cron return CronError.

Said by randlee, requirements register, 2026-10-25. Status: locked. Version 2.

Summary: One CronError enum

Scope: a bead that adds or edits crates/sc-cron

Deterministic check to pair with the development gate: The public error type in crates/sc-cron is CronError.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-whn`, plan questions `run-whn.1`, dev questions `run-whn.2`

Fixtures:
- pass: Fallible public functions in sc-cron return CronError.
- fail: Fallible public functions in sc-cron return a type other than CronError.

## ADR-RUN-0013

Anchor: sc-cron is an optional general-purpose SQLite cron library with Rust/Python/Monty hooks, application-owned host registration, and no MVP script signing or separate approval mechanism.

> sc-cron is an optional SQLite cron crate. A normal schedule string is five fields, minute first. The hook target is a Rust function or a Python module run as python3 <module> cron.tick or cron.stop with one JSON object on stdin.

Said by randlee, Historical register quote retained as immutable provenance. Current statement follows the amended ADR-RUN-0013 authority and owner-approved Phase B continuation ANC-027 through ANC-042 (2026-10-29); the historical quote is not the current normative contract., 2026-10-25. Status: locked. Version 5.

Summary: General-purpose cron and application host capabilities

Scope: a bead that adds or edits crates/sc-cron

Deterministic check to pair with the development gate: The optional library provides application-defined job and Monty registration tests, and no MVP signing configuration.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-pny`, plan questions `run-pny.1`, dev questions `run-pny.2`

Fixtures:
- pass: The optional SQLite cron library runs Rust/Python/Monty jobs with application-owned host registration and no MVP signing gate.
- fail: Consumers must sign scripts before the SQLite scheduler runs them.

## ADR-RUN-0305

Anchor: SQLite and Postgres meet in an adaptor crate generated into the consumer repository.

> SQLite and Postgres meet in an adaptor crate generated into the consumer repository.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Generated consumer store adaptor

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: docs/architecture.md contains ADR-RUN-0305.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.

Beads: anchor `run-0fr`, plan questions `run-0fr.1`, dev questions `run-0fr.2`

Fixtures:
- pass: SQLite and Postgres meet in an adaptor crate generated into the consumer repository.
- fail: SQLite and Postgres meet inside the sc-runtime library crate, and no adaptor is generated into the consumer repository.

## REQ-RUN-0313

Anchor: The relationship between the local SQLite store and the remote Postgres or SQL store is defined in an adaptor crate on the consumer repository.

> The relationship between the local SQLite store and the remote Postgres or SQL store is defined in an adaptor crate on the consumer repository.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Adaptor crate defines the stores

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: A generated repository contains an adaptor crate that names both stores.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-dgl`, plan questions `run-dgl.1`, dev questions `run-dgl.2`

Fixtures:
- pass: The relationship between the local SQLite store and the remote Postgres or SQL store is defined in an adaptor crate on the consumer repository.
- fail: The relationship between the local SQLite store and the remote store is defined in the sc-runtime library crate.

## REQ-RUN-0314

Anchor: The copy adaptor is a local SQLite database with the remote schema, for faster or offline access.

> The copy adaptor is a local SQLite database with the remote schema, for faster or offline access.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Copy adaptor matches the remote schema

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: A generated copy adaptor creates the shared remote tables in SQLite.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-5zk`, plan questions `run-5zk.1`, dev questions `run-5zk.2`

Fixtures:
- pass: The copy adaptor is a local SQLite database with the remote schema, for faster or offline access.
- fail: The copy adaptor keeps a schema that differs from the remote database.

## REQ-RUN-0315

Anchor: The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts.

> The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Host aggregate collects many hosts

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: A generated host-aggregate adaptor documents one SQLite database per host and one Postgres aggregate.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-2is`, plan questions `run-2is.1`, dev questions `run-2is.2`

Fixtures:
- pass: The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts.
- fail: The host-aggregate adaptor is one SQLite database that aggregates many hosts and has no Postgres database.

## REQ-RUN-0316

Anchor: The consumer repository defines the rollup from host rows into the Postgres aggregate.

> The consumer repository defines the rollup from host rows into the Postgres aggregate.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Consumer defines the host rollup

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: The host-aggregate adaptor has a consumer-owned function that maps host rows into the Postgres aggregate.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-id3`, plan questions `run-id3.1`, dev questions `run-id3.2`

Fixtures:
- pass: The consumer repository defines the rollup from host rows into the Postgres aggregate.
- fail: The sc-runtime library defines the rollup from host rows into the Postgres aggregate.

## REQ-RUN-0317

Anchor: Adaptor Rust code and models in a new repository are generated by the command just new (scripts/new_project.py).

> Adaptor Rust code and models in a new repository are generated by the command just new (scripts/new_project.py).

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: just new generates the adaptor

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: just new writes the adaptor crate and its models into the new repository.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-bda`, plan questions `run-bda.1`, dev questions `run-bda.2`

Fixtures:
- pass: Adaptor Rust code and models in a new repository are generated by the command just new (scripts/new_project.py).
- fail: Adaptor Rust code in a new repository is written by hand, and just new does not generate it.

## REQ-RUN-0318

Anchor: The library crate named sc-runtime does not generate adaptor code.

> The library crate named sc-runtime does not generate adaptor code.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Library crate generates nothing

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: crates/sc-runtime contains no generator.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-cck`, plan questions `run-cck.1`, dev questions `run-cck.2`

Fixtures:
- pass: The library crate named sc-runtime does not generate adaptor code.
- fail: The library crate named sc-runtime generates the adaptor code.

## REQ-RUN-0319

Anchor: Generation flags name tables and columns that exist on only one of the two stores. A skipped table may be the SQLite table or the SQL-server table.

> Generation flags name tables and columns that exist on only one of the two stores.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 3.

Summary: Flags name one-sided tables and columns

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: Generation input can name a SQLite-only table, a SQL-server-only table, and a column that exists on only one side.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-xoo`, plan questions `run-xoo.1`, dev questions `run-xoo.2`

Fixtures:
- pass: Generation flags name tables and columns that exist on only one of the two stores. A skipped table may be the SQLite table or the SQL-server table.
- fail: Generation has no flag for a table that exists on only one of the two stores.

## REQ-RUN-0320

Anchor: Every consumer repository uses the same generated implementation of the copy pattern and the same generated implementation of the host-aggregate pattern.

> Every consumer repository uses the same generated implementation of the copy pattern and the same generated implementation of the host-aggregate pattern.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: One implementation per common pattern

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: Two generated repositories that select the same pattern contain the same adaptor implementation.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-jp8`, plan questions `run-jp8.1`, dev questions `run-jp8.2`

Fixtures:
- pass: Every consumer repository uses the same generated implementation of the copy pattern and the same generated implementation of the host-aggregate pattern.
- fail: Each consumer repository writes its own copy-pattern implementation.

## REQ-RUN-0321

Anchor: Each database engine's SQL is in that engine's store crate.

> Generated SQLite code and generated Postgres code are separate files.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 3.

Summary: SQL stays in that engine's crate

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: store-sqlite contains no Postgres SQL, and store-postgres contains no SQLite SQL.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-iuv`, plan questions `run-iuv.1`, dev questions `run-iuv.2`

Fixtures:
- pass: Each database engine's SQL is in that engine's store crate.
- fail: Postgres SQL is in the SQLite store crate.

## REQ-RUN-0322

Anchor: An adaptor that is neither the copy pattern nor the host-aggregate pattern is written by the consumer from the store building blocks.

> An adaptor that is neither the copy pattern nor the host-aggregate pattern is written by the consumer from the store building blocks.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 3.

Summary: Advanced adaptor is consumer code

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: A third adaptor is a file the consumer wrote.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-5ds`, plan questions `run-5ds.1`, dev questions `run-5ds.2`

Fixtures:
- pass: An adaptor that is neither the copy pattern nor the host-aggregate pattern is written by the consumer from the store building blocks.
- fail: An adaptor that is neither the copy pattern nor the host-aggregate pattern is generated by just new, and the consumer does not write it.

## REQ-RUN-0324

Anchor: The consumer adaptor moves rows between the local store and the remote store.

> The consumer adaptor moves rows between the local store and the remote store.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Adaptor moves rows between stores

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: The adaptor crate contains the copy from one store to the other.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-8b6`, plan questions `run-8b6.1`, dev questions `run-8b6.2`

Fixtures:
- pass: The consumer adaptor moves rows between the local store and the remote store.
- fail: The consumer adaptor does not move rows between the local store and the remote store.

## REQ-RUN-0325

Anchor: The sc-runtime crates do not move rows between stores.

> The sc-runtime crates do not move rows between stores.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Framework crates move no rows

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: crates/ contains no store-to-store copy.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-683`, plan questions `run-683.1`, dev questions `run-683.2`

Fixtures:
- pass: The sc-runtime crates do not move rows between stores.
- fail: The sc-runtime crates move rows between stores.

## REQ-RUN-0323

Anchor: Store synchronization is scheduled by the consuming application. The adaptor starts no timer.

> Store synchronization is scheduled by the consuming application. The adaptor starts no timer.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: The application schedules sync

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: The adaptor crate starts no timer. A schedule is registered by application code.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-7mk`, plan questions `run-7mk.1`, dev questions `run-7mk.2`

Fixtures:
- pass: Store synchronization is scheduled by the consuming application. The adaptor starts no timer.
- fail: The adaptor starts a timer that synchronizes the stores.

## REQ-RUN-0326

Anchor: A new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine.

> A new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 3.

Summary: Database layouts of a new application

Scope: a bead that edits scripts/new_project.py, wizard/answers.schema.json, or template store crates

Deterministic check to pair with the development gate: wizard/answers.schema.json accepts no database, SQLite only, one sqlx SQL engine, and SQLite plus one sqlx SQL engine.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-a67`, plan questions `run-a67.1`, dev questions `run-a67.2`

Fixtures:
- pass: A new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine.
- fail: A new application is created with SQLite only, and is not created with no database, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine.

## REQ-RUN-0327

Anchor: Two stores in one application use the same schema or a similar schema.

> Two stores in one application use the same schema or a similar schema.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Same schema or a similar schema

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: The copy pattern uses the same schema on both stores. The host-aggregate pattern uses a similar schema.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-hpn`, plan questions `run-hpn.1`, dev questions `run-hpn.2`

Fixtures:
- pass: Two stores in one application use the same schema or a similar schema.
- fail: Two stores in one application use unrelated schemas.

## REQ-RUN-0328

Anchor: Two stores in one application use one shared trait or a different trait per store.

> Two stores in one application use one shared trait or a different trait per store.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Shared trait or a trait per store

Scope: a bead that adds, removes, or generates a store adaptor

Deterministic check to pair with the development gate: A generated pair names one trait that both stores implement, or a trait for each store.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.
- `b-8`: Sprint b-8 does not carry this obligation.

Beads: anchor `run-uc6`, plan questions `run-uc6.1`, dev questions `run-uc6.2`

Fixtures:
- pass: Two stores in one application use one shared trait or a different trait per store.
- fail: Two stores in one application share no trait and do not have a trait per store.

## REQ-RUN-0329

Anchor: A generated crate's package name is the project name, a hyphen, and its role.

> A generated crate's package name is the project name, a hyphen, and its role. The adaptor depends on <project>-types and <project>-schema.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 4.

Summary: Package name carries the project name

Scope: a bead that edits template crate manifests or generation of package names

Deterministic check to pair with the development gate: Two projects with different names produce no package name in common.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-3q5`, plan questions `run-3q5.1`, dev questions `run-3q5.2`

Fixtures:
- pass: A generated crate's package name is the project name, a hyphen, and its role.
- fail: A generated crate's package name is its role alone, and the project name is not part of that name.

## REQ-RUN-0330

Anchor: Row type structs are in the schema crate. The types crate is for non-database types.

> Row type structs are in the schema crate. The types crate is for non-database types.

Said by randlee, ADR-RUN-0305, 2026-10-25. Status: locked. Version 2.

Summary: Row structs are in the schema crate

Scope: a bead that adds or generates the schema crate or the types crate

Deterministic check to pair with the development gate: The schema crate contains the row structs. The types crate contains no database row struct. The adaptor depends on the schema crate.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-3mg`, plan questions `run-3mg.1`, dev questions `run-3mg.2`

Fixtures:
- pass: Row type structs are in the schema crate. The types crate is for non-database types.
- fail: Row type structs are in the types crate, and the schema crate holds no row structs.

## REQ-RUN-0331

Anchor: The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface.

> The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface.

Said by randlee, requirements register, 2026-10-25. Status: locked. Version 2.

Summary: Disableable database commands

Scope: a bead that edits template CLI, MCP, or HTTP commands

Deterministic check to pair with the development gate: A generated project with a store registers delete row <table> <id> on the CLI, MCP, and HTTP, and a flag removes those commands.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-w7j`, plan questions `run-w7j.1`, dev questions `run-w7j.2`

Fixtures:
- pass: The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface.
- fail: The template ships no database command on the CLI, on MCP, or over HTTP.

## ADR-RUN-0002

Anchor: sc-runtime is a cargo-generate template plus published library crates.

> sc-runtime is a cargo-generate template plus published library crates.

Said by randlee, requirements register, 2026-10-25. Status: locked. Version 2.

Summary: Template plus published crates

Scope: a bead that adds a library crate or edits the template

Deterministic check to pair with the development gate: Generated projects depend on the library crates by version, and template code is rendered into the project.

Excluded from (not judged; the cell passes as `excluded`):
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-o8s`, plan questions `run-o8s.1`, dev questions `run-o8s.2`

Fixtures:
- pass: sc-runtime is a cargo-generate template plus published library crates.
- fail: sc-runtime is only an application, with no cargo-generate template and no published library crates.

## REQ-RUN-0503

Anchor: A project generated by just new passes lint, test, and both builds with no file edited.

> A project generated by just new passes lint, test, and both builds with no file edited.

Said by randlee, requirements register, 2026-10-25. Status: locked. Version 2.

Summary: Generated project is green

Scope: a bead that edits the template or scripts/new_project.py

Deterministic check to pair with the development gate: just new on the sqlite fixture exits 0, and just lint, just test, just build-cli, and just build-daemon exit 0 with no file edited.

Excluded from (not judged; the cell passes as `excluded`):
- `b-1`: Sprint b-1 does not carry this obligation.
- `b-2`: Sprint b-2 does not carry this obligation.
- `b-3`: Sprint b-3 does not carry this obligation.
- `b-4`: Sprint b-4 does not carry this obligation.
- `b-5`: Sprint b-5 does not carry this obligation.
- `b-6`: Sprint b-6 does not carry this obligation.
- `b-7`: Sprint b-7 does not carry this obligation.

Beads: anchor `run-bfz`, plan questions `run-bfz.1`, dev questions `run-bfz.2`

Fixtures:
- pass: A project generated by just new passes lint, test, and both builds with no file edited.
- fail: A project generated by just new fails lint until a source file is edited.

## ANC-027

Anchor: sc-cron is a general-purpose cron library supporting application-defined jobs.

> sc-cron is a general-purpose cron library.

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: General-purpose scheduler

Scope: edits sc-cron scheduling or job interfaces

Deterministic check to pair with the development gate: A consumer can schedule an application-defined job without a tool-specific payload.

Beads: anchor `run-ojx`, plan questions `run-ojx.1`, dev questions `run-ojx.2`

## ANC-028

Anchor: Embedding applications can register additional Monty host functions without modifying sc-cron.

> the primary requirement of being able to be extensible w/ other apps/calls is supported.

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Application host extensions

Scope: edits Monty host registration or dispatch

Deterministic check to pair with the development gate: An integration test registers and invokes a consumer-defined host function.

Beads: anchor `run-mvj`, plan questions `run-mvj.1`, dev questions `run-mvj.2`

## ANC-029

Anchor: The sc_compose_run integration supports any sc-compose command/subcommand supplied by an application script.

> sc_compose_run (any subcommand) would be a useful working example for showing how to use sc-cron

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Compose command coverage

Scope: edits sc_compose_run or its examples

Deterministic check to pair with the development gate: Argument-forwarding tests cover render and non-render subcommands.

Beads: anchor `run-uoc`, plan questions `run-uoc.1`, dev questions `run-uoc.2`

## ANC-030

Anchor: Phase B provides an atm_run host integration supporting any ATM CLI command/subcommand through the same extension interface used by sc_compose_run.

> i am in agreement.

Said by randlee, Owner approved the preceding proposal: "Phase B provides an atm_run host integration supporting any ATM CLI command/subcommand through the same extension interface used by sc_compose_run." The immutable quote "i am in agreement." is the owner's response approving that exact statement., 2026-10-29. Status: locked. Version 2.

Summary: ATM host integration

Scope: implements Phase B Monty integrations

Deterministic check to pair with the development gate: An ATM executable fixture is invoked through the same registry as sc_compose_run.

Beads: anchor `run-7l7`, plan questions `run-7l7.1`, dev questions `run-7l7.2`

## ANC-031

Anchor: Every production release crate excludes sc-compose and ATM crate dependencies and Rust imports, enforced by boundary lint; test and benchmark references are permitted.

> sc-compose and atm crates should be blocked from any production release crate.  sc-compose and atm can be referenced for tests i.e. benchmark tests

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 2.

Summary: Production tool crate boundary

Scope: edits production release manifests, imports, features, or boundary lint

Deterministic check to pair with the development gate: Boundary lint rejects production imports/dependencies while permitting test-only references.

Beads: anchor `run-z1z`, plan questions `run-z1z.1`, dev questions `run-z1z.2`

## ANC-032

Anchor: render_template is not required by the sc-cron product contract.

> render_template is not a product requirement

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 2.

Summary: No mandatory render helper

Scope: edits Monty product requirements, examples, or benchmarks

Deterministic check to pair with the development gate: Product requirements and acceptance do not require render_template.

Beads: anchor `run-bn5`, plan questions `run-bn5.1`, dev questions `run-bn5.2`

## ANC-033

Anchor: MVP executes application-supplied Monty scripts without script signing or a separate approval mechanism.

> for mvp this is not something available for consumers to do.

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 2.

Summary: No MVP signing gate

Scope: edits Monty execution, configuration, or authorization

Deterministic check to pair with the development gate: MVP execution requires neither signatures nor approval configuration.

Beads: anchor `run-wdb`, plan questions `run-wdb.1`, dev questions `run-wdb.2`

## ANC-034

Anchor: Python and Monty hooks use the same schedule-environment configuration and resolution rules.

> python/monty having consistent settings would be least confusing

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Shared hook environment

Scope: edits Python or Monty environment delivery

Deterministic check to pair with the development gate: The same environment cases produce equivalent Python and Monty results.

Beads: anchor `run-c1j`, plan questions `run-c1j.1`, dev questions `run-c1j.2`

## ANC-035

Anchor: Hooks inherit daemon-process environment values only through variable names explicitly allowed by daemon configuration.

> whitelisting api keys globally (by name) in config file (this is really a daemon scope decision)

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Explicit daemon inheritance

Scope: edits daemon environment configuration or subprocess creation

Deterministic check to pair with the development gate: An allowed daemon value is delivered and an unlisted value is absent.

Beads: anchor `run-6cy`, plan questions `run-6cy.1`, dev questions `run-6cy.2`

## ANC-036

Anchor: Each schedule can explicitly set or clear its own environment variables.

> having each entry having the ability to set or clear specific environment variables

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Schedule environment overrides

Scope: edits schedule environment persistence or resolution

Deterministic check to pair with the development gate: Two schedules set distinct identities and clearing removes a prior value.

Beads: anchor `run-89l`, plan questions `run-89l.1`, dev questions `run-89l.2`

## ANC-037

Anchor: Schedule creation and updates never implicitly capture environment values from the caller or daemon process.

> Never capture or persist inherited host values accidentally

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 2.

Summary: No implicit environment capture

Scope: edits schedule creation, updates, or serialization

Deterministic check to pair with the development gate: Ambient canaries are absent from persisted rows without explicit set input.

Beads: anchor `run-5z3`, plan questions `run-5z3.1`, dev questions `run-5z3.2`

## ANC-038

Anchor: The schedule registry permits deliberately supplied environment values as plaintext configuration.

> why can we never add values?  all environment variables are not api keys?

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Explicit plaintext values

Scope: edits schedule environment validation or persistence

Deterministic check to pair with the development gate: An explicit set value round-trips through the registry.

Beads: anchor `run-oov`, plan questions `run-oov.1`, dev questions `run-oov.2`

## ANC-039

Anchor: Schedules can resolve environment values through an application-provided resolver for opaque references.

> I also like allowing this:   "OTHER_API_KEY": { "reference": "keychain://agents/planner-7/api-key" }

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Application secret references

Scope: edits environment reference resolution

Deterministic check to pair with the development gate: A consumer resolver supplies a referenced value without persisting the result.

Beads: anchor `run-9a5`, plan questions `run-9a5.1`, dev questions `run-9a5.2`

## ANC-040

Anchor: Schedules can load environment variables from full-path .env files.

> Another useful way to specify would be loading from a full-path .env file.

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Absolute dotenv sources

Scope: edits schedule environment sources or file loading

Deterministic check to pair with the development gate: An authorized absolute dotenv file supplies hook values.

Beads: anchor `run-xaz`, plan questions `run-xaz.1`, dev questions `run-xaz.2`

## ANC-041

Anchor: Monty scripts can read their configured environment during source initialization, before hook functions run.

> there should also be the ability to set environment variables in a monty script before it runs.

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: Environment before initialization

Scope: edits Monty source initialization or environment delivery

Deterministic check to pair with the development gate: A top-level getenv reads the configured snapshot before cron_stop or cron_tick.

Beads: anchor `run-euz`, plan questions `run-euz.1`, dev questions `run-euz.2`

## ANC-042

Anchor: The first released schedule schema establishes one baseline without requiring migrations among unreleased development schemas.

> i don't know why we are on schema v4 when we haven't even released binaries yet.

Said by randlee, Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., 2026-10-29. Status: locked. Version 1.

Summary: First release schema baseline

Scope: edits schedule schema versioning or migrations

Deterministic check to pair with the development gate: First-release schema validation does not require a v1-v4 development migration ladder.

Beads: anchor `run-fv4`, plan questions `run-fv4.1`, dev questions `run-fv4.2`
