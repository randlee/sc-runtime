# sc-cron Requirements

**Status:** Active

Active as of 2026-10-28. The contract is [architecture.md](architecture.md), [ADR-RUN-0013](../architecture.md), and [ADR-RUN-0016](../architecture.md). Sprints `b-3` through `b-17` implement these items.

# Requirement: Workspace member

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

---

## REQ-CRON-0001: Workspace member

**Status:** Active  

### Requirement Statement

Definitions: sc-monty is the shared Monty interpreter implementation. sc-storage is the shared `Storage<T>` crate. The common sc-command read/write helpers are the asynchronous `storage_read` and `storage_write` functions over `Storage<T>` named in ADR-RUN-0021. The Phase D public package set is the nine crates named in REQ-RUN-0002, released together at version 0.3.0. A Monty value is an interpreter value passed to a hook function through sc-monty's value-level invocation, as opposed to a JSON-encoded string. The port preserves the existing test assertions while moving the interpreter implementation into sc-monty. Weakening an assertion means removing a checked condition, widening an expected value, range or tolerance, or replacing an exact comparison with a looser one. The local facade is the former unpublished observability-types shim. A workspace library crate is a library crate that is a member of this Cargo workspace. Naming a crate here means a manifest dependency; comments and schema descriptions may reference documented wiring. Registry types are the canonical event types imported from the published sc-observability-types crate. The command contract edge is the normal `sc-command` dependency entry in `crates/sc-cron/Cargo.toml`, permitted by ADR-RUN-0017. Test doubles are substitute components used only by tests. Capability-proven upstream cleanup is the removal of the command contract edge after the upstream canonical logging capability required by ADR-RUN-0017 has been verified. An isolated consumer is a crate outside this workspace that depends only on the crate under test, so workspace feature unification does not apply. Server-feature leakage is the appearance of axum, rmcp or the sc-command `server` feature in `cargo tree -e features` output for an isolated consumer of sc-cron or of sc-cron-commands, with default features or with the `monty` feature. The generated daemon is the `crates/daemon` package of a project rendered from this repository's `template` directory. The embedding application is the application that renders the template and chooses the template's `cron` placeholder, whose default is `false`. A default generated build is a default-feature build of a project rendered with the `cron` placeholder at its default. A counterpart assertion is the assertion in `crates/sc-cron` test source ported from one beads-cache `develop` test assertion. The beads-cache `develop` original of a counterpart assertion is the assertion it was ported from in `crates/sc-cron` of the randlee/beads-cache repository at branch `develop`. monty and monty-types are the third-party Monty interpreter crates that sc-monty depends on. An independent Monty engine is code outside sc-monty that compiles Monty source, manages Monty sessions, hosts Monty functions, enforces Monty limits, or dumps or loads Monty state.

`crates/sc-cron` is a workspace member, ported from beads-cache at `develop`, and is one of the Phase D public package set. Phase D sc-cron consumes the single sc-monty implementation for interpreter behavior and sc-storage for shared storage behavior; it no longer directly depends on monty or monty-types. It retains cron policy and JSON-to/from-value adaptation; Monty hooks receive Monty values rather than JSON strings. sc-sha is permitted for SHA-256 cache keys. No independent Monty compile/session/host/limits/dump/load engine remains here. Existing unit and integration assertions are ported without deletion or weakening. sc-cron-commands consumes the common sc-command read/write helpers without server-feature leakage.

The command contract edge supplies the retained observability port. Canonical event values remain direct registry `sc-observability-types` imports. Registry types use the open floor `>=1.5.0` with no upper bound, as every published registry dependency does under REQ-RUN-0002. The historical `0.1.1` release used an exact `=1.5.0` command pin; that pin does not govern Phase D.

`sc-runtime` does not name sc-cron. The generated daemon may name sc-cron only as an optional dependency enabled by its `cron` feature, which the embedding application selects; a default generated build does not link sc-cron. Here "name" means a Cargo or other manifest dependency. Documented wiring may reference the crate in comments and schema descriptions.

sc-cron's command contract edge enables neither the sc-command `server` feature nor the sc-command `test-util` feature; test doubles use dev-dependencies only. The local facade is gone and does not return. Capability-proven upstream cleanup removes the command contract edge, after which sc-storage and sc-monty are the only workspace library crates sc-cron depends on.

**MUST statements:**

- `crates/sc-cron` MUST be a member of this Cargo workspace.
- `crates/sc-cron` MUST be ported from beads-cache.
- sc-cron MUST be a member of the Phase D public package set.
- Phase D sc-cron MUST consume the single sc-monty implementation.
- Phase D sc-cron's shared storage MUST be sc-storage's `Storage<T>`.
- sc-cron MUST retain cron policy.
- sc-cron MUST retain JSON-to/from-value adaptation.
- When sc-cron invokes a Monty hook, the hook function MUST receive the hook data as a Monty value.
- Each assertion in the beads-cache `develop` unit-test source MUST have a counterpart assertion in the `crates/sc-cron` unit-test source.
- Each assertion in the beads-cache `develop` integration-test source MUST have a counterpart assertion in the `crates/sc-cron` integration-test source.
- sc-cron-commands MUST consume the common sc-command read/write helpers.
- The command contract edge MUST be the only normal `sc-command` dependency entry in `crates/sc-cron/Cargo.toml`.
- The command contract edge MUST set `default-features = false`.
- The command contract edge MUST enable no features.
- Canonical event values MUST remain direct registry sc-observability-types imports.
- sc-cron's sc-observability-types requirement MUST be the registry floor `>=1.5.0`.
- `crates/sc-cron/Cargo.toml` MUST declare every test-double dependency under `[dev-dependencies]`.
- Capability-proven upstream cleanup MUST delete the command contract edge from `crates/sc-cron/Cargo.toml`.

- If the generated daemon's `Cargo.toml` declares an `sc-cron` dependency, the `sc-cron` dependency entry MUST set `optional = true`.
- If the generated daemon's `Cargo.toml` declares an `sc-cron` dependency, the generated daemon's `cron` Cargo feature MUST enable the `sc-cron` dependency.
- If the generated daemon's `Cargo.toml` declares an `sc-cron` dependency, the generated daemon's `default` feature list MUST include `cron` only when the embedding application sets the `cron` placeholder to `true`.

**MUST NOT statements:**

- When sc-cron invokes a Monty hook, the hook function MUST NOT receive the hook data as a JSON-encoded string.

- `crates/sc-cron/Cargo.toml` MUST NOT declare a dependency on any workspace library crate besides sc-cron, sc-command, sc-storage and sc-monty.
- After capability-proven upstream cleanup, `crates/sc-cron/Cargo.toml` MUST NOT declare a dependency on any workspace library crate besides sc-cron, sc-storage and sc-monty.
- sc-cron's sc-observability-types requirement MUST NOT declare an upper bound.
- `crates/sc-cron/Cargo.toml` MUST NOT declare a dependency on the local facade.
- The `crates/sc-cron` integration-test source MUST NOT lack a counterpart assertion for any assertion in the beads-cache `develop` integration-test source.
- Each counterpart assertion in the `crates/sc-cron` integration-test source MUST NOT be a weakening of its beads-cache `develop` original.
- The `crates/sc-cron` unit-test source MUST NOT lack a counterpart assertion for any assertion in the beads-cache `develop` unit-test source.
- Each counterpart assertion under `crates/sc-cron/src/` MUST NOT be a weakening of its beads-cache `develop` original.
- `crates/sc-cron/Cargo.toml` MUST NOT declare a dependency on the `monty` crate.
- `crates/sc-cron/Cargo.toml` MUST NOT declare a dependency on the `monty-types` crate.
- `crates/sc-cron/src/` MUST NOT contain an independent Monty engine.
- An isolated consumer of sc-cron MUST NOT show server-feature leakage.
- An isolated consumer of sc-cron-commands MUST NOT show server-feature leakage.
- The `sc-runtime` crate MUST NOT declare `sc-cron` in any dependency manifest.
- `cargo tree` output for a default generated build MUST NOT list sc-cron.
- The command contract edge MUST NOT enable the sc-command `server` feature.
- The command contract edge MUST NOT enable the sc-command `test-util` feature.

### Rationale

An independently packaged scheduler can be embedded without the generated template. Its sc-command dependency supplies the retained observability port, while sc-storage and sc-monty supply shared storage and interpreter behavior.

### Success Criteria

**Acceptance Criteria:**

- [ ] Isolated default/Monty consumers must prove no axum/rmcp/server leakage; workspace feature unification is not that proof.

## REQ-CRON-0002: Normal crontab strings

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: A schedule is the cron expression string stored with an sc-cron job. A normal crontab string is a cron expression of five whitespace-separated fields in standard crontab order: minute, hour, day of month, month and day of week. The aliases and the six-field form of ADR-RUN-0013 point 2 stay valid; this record does not exclude them.

**MUST statements:**

- sc-cron MUST accept a normal crontab string as a schedule.

### Rationale

The beads-cache expression parser refused a five-field string; a normal crontab string is five fields, minute first (ADR-RUN-0013 Context). Owner, 2026-10-25: schedules are cron strings.
### Success Criteria

**Acceptance Criteria:**

- [ ] `*/5 * * * *` means every fifth minute.

# Requirement: Rust, Python, or Monty hook

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0003: Rust, Python, or Monty hook

**Status:** Active  

### Requirement Statement

Definitions: Monty is the Pydantic Python interpreter. sc-monty is the Rust crate owning the Monty interpreter implementation. Interpreter extraction is the move of the Monty interpreter implementation from sc-cron into sc-monty. The interpreter-extraction diff is the source change that performs interpreter extraction. A test assertion is an assertion in the source code of an sc-cron unit or integration test. sc-config is the synchronous JSON configuration loader crate in `crates/sc-config`. A Python module is the Python file path that a schedule's Python target names. A Python hook child is a `python3` child process that sc-cron starts with a Python module path as its first argument. Host dispatch is the sc-cron code in `crates/sc-cron/src/monty_run.rs` that routes a Monty host-function call to its handler. Schedule JSON is the JSON schedule definition a user passes to cron.start. A binary target is a Cargo target that builds an executable. A Monty interpreter call is a call that compiles Monty source or runs or resumes a Monty session. A Monty interpreter host is code that compiles Monty source, runs or resumes a Monty session, enforces Monty limits, or dumps and loads Monty sessions. A cron host adapter is an sc-cron function converting between JSON values and Monty values for a host-function call. A thin value-level adapter is a cron host adapter that converts values and calls sc-monty without being a Monty interpreter host. Cron capability behavior is the values, error codes and run outcomes cron host adapters produce for a hook source and input. Weakening an assertion means deleting the assertion, removing a checked condition, widening an expected value, range or tolerance, or loosening an exact comparison. A `Job` is an in-process Rust implementation of sc-cron's `Job` trait. Beads is the bd issue tracker. An HTTP route is a URL path bound to a handler in an HTTP server. A consumer-specific field is a field that exists only to serve one consumer application. bd-cache (beads-cache) is a consumer application caching the Beads bd CLI in SQLite. ATM means agent-team-mail. MCP means Model Context Protocol. CLI means command-line interface. argv is the ordered argument list a child process receives, starting with the executable path. The hook object is the `sc-cron.hook.v1` JSON object with fields `protocol`, `event`, `schedule`, `payload` and `deadline`. A run outcome is the `RunOutcome` stored in a schedule row's `last_outcome` column. A run outcome records error code X when the run outcome is `Failed` with detail beginning with X. A custom handler is an application-supplied `MontyHostHandler`. A reserved host name is `__hook`, `cron_stop` or `cron_tick`. A valid host-name identifier is an ASCII letter or underscore followed by ASCII letters, digits or underscores. Bridge argument conversion converts a Monty host-function call's arguments to JSON. A CLI adapter is the opt-in `sc_compose_run` or `atm_run` host function. The CLI adapter call shape is one non-empty command/subcommand string plus an optional list of strings, with no keyword arguments. The configured executable and configured working directory are the program path and working directory in `ScComposeHostConfig` or `AtmHostConfig`. The resolved environment snapshot is the environment sc-cron resolves from a schedule's hook environment under ADR-RUN-0016. The expected argv of a CLI adapter call is the configured executable followed by the call's command/subcommand string and the call's string arguments in call order. A CronStorage operation is a method of sc-cron's `CronStorage` facade, which enforces sc-cron schedule policy. A configured value is a non-empty environment-variable value in the resolved environment snapshot. `MAX_HOOK_OUTPUT_BYTES` is sc-cron's per-stream child output capture limit. The CLI result object is the JSON object whose keys `stdout`, `stderr` and `exit_code` hold the child's captured standard output, standard error and exit code. The storage action of a command is the closure the command passes to `storage_write`. The blocking child driver is the synchronous child runner in `crates/sc-cron/src/monty_compose.rs`. The row deadline is the deadline of one schedule run, from the row's timeout. Process-group termination kills every process in the child's process group. The feed thread is the blocking thread running a Monty session. `drive_feed` is the production Monty resume loop in `crates/sc-cron/src/monty_run.rs`. User cron.start, cron.end and cron.list are the commands of those names in `crates/sc-cron-commands/src`. The application transform is the `Transform` the embedding application supplies to cron.start. A domain transaction is the existing scheduler lifecycle storage transaction, as distinct from a user command calling `storage_read` or `storage_write`. An internal scheduler lifecycle operation is a storage operation sc-cron performs for scheduling rather than for a user command.

sc-cron schedules application-defined jobs; the consumer owns its API and schedule limits. `MontyHostRegistry` is immutable. `MontyHostHandler: Send + Sync` accepts a `MontyHostCall` (validated `MontyHostName`, positional and keyword JSON arguments, schedule name, hook phase, row deadline) and returns `Result<serde_json::Value, CronError>`; matching closures are supported. `MontyHostName::new` validates and `as_str()` borrows; registry keys use `MontyHostName` while registration accepts strings. Hook source cannot install or replace handlers. Private typed dispatch preserves supplied-adapter errors without parsing custom error text. The deadline check cannot preempt application code.

Executable defaults are `sc-compose` and `atm`; the working-directory default is the daemon directory captured at configuration. Every command/subcommand is supported; flags including `--root` are forwarded unchanged; no render arguments or temp-root staging are forced. CLI adapters do not call async `process::supervise`. Hook code decides how a child result affects the hook. Spawn failures record `HOST_SPAWN`. `render_template` is not a product requirement and need not be a registered host function.

MVP accepts application-supplied Monty source without signatures, script approval, or a consumer option enabling approval. Capability registration and environment policy are enforced independently. In-process Monty and custom handlers keep stack-overflow, allocator-abort, panic and non-cooperative deadline risks and are no isolation boundary for hostile code. Stop/tick ordering, generation fencing, retained REPL, interpreter-checkpoint VM limits and joined shutdown remain as ADR-RUN-0013 specifies. The cfg(test) helper is not a production path.

**MUST statements:**

- In `crates/sc-cron/src`, every Monty interpreter call MUST target the sc-monty crate.
- Every cron host adapter in `crates/sc-cron/src` MUST be a thin value-level adapter.
- After interpreter extraction, cron capability behavior for each existing hook test input MUST equal the cron capability behavior before interpreter extraction.
- When a schedule row whose target is an in-process `Job` is due, the scheduler MUST call the `Job`'s `run` method.
- When a schedule row whose target is a Python module is due, the scheduler MUST start a Python hook child for that Python module.
- With optional feature `monty`, when a schedule row whose target is `Monty { source }` is due, the scheduler MUST run `source` as a Monty hook.
- A Python hook MUST receive the hook object as its input.
- A Monty hook MUST receive the hook object as its input.
- Custom handlers MUST reach Monty hooks only through the `MontyHostRegistry` installed by `Scheduler::with_monty_hosts`.
- `MontyHostRegistry::new()` MUST return a registry containing no registered host name.
- `MontyHostRegistry` registration MUST return `CronError::InvalidSpec` for a host name that is not a valid host-name identifier.
- `MontyHostRegistry` registration MUST return `CronError::InvalidSpec` for a host name that is already registered.
- `MontyHostRegistry` registration MUST return `CronError::InvalidSpec` for a reserved host name.
- A Monty hook call to an unregistered host name MUST produce a run outcome recording error code `HOST_UNDEFINED`.
- A failed bridge argument conversion MUST produce a run outcome recording error code `HOST_DENIED`.
- A CLI adapter call whose arguments do not match the CLI adapter call shape MUST produce a run outcome recording error code `HOST_DENIED`.
- A custom handler that returns an error, including `CronError::InvalidSpec`, MUST produce a run outcome recording error code `HOST_CALL_FAILED`.
- When the row deadline has passed before a host-function call, the run outcome of the schedule run MUST be `TimedOut`.
- When a synchronous handler returns after the row deadline, host dispatch MUST record a `TimedOut` run outcome.
- The child argv of an `sc_compose_run` call MUST equal the expected argv of the call.
- The child argv of an `atm_run` call MUST equal the expected argv of the call.
- In `crates/sc-cron/src`, each CLI adapter MUST start its child process only through the blocking child driver.
- The child driver MUST enforce the row deadline.
- When the row deadline expires, the blocking child driver MUST apply process-group termination to the child.
- When a CLI adapter's child writes more than `MAX_HOOK_OUTPUT_BYTES` bytes to stdout, the run outcome of the schedule run MUST record error code `HOST_OUTPUT`.
- When a CLI adapter's child writes more than `MAX_HOOK_OUTPUT_BYTES` bytes to stderr, the run outcome of the schedule run MUST record error code `HOST_OUTPUT`.
- When a CLI adapter's child exits with exit code zero, the CLI adapter call MUST evaluate to the CLI result object in the Monty hook.
- When a CLI adapter's child exits with a nonzero exit code, the CLI adapter call MUST evaluate to the CLI result object in the Monty hook.
- When the row deadline expires while a CLI adapter's child runs, the run outcome MUST be `TimedOut`.
- The environment of an `sc_compose_run` child MUST equal the resolved environment snapshot.
- The environment of an `atm_run` child MUST equal the resolved environment snapshot.
- User cron.start MUST write its schedule row only by calling `storage_write`.
- The storage action of user cron.start MUST call only CronStorage operations.
- User cron.end MUST write its schedule change only by calling `storage_write`.
- The storage action of user cron.end MUST call only CronStorage operations.
- User cron.list MUST read schedule rows only by calling `storage_read`.
- User cron.start MUST store the schedule JSON returned by the application transform in the new schedule row.
- In `crates/sc-cron/src`, each internal scheduler lifecycle operation MUST run inside a domain transaction.
- In `crates/sc-cron/src`, `drive_feed` MUST be the only production code that resumes a Monty session after a host-function call.

**MUST NOT statements:**

- The interpreter-extraction diff MUST NOT weaken a test assertion in `crates/sc-cron`.
- When a custom handler returns an error, the detail text of the run outcome MUST NOT include a configured value.
- When the row deadline has passed before a host-function call, host dispatch MUST NOT call the handler.
- In `crates/sc-cron/src`, CLI adapter code MUST NOT invoke a shell to start a child process.
- `crates/sc-cron/src` MUST NOT contain a Monty interpreter host.
- In `crates/sc-cron/src/hook.rs`, the hook object MUST NOT contain a consumer-specific field for Beads.
- In `crates/sc-cron/src/hook.rs`, the hook object MUST NOT contain a consumer-specific field for bd-cache.
- In `crates/sc-cron/src/hook.rs`, the hook object MUST NOT contain a consumer-specific field for ATM.
- `crates/sc-cron/src` MUST NOT register an HTTP route.
- `crates/sc-cron/src` MUST NOT define an MCP tool.
- The sc-cron package MUST NOT define a binary target.
- `crates/sc-cron/Cargo.toml` MUST NOT declare a dependency on sc-config.
- A Monty hook MUST NOT be able to change the configured executable.
- A Monty hook MUST NOT be able to change the configured working directory.
- A CLI adapter MUST NOT start an async runtime on the feed thread.
- A CLI adapter MUST NOT call `block_on` on the feed thread.
- A Monty hook that calls no CLI adapter MUST NOT fail when the `sc-compose` and `atm` executables are absent.

### Rationale

Application-supplied hooks and value adapters let the reusable scheduler execute registered application behavior without implementing another interpreter host.

### Success Criteria

**Acceptance Criteria:**

- [ ] Host registry integration tests invoke custom, sc-compose, and ATM functions through one extension interface.
- [ ] Argument-forwarding tests cover render and non-render subcommands.
- [ ] MVP execution requires neither signatures nor approval configuration.

# Requirement: Application schedule storage and compiled-session cache

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0004: Application schedule storage and compiled-session cache

**Status:** Active  
**Last Updated:** 2026-11-13

### Requirement Statement

Definitions: CronRow is the schedule-row type owned by sc-cron. Storage<T> is the shared storage trait for row type T. hook_environment is the canonical schedule environment containing dotenv sources and variable operations. ScheduleLimits is the embedding application's schedule-limit configuration. sc-monty is the shared Monty interpreter implementation. Monty dump version identifies the compiled-session serialization version. SHA-256 is the source digest supplied by sc-sha. Cache rows are the sc-cron-owned MontyBlobRow rows; a consumer table is any application table other than the CronRow table and the MontyBlobRow table. sc-cron migrations are the schema migrations that sc-cron runs. A consumer table is unchanged when its schema and rows equal those it had before sc-cron migrations ran. CronStorage is sc-cron's policy-checked schedule-storage facade over Storage<CronRow> (ADR-RUN-0021). The schedule-row limit is the row limit in ScheduleLimits. A schedule writer is any process or sc-cron component that writes a CronRow through CronStorage. A compiled session is the result of sc-monty compiling a schedule row's source; its blob is that result serialized in a MontyBlobRow. A cache entry is one MontyBlobRow. Generic row schemas are the schemas of rows stored through Storage<T>. MontyBlobRow is the row type that stores compiled-session blobs. An sc-cron generic row type is a Storage<T> row type defined in the sc-cron crate. The configuration schema here is sc-cron's schedule configuration schema. The application's SQLite database is the SQLite database that holds the application's own rows. The application's databases are the SQLite, Postgres or MySQL databases that the application's configuration opens. The application database selected for cron is the application's database, SQLite, Postgres or MySQL, that the application's configuration names for the CronRow table, or the application's SQLite database when the configuration names none. Hook run state is any value a hook stores while it runs, such as a value cron_tick stores for a later tick. A missing cache entry is the absence of a MontyBlobRow for the key. A stale cache entry is a MontyBlobRow whose key does not match the schedule-row source. A corrupt cache entry is a MontyBlobRow that sc-monty cannot load. An unreadable-version cache entry is a MontyBlobRow whose Monty dump version differs from the running sc-monty's dump version. A cache entry missing a hook is a MontyBlobRow whose loaded session lacks cron_stop or cron_tick.

sc-cron owns CronRow for schedules and MontyBlobRow for compiled hook sessions. Generic storage engines contain no cron-specific schema or SQL.

**MUST statements:**

- A CronRow that CronStorage writes MUST be returned by a read from the application's Storage<CronRow> implementation.
- After CronStorage writes a CronRow, the CronRow table MUST be in the application database selected for cron.
- When the application's configuration names no database for the CronRow table, the CronRow table MUST be created in the application's SQLite database.
- CronRow MUST store the schedule's canonical hook_environment unchanged.
- The default hook_environment MUST be the empty value {"dotenv":[],"variables":{}}.
- The configured schedule-row limit MUST count enabled and disabled rows.
- When the number of schedule rows equals the schedule-row limit, CronStorage MUST return CronError::InvalidSpec for a write that adds a new schedule name.
- For a CronRow write by any schedule writer, CronStorage MUST enforce the schedule-row limit of the embedding application's ScheduleLimits.
- Every schedule writer that writes CronRows to one database MUST pass identical ScheduleLimits.
- CronStorage MUST return an error for a schedule write whose Monty source sc-monty cannot compile.
- Each `MontyBlobRow` MUST contain the compiled session exactly as sc-monty compiled it.
- Compiled-session blobs MUST use sc-sha SHA-256 of the exact source plus Monty dump version as their key.
- The schedule-row source MUST remain authoritative for matching a cached blob.
- A cache entry that is missing, stale, corrupt, of an unreadable version or missing a hook MUST trigger recompilation from the schedule-row source.
- When sc-cron saves a recompiled session, sc-cron MUST write that session as a new `MontyBlobRow`.
- After sc-cron writes a new `MontyBlobRow` for a recompiled session, that `MontyBlobRow` MUST replace the cache entry for subsequent use.
- MontyBlobRow MUST be an sc-cron generic row type.
- Cache rows MUST use sc-cron-owned migrations.
- After sc-cron migrations run, every consumer table MUST be unchanged.
- After sc-cron migrations run, every table that sc-cron migrations created MUST be in one of the application's databases.

**MUST NOT statements:**

- sc-cron MUST NOT change the bytes of a stored `MontyBlobRow`.
- After a hook runs, the stored `MontyBlobRow` MUST NOT contain hook run state.
- The configuration schema MUST NOT add a separate environment table.
- The configuration schema MUST NOT add a separate secret table.

### Rationale

The schedule configuration is one CronRow table in an application database. Reloading schedules loads every row into memory; an uncapped row count is unsafe when schedules come from operators or configuration, and sc-cron must not depend on `sc-config` (ADR-RUN-0014).

### Success Criteria

The ported cron conformance tests preserve schedule fields and row-limit behavior. Compiled-cache tests cover source/version match, invalid source at submit, pre-run snapshot timing and each cache-miss condition.

### Implementation Notes

The row limit is application configuration rather than stored table state; independent writers with different configured limits do not enforce one another's limits. Saving a compiled session is permitted, not required: PRD-MONTY-5 says sc-cron can save a compiled Monty binary and leaves that design open to refinement. The CronRow table may be in a SQLite, Postgres or MySQL application database (PRD-CRON-3, PRD-STO-2).

## REQ-CRON-0005: One error enum

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: sc-cron is the crate at crates/sc-cron, whose Rust library name is `sc_cron`. A schedule-store method is a method of `Storage<CronRow>`, the shared storage trait through which sc-cron reads and writes its schedule rows (PRD-CRON-2). A store is a `Storage<CronRow>` implementation that holds schedule rows. An operation completes when its call returns, whether with a value or an error. A fallible sc-cron function is a public function or public method reachable from the public API of the `sc_cron` crate whose return type is any `Result` type; trait methods that the application implements, such as `EnvironmentReferenceResolver::resolve`, are excluded. `Captured` is the child-output type in crates/sc-cron/src/process.rs. The CronError table is the variant-to-code table below. StoreErrorKind is the `#[non_exhaustive]` discriminator inside `CronError::Store`, defined in crates/sc-cron/src/error.rs. A cooperative operation is a schedule-store operation run under a `StoreOperationContext` (REQ-CRON-0020). Call arguments are the values passed to the store method. A cooperative operation failure is an error returned by a cooperative operation. Schedule-row contents are the stored field values of one schedule row. Provider prose is text produced by an application callback or a storage provider. A Monty feed is one run of a schedule row's Monty hook with feature `monty`. The condition table is the condition-to-code table below. A host adapter is an sc-cron component that answers a Monty host call, such as the CLI adapters `sc_compose_run` and `atm_run`. A host-adapter error is a failure returned by a host adapter. A host-deadline expiration is a Monty feed that exceeds its row deadline, as in the last two rows of the condition table. A code token is a code listed in the CronError table or in the condition table. A host-deadline outcome is the `RunOutcome::TimedOut` recorded for a host-deadline expiration. A deadline-exceeded failure is a `StoreError` whose kind is `StoreErrorKind::DeadlineExceeded`. An indeterminate store error is a `StoreError` whose kind is `StoreErrorKind::Indeterminate`. SQL (Structured Query Language) text is the text of an SQL statement. The detail form of a condition-table row is `CODE: message; recovery`, where `CODE` is the row's code, message is the failure text and recovery is the row's operator recovery text. A `TransactionOutcome` is the persistence report returned by `StoreError::transaction_outcome()`.

`DeadlineExceeded` may carry confirmed committed-late evidence. `StoreError::transaction_outcome()` is optional. REQ-CRON-0020 and ADR-RUN-0018 govern the admission and persistence report policy. Application callbacks keep their own result types: `EnvironmentReferenceResolver::resolve` fails with the value-free `EnvironmentReferenceError::{Denied, Unavailable}`. `RunOutcome::failure_parts` in crates/sc-cron/src/run_failure.rs parses a `Failed` detail. Reading crates/sc-cron checks statements about types, signatures and enum variants. Unit tests in crates/sc-cron check the other statements by calling the named function or operation and reading the returned error, the recorded `RunOutcome` or the stored schedule row.

| CronError variant | Code |
|---|---|
| `BadExpression` | `BAD_EXPRESSION` |
| `InvalidSpec` | `INVALID_SPEC` |
| `AlreadyActive` | `ALREADY_ACTIVE` |
| `NotFound` | `NOT_FOUND` |
| `HookFailed` | `HOOK_FAILED` |
| `Store` | `STORE_FAILED` |
| `Fenced` | `FENCED` |

The condition table matches [architecture.md](architecture.md):

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Bridge argument conversion or supplied CLI adapter shape rejected before execution | `HOST_DENIED` | `Failed` | Fix the call's JSON conversion or the supplied adapter's documented argument shape. |
| Custom application handler returns an error, including `InvalidSpec` | `HOST_CALL_FAILED` | `Failed` | Fix the application handler; configured environment values are redacted from the recorded detail. |
| CLI executable missing or not runnable | `HOST_SPAWN` | `Failed` | Fix the configured executable path or install the external CLI. |
| CLI output is invalid UTF-8 or exceeds the per-stream cap | `HOST_OUTPUT` | `Failed` | Return bounded UTF-8 stdout and stderr. |
| CLI exits nonzero | — | Script receives result | Inspect `exit_code` and bounded `stderr`; the adapter does not turn an application CLI result into an automatic hook failure. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| Monty `OsCall` other than `Getenv` or `GetEnviron` | `HOST_OS` | `Failed` — detail names the operation | Use the configured host registry and Getenv/GetEnviron; other OS operations remain denied. |
| Allowlisted daemon variable is not Unicode or exceeds the value cap | `ENV_PROCESS_INVALID` | `Failed` — value-free detail | Restart the daemon with a bounded UTF-8 value or remove the name from the allowlist. |
| `.env` path outside daemon policy | `ENV_FILE_DENIED` | `Failed` — value-free detail | Authorize the canonical file/root or remove the schedule source. |
| Required `.env` file missing | `ENV_FILE_MISSING` | `Failed` — value-free detail | Restore the absolute file or mark the source optional. |
| `.env` file invalid | `ENV_FILE_INVALID` | `Failed` — value-free detail | Fix the bounded data-file syntax. Shell expansion is not supported. |
| Reference missing or resolver failed | `ENV_REFERENCE` | `Failed` — value-free detail | Configure the daemon resolver and restore the referenced value. |
| Resolved entry-count or encoded-block limit exceeded | `ENV_SNAPSHOT_LIMIT` | `Failed` — value-free detail | Remove variables or reduce values before retrying. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| `cron_stop` return value is not a valid stop reply | `HOST_REPLY` | `Failed` — detail is `stop check: HOST_REPLY:` then the parse error | Fix hook source; return an object with string `"action"` (`continue`, `disable`, or `delete`) and a non-empty `"reason"` when disabling or deleting. |
| Monty feed task join failed (cancelled blocking-pool task) | `FEED_TASK_FAILED` | `Failed` | Retry the run; if it persists, inspect blocking-pool capacity and hook source. |
| Monty feed task panicked (worker join) | `FEED_JOIN_FAILED` | `Failed` | Fix the panic in the hook or host and retry the schedule. |
| Monty due task panicked or was cancelled | `DUE_JOIN_FAILED` | `Failed` | Retry the schedule; if it persists, inspect the Monty due task and hook source. |
| Monty due path used for a non-Monty hook | `MONTY_RESOLVE` | `Failed` | Run this row on its Rust or Python target, or set `HookTarget::Monty`. |
| `cron_tick` returned a stop action (disable/delete) | `MONTY_TICK` | `Failed` | Return normally from `cron_tick`; disable and delete belong in `cron_stop`. |
| After the CLI returns, host `Instant` exceeds the row budget |  | `TimedOut`, session dropped | Reduce CLI work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Interpreter checkpoints govern VM execution; host handlers use the row deadline. |

**MUST statements:**

- Every fallible sc-cron function MUST have a return type of the form `Result<T, CronError>`.
- The `CronError` enum in crates/sc-cron/src/error.rs MUST define exactly the variants in the CronError table.
- `CronError::code()` called on any `CronError` value MUST return the code that the CronError table lists for the value's variant.
- `process::supervise` in crates/sc-cron/src/process.rs MUST return `Result<Captured, CronError>`.
- `process::supervise_with_logger` in crates/sc-cron/src/process.rs MUST return `Result<Captured, CronError>`.
- Every cooperative operation failure MUST be a `CronError::Store` value.
- StoreErrorKind MUST carry the `#[non_exhaustive]` attribute.
- StoreErrorKind MUST include the variant `Cancelled`.
- StoreErrorKind MUST include the variant `DeadlineExceeded`.
- StoreErrorKind MUST include the variant `Indeterminate`.
- When a schedule's environment reference is missing, the Monty feed's recorded detail MUST start with `ENV_REFERENCE:`.
- When `EnvironmentReferenceResolver::resolve` returns an error, the Monty feed's recorded detail MUST start with `ENV_REFERENCE:`.
- For each row of the condition table, a Monty feed that meets the row's condition MUST record the row's `RunOutcome`.
- For each row of the condition table that lists a code, the recorded `RunOutcome::Failed` detail MUST have the row's detail form.
- For a host-deadline expiration, the Monty feed MUST record `RunOutcome::TimedOut`.

**MUST NOT statements:**

- A host-deadline outcome MUST NOT contain a code token.
- When a cooperative operation fails, the string returned by `to_string()` on the returned error MUST NOT contain SQL text.
- When a cooperative operation fails, the string returned by `to_string()` on the returned error MUST NOT contain provider prose.
- After an operation returns a deadline-exceeded failure, sc-cron MUST NOT run the operation again without a new call from the application.
- When `transaction_outcome()` is called on an indeterminate store error, the call MUST NOT return a `TransactionOutcome` whose persistence is `RolledBack`.
- When `transaction_outcome()` is called on an indeterminate store error, the call MUST NOT return a `TransactionOutcome` whose persistence is `NoMutationAttempted`.
- When `to_string()` is called on an indeterminate store error, the returned string MUST NOT state that the store transaction was not persisted.
- A public error type in crates/sc-cron MUST NOT declare a field that holds provider prose.
- The `CronError` enum in crates/sc-cron/src/error.rs MUST NOT contain a variant for host-adapter errors.

### Rationale

Every cross-boundary failure exposes stable codes, causes, and recovery/remediation; agents must be able to act on failures without reading source (framework charter). The variant set and codes match the beads-cache crate that sc-cron ports.
### Success Criteria

**Acceptance Criteria:**

- [ ] The public error type in crates/sc-cron is CronError.

## REQ-CRON-0019: Explicit Python and Monty hook environment

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-28  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: sc-cron is the scheduler crate of this workspace. CLI means command-line interface. OS means operating system. KiB means 1,024 bytes. UTF-8 is the Unicode byte encoding. A schedule row is one stored schedule. A Python row, a Monty row and a Rust row are schedule rows whose hook is Python source, Monty source and a Rust `Job`. A scripted row is a Python row or a Monty row. Hook source is the Python or Monty code of a scripted row. A schedule run is one execution of a schedule row. `cron.stop` and `cron.tick` are the two hook entry points of a schedule run, the stop check and the tick, called in that order. `HookEnvironment` is the type that holds a row's environment configuration: an ordered list of absolute dotenv file paths followed by schedule operations. Schedule operations are the name-keyed `Set`, `Reference` and `Clear` operations of a `HookEnvironment`: `Set` gives a literal value, `Reference` names an opaque reference, and `Clear` removes the name. A `HookEnvironment` is empty when it has no dotenv file and no schedule operation. An existing constructor is a schedule-row constructor that sc-cron offered before `HookEnvironment` was added. `HookEnvironmentHost` is the daemon-scoped policy type with four parts: the inherit list (names of daemon-process variables that may be inherited), the list of authorized dotenv files, the list of authorized dotenv roots, and an optional reference resolver. An empty host is a `HookEnvironmentHost` whose three lists are empty and whose reference resolver is absent. The daemon is the application's long-running process, and the daemon environment is the set of environment variables of that process. The reference resolver is the application-supplied function that turns an opaque reference into a value; it answers `Denied`, `Unavailable`, missing or a value. An opaque reference is the locator string of a `Reference` operation, which sc-cron does not interpret. Reference resolution is one call of the reference resolver. The resolver request is the argument set of that call and has exactly four fields: the schedule name, the variable name that the `Reference` operation is keyed by, the opaque reference, and the row deadline. Provider prose is any text from the reference resolver other than a returned value. sc-cron emits a text when it writes that text to an outcome, an event, a trace, a log or a persisted reason. The row deadline is the deadline of the schedule row being run. `RunOutcome::TimedOut` is the outcome sc-cron records for a run that passed its deadline. The generated administrator configuration is the daemon configuration file that the project template renders. Its allowlist keys are `allow_process`, `allow_dotenv_files` and `allow_dotenv_roots`. Generated scheduler composition is the generated daemon code that builds the scheduler. The prebuilt host policy is the `HookEnvironmentHost` value that the generated daemon builds and passes to generated scheduler composition. An allowlisted daemon value is the value of a daemon-process variable named in the inherit list. A file value is a value read from a dotenv file. A reference value is a value returned by the reference resolver. A resolved value is an allowlisted daemon value, a file value or a reference value. A stored value is a `Set` value held in a schedule row. An environment value is a stored value or a resolved value. Resolution builds the resolved map from sources; applying a source writes its entries into the map being built and replaces an entry of the same name. The resolution order is: first the allowlisted daemon values, then the dotenv files in the order in which the `HookEnvironment` lists them, then the schedule operations. The resolved map is the map of environment names to values that resolution produces for one schedule run. A snapshot map is the map being built after any one source is applied, or the resolved map. The encoded final environment block is the byte encoding of the resolved map that is handed to a hook; ADR-RUN-0016 specifies the encoding. A limit failure is the error for one source that breaks a limit of this record (a per-source oversize failure), or the `ENV_SNAPSHOT_LIMIT` error for a snapshot map or an encoded block that breaks a limit of this record (an aggregate-limit failure). To persist is to write to the sc-cron database or to a file. A Monty OS call is a request from Monty code to its host for an operating-system service. `Getenv` is the Monty OS call that reads one variable and takes a caller-supplied default. `GetEnviron` is the Monty OS call that reads all variables. A non-environment OS call is a Monty OS call that is not `Getenv` and is not `GetEnviron`. A Monty OS call is denied when sc-cron returns an error to the Monty code and does not perform the service. An absent key is a key that no source set, or a key that a `Clear` operation removed. A CLI adapter child is a child process started by `sc_compose_run` or `atm_run`. An invoking caller is the code or process that calls into sc-cron. sc-cron takes a caller's environment when it reads the caller's process environment variables (capture), or derives a variable value from anything about the caller other than an explicit `HookEnvironment` or `HookEnvironmentHost` (inference). A controlled text is a failure detail (the error text of a failed run) or a stop reason (the text a hook gives for stopping its schedule) that contains text produced by hook code, a CLI adapter child or the reference resolver: Python output, Monty exception text, Monty return text, and the standard output and standard error of each CLI adapter child. Value replacement substitutes the literal `[REDACTED_ENV]` for each occurrence of each distinct non-empty resolved value in a text. A later step is the byte cap (the length limit sc-cron applies to a controlled text) or output assembly (composing fixed codes, recovery messages, outcomes, events, traces and persisted reasons). An exempt text is stable text (the fixed codes and recovery messages), the non-provider metadata of a run, or the serialized `HookEnvironment` written by the canonical environment configuration write. `HookOutput` is the public type that carries a hook's raw output; its raw output fields hold text that has had no value replacement.

**MUST statements:**

1. Every scripted row MUST hold its environment configuration as a `HookEnvironment`.
2. Each existing constructor MUST create a schedule row whose `HookEnvironment` is empty.
3. The default value of `HookEnvironmentHost` MUST be an empty host.
4. The generated administrator configuration MUST set `allow_process` to an empty array.
5. The generated administrator configuration MUST set `allow_dotenv_files` to an empty array.
6. The generated administrator configuration MUST set `allow_dotenv_roots` to an empty array.
7. The scheduler built by generated scheduler composition MUST use the reference resolver of the prebuilt host policy.
8. Each reference resolution MUST pass the complete resolver request to the reference resolver.
9. Reference resolution MUST end no later than the row deadline.
10. If the row deadline expires during reference resolution, the run's outcome MUST be `RunOutcome::TimedOut`.
11. Resolution MUST finish building the resolved map before `cron.stop` is called.
12. `cron.tick` MUST receive the resolved map that `cron.stop` received.
13. Resolution MUST apply its sources in the resolution order.
14. The canonical environment configuration write MUST store each `Set` value as unencrypted text.
15. sc-cron MUST reject a Rust row whose `HookEnvironment` is not empty.
16. Every snapshot map MUST contain at most 128 entries.
17. Every environment value MUST be at most 16 KiB long.
18. The encoded final environment block MUST be at most 32,000 UTF-8 bytes long.
19. The error text of every limit failure MUST contain no environment value.
20. The environment of the Python process that runs a Python row MUST equal the resolved map.
21. For a key present in the resolved map, `Getenv` MUST return the resolved-map value.
22. For an absent key, `Getenv` MUST return the default supplied in the call.
23. `GetEnviron` MUST return the resolved map.
24. Each non-environment OS call MUST be denied.
25. The environment of each CLI adapter child MUST contain every entry of the resolved map.
26. Value replacement MUST be applied to every controlled text.
27. Value replacement of a controlled text MUST run before each later step of that text.
28. Value replacement MUST leave every exempt text unchanged.
29. The `Debug` formatting of `HookOutput` MUST omit the contents of its raw output fields.

**MUST NOT statements:**

30. The resolved map MUST NOT change after resolution finishes.
31. If the row deadline expires during reference resolution, hook source MUST NOT run.
32. A CLI adapter child MUST NOT inherit the daemon process environment.
33. sc-cron MUST NOT take the environment of an invoking caller.
34. sc-cron MUST NOT emit provider prose.
35. sc-cron MUST NOT persist a resolved value.
36. sc-cron MUST NOT change a variable of the daemon environment.

### Rationale

ANC-034 asks for consistent Python and Monty settings, so both row kinds use one `HookEnvironment`. ANC-037 forbids accidental capture or persistence of inherited host values, so the default host is empty and resolved values stay out of storage. ADR-RUN-0016 records that environment variables are not uniformly secrets: durable values such as `ATM_IDENTITY` and `ATM_TEAM` belong with a schedule, while API keys may be daemon-wide inherited values, explicit schedule values, references resolved by an application secret provider, or values loaded from an authorized absolute dotenv file. Reference locators are non-secret registry metadata; only their resolved values remain external. Trusted Rust jobs keep their existing `Job` interface, which is why a Rust row carries no hook environment. Trusted direct callers may read the raw `HookOutput` fields. Value replacement mitigates accidental propagation only: a trusted hook can transform or exfiltrate a value, so this record sets no untrusted-code boundary. Full semantics, limits, file authorization and the release-v1 schema baseline are in ADR-RUN-0016.

### Success Criteria

Every check below is a unit test in the sc-cron test suite unless it names a file. Each test drives time through a controlled clock and a controlled reference resolver.

**Acceptance Criteria:**

- [ ] Statement 1: a unit test builds a Python row and a Monty row from one `HookEnvironment` value and reads the same value back from each.
- [ ] Statement 2: a unit test calls each existing constructor and asserts the row's `HookEnvironment` is empty.
- [ ] Statement 3: a unit test asserts the three lists and the resolver of the default `HookEnvironmentHost`.
- [ ] Statements 4 to 6: inspection of the generated administrator configuration file of a generated project shows the three allowlist keys as empty arrays.
- [ ] Statement 7: a unit test of generated scheduler composition passes a host policy with a recording resolver and asserts a `Reference` operation reaches that resolver.
- [ ] Statement 8: a unit test with a recording resolver asserts the four fields of the resolver request.
- [ ] Statements 9, 10 and 31: a unit test uses a resolver that does not answer, advances the controlled clock past the row deadline, and asserts the call ended, the outcome is `RunOutcome::TimedOut`, and the hook source recorded no call.
- [ ] Statements 11, 12 and 30: a unit test records the map seen at `cron.stop` and at `cron.tick`, changes a daemon variable between the two, and asserts both maps are equal.
- [ ] Statement 13: a unit test sets one name in a daemon variable, two dotenv files and a `Set` operation, and asserts which value wins at each step.
- [ ] Statements 14 and 28: a unit test stores a row with a `Set` value equal to a resolved value and reads the same text back from the stored row; a second test asserts fixed codes, recovery messages and metadata that contain a resolved value are unchanged.
- [ ] Statement 15: a unit test asserts an error for a Rust row with one `Set` operation.
- [ ] Statements 16 to 19: unit tests at each limit and one over it assert acceptance, then the failure, and assert the error text holds none of the test's values.
- [ ] Statement 20: a unit test runs a Python row that returns its whole environment and compares it with the resolved map.
- [ ] Statements 21 to 24: unit tests run a Monty row that calls `Getenv` for a present key, a never-set key and a cleared key, calls `GetEnviron`, and makes one non-environment OS call.
- [ ] Statements 25 and 32: a unit test starts each CLI adapter child with a stub program that returns its environment, with a canary variable set in the daemon environment.
- [ ] Statements 26 and 27: unit tests feed a resolved value through Python output, Monty exception text and adapter standard error, and assert `[REDACTED_ENV]` in the failure detail and the stop reason, and a replaced value that is not cut by the byte cap.
- [ ] Statement 29: a unit test formats a `HookOutput` with `{:?}` and asserts the raw text is absent.
- [ ] Statement 33: a unit test sets a canary variable in the test process and asserts it is absent from the resolved map of a row run with an empty host.
- [ ] Statement 34: a unit test uses a resolver whose `Denied` answer carries extra text and asserts that text is absent from every outcome, event, trace and persisted reason.
- [ ] Statement 35: a unit test resolves a canary daemon value, file value and reference value, then asserts none appears in the sc-cron database.
- [ ] Statement 36: a unit test compares the daemon environment before and after a schedule run.

## NFR-CRON-0001: Allowed third-party dependencies

**Version:** 1.0.0  
**Created:** 2026-10-27  
**Last Updated:** 2026-11-12  
**Owner:** randlee  
**Status:** Active  

### Requirement Statement

Definitions: ATM means agent-team-mail and its tool crates. sc-compose crates are sc-compose, sc-composer and sc-composer-beads; sc-sha is published from the sc-compose repository but is not an sc-compose crate (ANC-061). The command contract edge is the dependency on sc-command with `default-features = false` and no features, permitted by ADR-RUN-0017. sc-storage is the shared `Storage<T>` crate and sc-monty is the shared Monty interpreter crate; both are workspace crates that REQ-RUN-0002 requires sc-cron to consume. The interpreter crates are sc-monty, monty, monty-types, monty-pool, monty-fs and monty-runtime. The default tree is the output of `cargo tree -p sc-cron -e normal`; the monty-feature tree is the same command with `--features monty`. sc-lint-boundary accepts workspace names only, so monty/monty-types are enforced by cargo-tree gates rather than entries in that list. Test and benchmark references are permitted; CLI process invocation creates no crate dependency. The `allowed_dependencies` list is in `boundaries/sc-cron/sc-cron.toml`. The production trees are the outputs of `cargo tree -p sc-cron -e normal,build` with default features and with `--features monty`. The schedule store has moved to Storage<CronRow> once `crates/sc-cron` no longer defines the `ScheduleStore` trait (code-alignment CRON-3). The third-party entries are the entries of the allowed_dependencies list that are not workspace crates. A workspace crate is a member of the workspace declared in the root `Cargo.toml`. The permitted workspace crates are sc-command, sc-storage and sc-monty. A production import is a `use` item or an `extern crate` item in the Rust code of `crates/sc-cron` outside test and benchmark code. Permitted interpreter crates in the monty-feature tree are sc-monty, monty and monty-types.

`rusqlite` leaves this list when the schedule store moves to `Storage<CronRow>`.

This item owns the third-party crates `sc-cron` may depend on. The `allowed_dependencies` list is the normal third-party dependencies: `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `rusqlite` (bundled), `sc-observability-types`, `sc-sha`, `serde`, `serde_json`, `tempfile`, `tokio`, and `tracing`. `sc-sha` supplies the SHA-256 compiled-session cache key. `sc-lint-boundary` accepts only workspace package names, so `monty` and `monty-types` are not entries in that list. Feature `monty` stays optional and adds the interpreter only through sc-monty. It is enforced by the `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml`: the default tree has no interpreter crate, and the only interpreter crates in the monty-feature tree are `sc-monty`, `monty` and `monty-types`, with `monty` and `monty-types` reached through `sc-monty`. `monty-pool`, `monty-fs`, and `monty-runtime` are not allowed. The workspace crates sc-cron may depend on are the command contract edge, sc-storage and sc-monty; no other workspace crate is allowed.

The tool-crate exclusion in REQ-RUN-0005 applies to sc-cron: sc-compose and ATM crates are forbidden in production dependencies/imports; test and benchmark references are permitted. CLI process invocation does not create a crate dependency.

**MUST statements:**

- Until the schedule store has moved to Storage<CronRow>, the third-party entries in `boundaries/sc-cron/sc-cron.toml` MUST be exactly async-trait, chrono, cron, getrandom, process-wrap, rusqlite (bundled), sc-observability-types, sc-sha, serde, serde_json, tempfile, tokio and tracing.
- When the schedule store has moved to Storage<CronRow>, the allowed_dependencies list in `boundaries/sc-cron/sc-cron.toml` MUST omit rusqlite.
- Feature monty MUST remain optional.
- The default tree MUST contain no interpreter crate.
- Every interpreter crate in the monty-feature tree MUST be a permitted interpreter crate.
- In the monty-feature tree, every path to monty MUST pass through sc-monty.
- In the monty-feature tree, every path to monty-types MUST pass through sc-monty.
- sc-cron-tree-default in sc-lint.toml MUST enforce the default tree.
- sc-cron-tree-monty in sc-lint.toml MUST enforce the monty-feature tree.
- The sc-command dependency in `crates/sc-cron/Cargo.toml` MUST be the command contract edge.

**MUST NOT statements:**

- Each production tree MUST NOT list a workspace crate other than sc-cron and the permitted workspace crates.
- sc-cron MUST NOT depend on monty-pool.
- sc-cron MUST NOT depend on monty-fs.
- sc-cron MUST NOT depend on monty-runtime.
- Each production tree MUST NOT list an sc-compose crate.
- Each production tree MUST NOT list an ATM crate.
- A production import MUST NOT name an sc-compose crate.
- A production import MUST NOT name an ATM crate.

### Rationale

Owner (ANC-031): "sc-compose and atm crates should be blocked from any production release crate.  sc-compose and atm can be referenced for tests i.e. benchmark tests" `monty` runs in-process when the host controls the code; `monty-pool` runs the interpreter in worker subprocesses for untrusted code. The Monty hook target is the trusted case, and a subprocess pool would spend the process spawn the target exists to avoid. A consumer who does not enable `monty` does not link the interpreter.
### Success Criteria

**Acceptance Criteria:**

- [ ] The `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml` enforce that feature `monty` stays optional: the default tree has no interpreter crate, and the only interpreter crates in the `--features monty` tree are `sc-monty`, `monty` and `monty-types`, with `monty` and `monty-types` reached through `sc-monty`.
### Related Documents

**Requirements:**

- [REQ-RUN-0005](../requirements.md) — point 4 points here

## NFR-CRON-0002: Environment isolation and redaction

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-28  
**Last Updated:** 2026-10-28  
**Owner:** randlee  

### Requirement Statement

Definitions: Allowlisted daemon values are process variables whose names are permitted by HookEnvironmentHost.allow_process. A non-Unicode value cannot be decoded as Unicode text. Diagnostic surfaces are configuration Debug, scheduled errors, recovery text, RunOutcome, tracing, canonical events and persisted stop reasons; trusted direct HookOutput fields remain the named exception. Stable metadata is fixed non-provider metadata and error/recovery text, rather than values copied from environment sources. A store connection is a database connection held for a storage operation. Monty is the embedded interpreter that runs sc-cron Monty hooks. ATM is the external `atm` executable. The atm_run adapter is the opt-in Monty host function that runs the ATM executable as a child process. A resolved environment snapshot is the immutable name-to-value map built once per occurrence from daemon values, ordered dotenv files and Set/Reference/Clear entries. An authorized dotenv path is a path whose symlink-resolved canonical form equals an administrator exact-file allowlist entry or lies under an administrator root allowlist entry. A failed reference-provider result is a missing, denied, unavailable or invalid result from EnvironmentReferenceResolver. A synchronous lock is a blocking lock guard such as a `std::sync::Mutex` guard. Resolution is the asynchronous construction of a resolved environment snapshot in `crates/sc-cron/src/environment/resolve.rs`. An await-spanning guard is a store connection or synchronous lock guard that is still in scope at an `.await` expression. A hook executable is the Python, sc-compose or ATM executable that a hook or host adapter starts. An environment value is a Set value, resolved value, dotenv file content or reference string. The environment size limits are 256 bytes per name, 16 `.env` sources, 128 entries, 16 KiB per stored or resolved value, 4 KiB per reference locator or path, a 32,000-byte encoded final block, a 256 KiB canonical document and a 1 MiB dotenv file. Execution is the running of one occurrence's hook with its resolved environment snapshot. The schema version 1 semantic check compares an existing sc-cron schedule table's columns, defaults, constraints and unique name index with the first-release schema. The hook configuration directory is the daemon current directory captured when hook or host configuration is built. Construction is the validation of a `HookEnvironment` value when it is built. Persistence is the encoding of a `HookEnvironment` into a schedule row's `hook_environment` column. Row decode is the conversion of a stored schedule row, including its `hook_environment` document, into a `Schedule`. `ChildEnv` is the environment a supervised child process starts with. `ChildSpec` is the program, arguments, working directory, input and environment of a supervised child process. `PythonHook` is a Python file run as a cron hook. A basename executable is a hook executable given as a file name with no directory component.

The implemented registry builder uses `register(name, handler)`,
`with_sc_compose(ScComposeHostConfig)`, and `with_atm(AtmHostConfig)`, each
returning `Result<MontyHostRegistry, CronError>`. The generated development
example and custom closure setup are in [monty-generated-example.md](monty-generated-example.md).
The b-16 child environment is explicitly empty; the resolved environment
snapshot contract in this record is the b-17 successor contract.

Shared hook environment implementation (B17):

`sc-cron::HookEnvironment` is the validated, canonical schedule policy;
`HookEnvironmentHost` is separately injected daemon authorization. Names use
portable ASCII identifiers and reject ASCII case collisions. Administrator
process-name, exact dotenv-file, and dotenv-root allowlists default empty.
Absolute normalized file paths are authorized after canonicalization, including
nearest-existing-ancestor resolution for missing suffixes. Bounded regular-file
reads run on the blocking pool. The dotenv parser never evaluates shell or
reads ambient values. Application-owned `EnvironmentReferenceResolver` receives
schedule, variable, opaque locator, and row deadline; missing, denied, unavailable
and invalid provider results use fixed value-free `ENV_REFERENCE` diagnostics.

One occurrence resolves daemon → ordered files → Set/Reference/Clear into an
immutable snapshot before stop, then reuses it for tick, Monty initialization,
and both opt-in CLI adapters. Python starts with `ChildEnv::Only`; direct trusted
callers can explicitly choose Inherit. Monty permits only environment OS reads,
including top-level reads, and denies registered application host calls during
initialization. Retained module globals initialize once per generation;
function-time getenv uses the current occurrence. Executables resolve before
child PATH clearing, independently of configured child working directories.
Windows snapshot names are uppercase and getenv lookup ignores case. Rust jobs
retain their application process environment and reject nonempty schedule policy.

Explicit Set values and opaque reference metadata are intentional plaintext
configuration. Resolved process/file/reference values never persist in registry
writes. Runtime details and reasons redact distinct nonempty snapshot values,
longest first, before caps and fixed diagnostic/recovery assembly. Debug exposes
names/kinds/counts only; trusted raw HookOutput fields remain accessible.
Exact replacement mitigates accidental disclosure and does not isolate hostile
source. The complete first-release SQLite schema is version 1 with canonical
`hook_environment` default `{"dotenv":[],"variables":{}}`; semantic schema checks
reject incompatible unreleased development tables without migrating/overwriting.
The library remains general purpose, with open application host registrations,
no signing/approval gates, and no production dependency on sc-compose or ATM.

**MUST statements:**

- Environment names MUST match `[A-Za-z_][A-Za-z0-9_]*`.
- Construction MUST enforce the environment size limits.
- Persistence MUST enforce the environment size limits.
- Row decode MUST enforce the environment size limits.
- Execution MUST enforce the environment size limits.
- Environment names MUST be UTF-8.
- Environment names MUST be NUL-free.
- Environment paths MUST be UTF-8.
- Environment paths MUST be NUL-free.
- Environment values MUST be UTF-8.
- Environment values MUST be NUL-free.
- Environment references MUST be UTF-8.
- Environment references MUST be NUL-free.
- ASCII case-fold name collisions MUST be rejected on every platform.
- The error code for an allowlisted daemon value that is non-Unicode MUST be `ENV_PROCESS_INVALID`.
- The error code for a dotenv file larger than 1 MiB MUST be `ENV_FILE_INVALID`.
- The error code for a dotenv file that is not valid UTF-8 MUST be `ENV_FILE_INVALID`.
- The error code for a dotenv source whose path is not an authorized dotenv path MUST be `ENV_FILE_DENIED`.
- The error code for a missing dotenv file whose path is not an authorized dotenv path MUST be `ENV_FILE_DENIED`.
- On Unix, resolved environment snapshot names MUST keep the letter case of the configured variable name.
- On Windows, resolved environment snapshot names passed to Python hooks MUST be uppercase.
- On Windows, resolved environment snapshot names passed to Monty hooks MUST be uppercase.
- A schedule read back from the store MUST keep the letter case of each configured variable name.
- Windows lookup MUST be case-insensitive.
- A basename executable MUST resolve to an absolute path through the daemon launch `PATH`.
- On Windows, the `PATH` search for a basename executable MUST also match file names that add an extension listed in the daemon launch `PATHEXT`.
- A hook executable given as an absolute path MUST be used unchanged.
- A hook executable given as a relative path with directory components MUST resolve against the hook configuration directory.
- The `Debug` text of a `ChildEnv` value MUST contain no environment value.
- The `Debug` text of a `ChildSpec` value MUST contain no environment value.
- The `Debug` text of a `PythonHook` value MUST contain no environment value.
- Resolution MUST read dotenv files inside `tokio::task::spawn_blocking`.
- Every failed reference-provider result MUST be reported with error code `ENV_REFERENCE`.
- Formatting an `ENV_REFERENCE` error MUST produce text that contains no environment value.
- Stop MUST use the same resolved environment snapshot.
- Tick MUST use the same resolved environment snapshot.
- Monty initialization MUST use the same resolved environment snapshot.
- The sc_compose_run adapter MUST use the same resolved environment snapshot.
- The atm_run adapter MUST receive the resolved environment snapshot of its occurrence.
- In SQLite schema version 1, a schedule row inserted without a `hook_environment` value MUST read back `hook_environment` as `{"dotenv":[],"variables":{}}`.
- Opening an existing sc-cron SQLite table that fails the schema version 1 semantic check MUST return an error.
- The error code for a dotenv file that contains a duplicate name MUST be `ENV_FILE_INVALID`.
- The error code for a dotenv file that contains NUL MUST be `ENV_FILE_INVALID`.
- The error code for a dotenv value that contains a backtick MUST be `ENV_FILE_INVALID`.
- The error code for a dotenv value that contains `$(` MUST be `ENV_FILE_INVALID`.
- The error code for a dotenv value that contains `$` followed by a letter, `_` or `{` MUST be `ENV_FILE_INVALID`.

**MUST NOT statements:**

- The `PATH` variable in the resolved environment snapshot MUST NOT change the resolved hook executable path.
- Opening an existing sc-cron SQLite table that fails the schema version 1 semantic check MUST NOT change the schema of the failing table.
- Opening an existing sc-cron SQLite table that fails the schema version 1 semantic check MUST NOT change the rows of the failing table.
- Set values MUST NOT be copied into diagnostic surfaces.
- Resolved values MUST NOT be copied into diagnostic surfaces.
- File contents MUST NOT be copied into diagnostic surfaces.
- Reference strings MUST NOT be copied into diagnostic surfaces.
- Stable metadata MUST NOT be rewritten by resolved-value redaction, including when it coincides with a resolved value.
- Resolution code MUST NOT contain an await-spanning store connection.
- Resolution MUST NOT create an await-spanning synchronous lock guard.
- Resolution MUST NOT call std::env::set_var.
- Resolution MUST NOT call std::env::remove_var.

### Rationale

ADR-RUN-0016: the Python hook inherited the complete daemon environment while Monty denied all OS calls. That is an inconsistent capability boundary and has caused difficult identity/team bugs when a daemon starts under one agent and later executes another agent's schedule. The library must support environment choices without silently capturing host values. Owner (ANC-037): "Never capture or persist inherited host values accidentally"
### Success Criteria

**Acceptance Criteria:**

- [ ] Tests make Python, Monty, both CLI adapter, and resolver paths echo canaries and prove redaction, positive delivery, and absence of a non-allowlisted daemon secret.

## Phase C approved target contract (planned, not implemented)

The following additive requirements describe the approved next release; they
do not claim current implementation or alter Phase B provenance. The existing
core remains free of product HTTP/MCP/CLI APIs. A separate optional standard
command adapter is permitted by ADR-RUN-0019.

## REQ-CRON-0020: Public receiver and cooperative owned shutdown

**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-13  
**Owner:** randlee  
**Status:** Draft  

### Requirement Statement

Definitions: sc-cron is the crate at `crates/sc-cron`, whose library name is `sc_cron`. The current revision is the commit under review. A public constructor is a `pub` associated function of a public `sc_cron` type that returns a value of that type. A signature is a function's name, generic parameters, parameter types and return type. A default body is a method body written in the trait definition, which an implementation may omit. The canonical receiver set is the `CanonicalLog` logging contract and its supporting values defined in `sc_command::observability`. A legacy store implementation is a `ScheduleStore` implementation that overrides neither `operation_support` nor `with_operation_context`, so `operation_support()` returns the default `StoreOperationSupport::Legacy`. A cooperative custom store is a `ScheduleStore` implementation outside sc-cron whose `operation_support()` returns `StoreOperationSupport::Cooperative`. Cooperative provider obligations are the statements of this record whose subject is a cooperative custom store. A persistence call is any read or write a store makes to its storage, including application-owned tables. Cooperative mode is the worker started by `Scheduler::spawn_worker_cooperative` together with the commands that receive its live operation scope. The live operation scope is the `StoreOperationContext` returned by `WorkerStopControl::operation_context()`, carrying the current stop, deadline and cancellation state. A live operation scope ends when it is cancelled or its deadline passes. A shortened deadline is a deadline that `WorkerStopControl::request_stop` moves earlier while an operation is already using the live operation scope. Waking an operation makes its wait return so the operation rechecks the live operation scope. Cooperative operation points are initial load, queue submission, lock acquisition, reload, decode, and every persistence, retry and end path of cooperative-mode work. G is the grace cutoff returned by `ShutdownBudget::grace()`, after which no ordinary commit is admitted. K is the cleanup cutoff returned by `ShutdownBudget::cleanup()`, after which no terminal-cleanup commit is admitted. A zero budget is a `ShutdownBudget` whose G and K both equal the instant `request_stop` is called. Ordinary work is any cooperative-mode operation other than terminal cleanup, and terminal cleanup is the generation-fenced end bookkeeping of the stopped schedule. The controller mutex is the single per-worker lock that guards stop state, earliest G and K, and commit admission decisions. The commit fence is the bundled `SqliteScheduleStore` commit hook that records `CommitAdmission::Admitted` or `CommitAdmission::Vetoed` for a write transaction. A bundled-store COMMIT runs from the moment the bundled store issues the SQLite COMMIT until that SQLite call returns. An explicit write is a transaction opened with BEGIN, and an implicit write is a single SQLite statement run in autocommit mode. Cooperative mode was added in sc-cron 0.2.0; the repository tag `v0.1.1` marks the release before it. A logging receiver trait is a trait with a method that takes a `LogEvent` from `sc_command::observability`. Expiry or drop of an operation confirms neither commit nor rollback. Uncertain connection state is a SQLite connection whose last transaction ended with `PersistenceOutcome::Indeterminate`. A stop cutoff check compares the current time with G or K. TLS means thread-local storage holding the SQLite operation context, not transport security. Restorable state is the SQLite operation hooks and TLS operation context of a bundled-store connection. The known baseline is the restorable state as it was before the operation started. A safe restoration path is any operation exit on which the connection's transaction outcome is known. A public item of a module is a type, trait, function, constant or module that the module exports with `pub`. A mutation is an insert, update or delete of stored rows. A readback is a read of schedule rows to reconcile an uncertain mutation. A hook run is one execution of a schedule's Rust, Python or Monty hook. An operation runs under the live operation scope when it receives the live operation scope and returns the scope's error once the scope ends. Owned work is each admitted worker or context-aware command task that `WorkerHandle::join_owned` must join. The waiter future of owned work is the future a caller awaits for that work to finish. Owned work physically completes when its worker or task returns.

This target contract is written against `ScheduleStore` and `SqliteScheduleStore`, which PRD-CRON-2 replaces with `Storage<CronRow>`. It has not been re-planned.

Scope: The `ScheduleStore` trait and generic `Scheduler` permit application-owned columns and tables through custom stores; preserving the bundled schema does not forbid consumer schemas or add product fields to `Schedule` or command DTOs. SQLite-specific callbacks apply only to the bundled `SqliteScheduleStore`. Legacy store implementations and direct legacy calls remain caller-owned with explicitly documented guarantees. Physical joins may take time; no zero-duration join, universal 2T completion or native/custom-host preemption is promised. Commit admission is a logical fence, not a physical commit/fsync cutoff. A bounded, separately authorized readback may reconcile observed state without claiming exactly-once identity. The bundled schema, generation fencing, stop-before-tick, existing Rust/Python/Monty hooks and the REQ-CRON-0019 environment policy are preserved.

**MUST statements:**

- `sc_cron::observability` MUST export the canonical receiver set.
- When a public constructor is present at tag `v0.1.1`, the constructor MUST have the same signature at the current revision.
- When a `ScheduleStore` method is absent at tag `v0.1.1`, the method MUST have a default body.
- Every public item of `sc_cron::observability` MUST be a `pub use` of an item from `sc_command::observability`.
- When called on a custom store that does not meet every cooperative provider obligation, `operation_support()` MUST return `StoreOperationSupport::Legacy`.
- Each persistence call a cooperative custom store makes under a cancelled live operation scope MUST fail with `StoreErrorKind::Cancelled`.
- When an end request names a generation other than the stored generation, a cooperative custom store MUST return `CronError::Fenced`.
- When an end request names a generation other than the stored generation, a cooperative custom store MUST NOT change the row.
- A cooperative custom store's `query` MUST return no more than `ScheduleQuery::limit` rows.
- A cooperative custom store's `query` MUST return rows in ascending id order.
- When `ScheduleQuery::after_id` is set, a cooperative custom store's `query` MUST return only rows whose id is greater than `after_id`.
- When a cooperative custom store write fails, `StoreError::transaction_outcome()` on the returned error MUST return the `PersistenceOutcome` that matches the stored state after the write.
- When the caller drops the future of an admitted operation, a cooperative custom store MUST continue the operation until the operation returns.
- When the store is a legacy store implementation, `Scheduler::spawn_worker_cooperative` MUST return the `operation_context_unsupported` error.
- Each cooperative operation point MUST run under the live operation scope.
- When a shortened deadline passes, a context check blocked on the live operation scope MUST return `StoreErrorKind::DeadlineExceeded`.
- When a shortened deadline passes during a bundled-store SQLite statement, the bundled store MUST interrupt the statement through the connection's `rusqlite::InterruptHandle`.
- After `WorkerStopControl::request_stop` returns, the scheduler MUST refuse each new ordinary operation with the `operation_stopping` error.
- When a later `request_stop` call passes a grace cutoff later than the recorded G, `WorkerStopControl` MUST keep the recorded G.
- When a later `request_stop` call passes a cleanup cutoff later than the recorded K, `WorkerStopControl` MUST keep the recorded K.
- While `WorkerHandle::join_owned` runs, each `request_stop` call MUST wake every operation waiting on the live operation scope.
- `ShutdownBudget::new` MUST take G as a `std::time::Instant` value.
- `ShutdownBudget::new` MUST take K as a `std::time::Instant` value.
- Each stop cutoff check MUST read the current time from `std::time::Instant::now()`.
- `ShutdownBudget::new(g, k)` MUST return `Ok` when `g` equals `k`.
- After `request_stop` with a zero budget returns, `StoreOperationContext::check()` on the live operation scope MUST return an error.
- When `request_stop` receives a zero budget, the commit fence MUST record `CommitAdmission::Vetoed` for each terminal-cleanup commit.
- When `request_stop` takes the controller mutex before the commit fence, the commit fence MUST record `CommitAdmission::Vetoed`.
- When the commit fence takes the controller mutex before `request_stop`, the commit fence MUST decide using the stop state from before the `request_stop` call.
- When the commit fence decides an ordinary commit at or after G, the commit fence MUST record `CommitAdmission::Vetoed`.
- When the commit fence decides a terminal-cleanup commit at or after K, the commit fence MUST record `CommitAdmission::Vetoed`.
- While the SQLite call of an admitted bundled-store COMMIT has not returned, a second `WorkerStopControl::request_stop` call MUST return.
- When cooperative-mode work makes an explicit write through the bundled store, `TransactionOutcome::admission` of the write MUST equal the `CommitAdmission` value the commit fence recorded for the write.
- When cooperative-mode work makes an implicit write through the bundled store, `TransactionOutcome::admission` of the write MUST equal the `CommitAdmission` value the commit fence recorded for the write.
- For a write the store confirms committed, `StoreError::transaction_outcome()` MUST return persistence `Committed`.
- When the store confirms that a commit of ordinary work completed after G, the `completed_after_cutoff` field returned by `StoreError::transaction_outcome()` MUST be true.
- When the store confirms that a terminal-cleanup commit completed after K, the `completed_after_cutoff` field returned by `StoreError::transaction_outcome()` MUST be true.
- For a write whose rollback the store confirms, `StoreError::transaction_outcome()` MUST return persistence `RolledBack`.
- When an operation attempted no mutation, `StoreError::transaction_outcome()` MUST return a `TransactionOutcome` whose persistence is `PersistenceOutcome::NoMutationAttempted`.
- When the store has confirmed neither commit nor rollback of a write, the reported `PersistenceOutcome` of the write MUST be `Indeterminate`.
- When a bundled-store operation exits on a safe restoration path, the restorable state MUST equal the known baseline.
- `ShutdownReport::cleanup_exceeded` returned by `WorkerHandle::join_owned` MUST be true exactly when terminal cleanup exceeded K.
- `ShutdownReport::cleanup` returned by `WorkerHandle::join_owned` MUST equal the `TransactionOutcome` of the terminal cleanup write.

**MUST NOT statements:**

- Source files under `crates/sc-cron/src` MUST NOT declare a logging receiver trait.
- When the store is a legacy store implementation, `Scheduler::spawn_worker_cooperative` MUST NOT run the job.
- A cooperative operation blocked acquiring a lock MUST NOT stay blocked after the live operation scope ends.
- Dropping the waiter future of owned work MUST NOT let `WorkerHandle::join_owned` return before the owned work physically completes.
- While a bundled-store COMMIT runs, the commit fence MUST NOT hold the controller mutex.
- The bundled store MUST NOT hand a connection in uncertain connection state to another operation.
- When a mutation's `PersistenceOutcome` is `Indeterminate`, sc-cron MUST NOT re-issue the mutation.
- When the live operation scope has ended, sc-cron MUST NOT start a readback under the live operation scope.
- Once `WorkerHandle::join_owned` has returned, owned work MUST NOT start a store write.
- Once `WorkerHandle::join_owned` has returned, owned work MUST NOT start a job run.
- Once `WorkerHandle::join_owned` has returned, owned work MUST NOT start a hook run.
- sc-cron MUST NOT use the `bool` value `false` to report an unknown persistence result.

### Rationale

Make the existing cron library consumable through its public logging facade and cooperative operation/lifecycle contract, preserving the already ported Rust, Python and optional Monty execution targets. The framework charter leaves application stores, schemas and wiring to the consumer. Owner (ANC-017): "it is not sc-runtime job to guarantee a customer can not misuse the core libraries."
### Success Criteria

**Acceptance Criteria:**

- [ ] The cron README and isolated external consumer prove one custom field and the chosen cooperative-or-legacy lifecycle through public scheduling/query/end APIs.
### Implementation Notes

Phase C approved target contract (planned, not implemented). Status is Draft since 2026-11-13 because PRD-CRON-2 stores the schedule table through `Storage<CronRow>`, so this record does not require `ScheduleStore` signatures to stay unchanged.

## REQ-CRON-0021: Tracked public operations and bounded schedule query

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: `sc-cron` is the crate in `crates/sc-cron`. The Scheduler is `sc_cron::Scheduler`. A public operation is one call of `Scheduler::start_with_context`, `Scheduler::stop_with_context` or `Scheduler::query_with_context`. Command capacity is the maximum number of public operations on one Scheduler that hold a permit at the same time. A permit is one command-capacity slot, taken when a public operation is admitted and released once when its store work physically completes. Physical completion is the return of the store work a public operation started, independent of whether the future awaiting the operation is still polled or has been dropped. The capacity refusal is `CronError::Store` with `StoreErrorKind::Unavailable` and reason `command_capacity_exhausted`. The fixed unsupported error is `CronError::Store` with `StoreErrorKind::Unavailable` and reason `schedule_query_unsupported`. A legacy store is a `ScheduleStore` implementation that does not override `ScheduleStore::query`. Admission is the single step of a public operation that checks stop, deadline, cancellation and a free permit before the operation registers or queues work. Atomic admission means no stop request, deadline expiry or cancellation can take effect between the admission check and the permit being taken. Registering work is inserting the operation into the Scheduler's operation registry; queuing work is submitting it to the blocking pool. A capacity-wait queue is any structure that holds a refused public operation until a permit frees. Automatic replay is re-running an admitted mutation without a new caller request. Private worker dispatch is the path a Scheduler worker uses for scheduled jobs and its own load, reload and bookkeeping store calls. Restricted internal cleanup is the terminal cleanup run by `WorkerStopControl` after stop is requested. A transport timeout is expiry of the caller-facing wait on a public operation; it does not complete the underlying store mutation. A blocking mutation is store write work queued by a public start or stop. `StoreOperationContext` is the `sc-cron` type carrying the shared stop, deadline and cancellation state passed to each operation. `ScheduleQuery` is the query input with fields `selection` (`Active`, `Ended` or `All`), `name: Option<ScheduleName>`, `after_id: Option<ScheduleId>` and `limit: u32`. A generation is the `Generation` value that identifies one incarnation of a schedule name; the active generation is the latest incarnation that has not ended. `CronOperations` is the public `sc-cron` trait with methods `start`, `end` and `query`. `CronOperationFuture<'a, T>` is `Pin<Box<dyn Future<Output = Result<T, CronError>> + Send + 'a>>`. `sc-cron-commands` is the command adapter crate in `crates/sc-cron-commands`. G is the grace deadline passed to `ShutdownBudget::new` as `grace`, and K is the cleanup deadline passed as `cleanup`. `WorkerHandle` is the `sc-cron` handle returned by `Scheduler::spawn_worker`; a framework worker is a worker started by `Scheduler::spawn_worker`. An application `CronOperations` implementation is a type outside this repository that implements `CronOperations`. A Tokio runtime is a `tokio::runtime::Runtime` executing async tasks. `test-utils` is the `sc-cron` Cargo feature that enables `ManualClock` and `SeqJitter` for tests. Dyn-compatible means usable as a `dyn` trait object type under the Rust reference's dyn-compatibility rules. A pre-shutdown context is a `StoreOperationContext` returned by `WorkerStopControl::operation_context()` before `WorkerStopControl::finish_application_shutdown()` is called. A post-shutdown context is a `StoreOperationContext` returned by `WorkerStopControl::operation_context()` after `WorkerStopControl::finish_application_shutdown()` returns.

**MUST statements:**

- The Scheduler MUST start with command capacity 64 when `Scheduler::with_command_capacity` is not called.
- When `Scheduler::with_command_capacity` receives a capacity outside 1..=65536, `Scheduler::with_command_capacity` MUST return an error.
- When every permit is held, a public operation MUST return the capacity refusal.
- A permit MUST stay held until physical completion of its public operation after the future awaiting that operation is dropped.
- Admission MUST be atomic.
- `Scheduler::start_with_context` MUST have the signature `async fn start_with_context(&self, spec: ScheduleSpec, context: StoreOperationContext) -> Result<Schedule, CronError>`.
- `Scheduler::stop_with_context` MUST have the signature `async fn stop_with_context(&self, id: ScheduleId, generation: Generation, reason: String, context: StoreOperationContext) -> Result<Schedule, CronError>`.
- When the given `Generation` is not the active generation of the schedule with the given `ScheduleId`, `Scheduler::stop_with_context` MUST return `CronError::Fenced`.
- `Scheduler::query_with_context` MUST have the signature `async fn query_with_context(&self, query: ScheduleQuery, context: StoreOperationContext) -> Result<SchedulePage, CronError>`.
- When `ScheduleQuery::selection` is `Active`, `Scheduler::query_with_context` MUST return only rows of schedules that have not ended.
- When `ScheduleQuery::selection` is `Ended`, `Scheduler::query_with_context` MUST return only rows of ended schedules.
- When `ScheduleQuery::selection` is `All`, `Scheduler::query_with_context` MUST return rows without filtering on whether a schedule has ended.
- When `ScheduleQuery::name` is set, `Scheduler::query_with_context` MUST return only rows with that `ScheduleName`.
- `Scheduler::query_with_context` MUST return rows in ascending `ScheduleId` order.
- When `ScheduleQuery::after_id` is set, `Scheduler::query_with_context` MUST return only rows whose `ScheduleId` is greater than `after_id`.
- When `ScheduleQuery::limit` is outside 1..=1000, `Scheduler::query_with_context` MUST return an error.
- `Scheduler::query_with_context` MUST return at most `ScheduleQuery::limit` rows.
- `Scheduler::query_with_context` MUST read the existing schedule table.
- When called on a legacy store, `ScheduleStore::query` MUST return the fixed unsupported error.
- When `CronOperations::end` returns `Ok`, `EndedSchedule::id` MUST equal the id of the schedule row with name `EndSchedule::name` and generation `EndSchedule::generation`.
- When `CronOperations::end` returns `Ok`, `EndedSchedule::generation` MUST equal `EndSchedule::generation`.
- The `CronOperations` trait definition in `crates/sc-cron` MUST be dyn-compatible.
- `CronOperations::start` MUST take a `StoreOperationContext` argument.
- `CronOperations::start` MUST return `CronOperationFuture<'_, Schedule>`.
- `CronOperations::end` MUST take a `StoreOperationContext` argument.
- `CronOperations::end` MUST return `CronOperationFuture<'_, EndedSchedule>`.
- `CronOperations::query` MUST take a `StoreOperationContext` argument.
- `CronOperations::query` MUST return `CronOperationFuture<'_, SchedulePage>`.
- When stop is requested on a parent `StoreOperationContext`, `stop_requested()` on a child returned by the parent's `with_deadline` MUST return true.
- When `StoreOperationContext::with_deadline(deadline)` is called, `deadline()` on the returned child MUST return the earlier of the parent deadline and `deadline`.
- When called inside a Tokio runtime, `WorkerStopControl::new()` MUST return `Ok`.
- When `WorkerStopControl::new()` has returned `Ok`, `check()` on a `StoreOperationContext` from `WorkerStopControl::operation_context()` MUST return `Ok`.
- When no Scheduler exists, `WorkerStopControl::new()` called inside a Tokio runtime MUST return `Ok`.
- When no framework worker is running, `WorkerStopControl::new()` called inside a Tokio runtime MUST return `Ok`.
- An application `CronOperations` implementation MUST execute the store work of calls made to the implementation.
- Before calling `WorkerStopControl::finish_application_shutdown()`, an application MUST join all store work started by the application's `CronOperations` implementation.
- After `WorkerStopControl::finish_application_shutdown()` returns `Ok`, `check()` on a pre-shutdown context MUST return an error.
- After `WorkerStopControl::finish_application_shutdown()` returns `Ok`, `check()` on a post-shutdown context MUST return an error.
- `ShutdownBudget` MUST have only private fields.
- `ShutdownBudget::grace()` MUST return the `grace` value passed to `ShutdownBudget::new`.
- `ShutdownBudget::cleanup()` MUST return the `cleanup` value passed to `ShutdownBudget::new`.
- `ShutdownBudget::new` MUST return an error when K is earlier than G.
- Joins of framework workers MUST go through `WorkerHandle`.

**MUST NOT statements:**

- When every permit is held, a refused public operation MUST NOT register work.
- When every permit is held, a refused public operation MUST NOT queue work.
- `WorkerStopControl::new()` MUST NOT be gated by the `test-utils` feature.
- `ShutdownBudget` MUST NOT have a method that takes `&mut self`.
- The Scheduler MUST NOT hold a refused public operation in a capacity-wait queue.
- The Scheduler MUST NOT perform automatic replay of an admitted mutation.
- When every permit is held, private worker dispatch MUST NOT perform store work for a refused public operation.
- When every permit is held, restricted internal cleanup MUST NOT perform store work for a refused public operation.
- A transport timeout MUST NOT release the permit of a blocking mutation before physical completion.
- When `EndSchedule::generation` is older than the active generation of the schedule named `EndSchedule::name`, `CronOperations::end` MUST NOT change the schedule row of the active generation.
- `CronOperations` method signatures MUST NOT contain a type defined in `sc-cron-commands`.
- Calls made with a child returned by `StoreOperationContext::with_deadline` MUST NOT take a permit in addition to the parent's permit.
- `StoreOperationContext::with_deadline` MUST NOT create a registry separate from the parent's registry.
- While a receiver returned by `WorkerStopControl::subscribe_status()` has not been dropped, an application MUST NOT shut down the Tokio runtime in which `WorkerStopControl::new()` was called.

### Rationale

This completes the existing pre-submission admission/physical-ownership promise of REQ-CRON-0020/0021 and ADR-RUN-0018 under NFR-RUN-0002's nonblocking request path and the existing service-hardening production checklist section 7.
### Success Criteria

**Acceptance Criteria:**

- [ ] Real query returns ended as well as active rows with deterministic bounded continuation; malformed limits/cursors fail; replacement races preserve generation fencing.
- [ ] Cancellation of command waiters leaves no untracked work.
### Implementation Notes

Phase C approved target contract (planned, not implemented).

PRD-CRON-2 replaces the legacy ScheduleStore contract with Storage<CronRow>. References to ScheduleStore in this record describe the legacy interface, not a requirement to retain that trait. Capacity, admission, generation fencing and bounded-query behavior remain obligations of the scheduler using the shared storage contract.

Scope: Standard command execution requires these operations. `list_active` alone does not satisfy the query obligations. Command capacity does not change embedder row limits. Command capacity and admission do not apply to application `CronOperations` or `CronExecutor` implementations, which own their own intake. No new job kind, job registry, second database or arbitrary hard-delete command is implied. Ordinary registration continues through REQ-CRON-0003 APIs. The Phase C compatibility contract defines exact configuration, reason mapping and causal proof owners; the Scheduler implementation belongs to sprint c-5.
