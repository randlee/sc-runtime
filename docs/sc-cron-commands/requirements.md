# sc-cron-commands requirements

Status: approved Phase C target, planned and unimplemented. This optional
library reconciles the new human command requirement with the scheduler core's
existing no-product-API rule. It is not an ATM implementation or dependency.

## REQ-CRNC-0001: One standard versioned cron command contract

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

Definitions: DTO means data transfer object; JSON means JavaScript Object Notation; CLI means command-line interface; MCP means Model Context Protocol; HTTP means Hypertext Transfer Protocol; SQL means Structured Query Language; ATM means agent-team-mail. A strict version-1 DTO is the request representation whose version and fields are validated before writes. Envelope and OpError are the existing sc-command result and error types. A Monty target runs the optional Monty interpreter; Rust targets use the application Job or dispatcher. Opaque payload/environment are the application JSON data and hook-environment configuration carried by the request. Bounded timeout uses the core RunTimeout type and its existing maximum, MAX_TIMEOUT of 24 hours. List continuation is ascending stable row-id pagination with a positive capped result limit and active/ended/all selection. A cursor is the prior page's exclusive row-id continuation value. Overflow means a numeric value not representable by its declared field type. An unsupported target is a target unavailable in the selected build or application bindings. Core-derived limits are the existing scheduler input limits frozen by the contract sprint. Value-free error mapping uses stable typed codes rather than raw SQL, environment, source or payload. Sensitive read fields are hook source, environment and payload. A serialized function would carry executable Rust function data in a request; a job catalog would add another job registration system. The five-field grammar is the ordinary cron syntax; supported aliases remain accepted. A read view is the command response exposing schedule-row data. A separately authorized application projection is an application-owned response representation explicitly permitted to expose the named sensitive fields; this record specifies no authorization mechanism.

Provide `cron.start`, `cron.end` and `cron.list` through strict version-1 JSON
DTOs and existing sc-command Envelope/OpError. CLI, REST and MCP projections
use the same request, typed result and namespaced error codes. Start selects
existing Rust, Python or optional Monty targets, ordinary five-field cron or
supported aliases, bounded timeout and opaque payload/environment. Rust uses
the application-configured Job/dispatcher; no serialized function or new job
catalog. End requires name and generation. List supports bounded deterministic
active/ended/all continuation with id/generation/state views. No implicit
end-all, hard deletion, ATM team/phase/mandatory-until or six-field-only grammar.

Reject unknown/duplicate fields, invalid version/generation/limits/cursor,
overflow and unsupported target before writes. Freeze exact core-derived size
limits and value-free error mapping in the contract sprint. Do not expose raw
SQL, environment values, source code or payload in diagnostics. Read views
exclude sensitive source/environment/payload unless a separate authorized
application projection explicitly exposes them.

**MUST statements:**

- `cron.start` MUST use a strict version-1 JSON DTO.
- `cron.end` MUST use a strict version-1 JSON DTO.
- `cron.list` MUST use a strict version-1 JSON DTO.
- Command results MUST use the existing sc-command Envelope/OpError.
- CLI, REST and MCP projections MUST use the same request.
- CLI, REST and MCP projections MUST use the same typed result.
- CLI, REST and MCP projections MUST use the same namespaced error codes.
- Start MUST support existing Rust targets.
- Start MUST support existing Python targets.
- Start MUST support optional Monty targets.
- Start MUST support ordinary five-field cron or supported aliases.
- Start MUST support a bounded timeout.
- Start MUST support opaque payload/environment.
- Rust commands MUST use the application-configured Job or dispatcher.
- End MUST require name.
- End MUST require generation.
- List MUST support bounded deterministic continuation for active schedules.
- List MUST support bounded deterministic continuation for ended schedules.
- List MUST support bounded deterministic continuation for all schedules.
- List views MUST expose id.
- List views MUST expose generation.
- List views MUST expose state.
- Unknown fields MUST be rejected before writes.
- Duplicate fields MUST be rejected before writes.
- Invalid version MUST be rejected before writes.
- Invalid generation MUST be rejected before writes.
- Invalid limits MUST be rejected before writes.
- Invalid cursor MUST be rejected before writes.
- Overflow MUST be rejected before writes.
- Unsupported targets MUST be rejected before writes.
- The contract sprint MUST freeze core-derived size limits.
- The contract sprint MUST freeze value-free error mapping.
- Read views MUST exclude sensitive source unless a separately authorized application projection explicitly exposes that source.
- Read views MUST exclude environment unless a separately authorized application projection explicitly exposes that environment.
- Read views MUST exclude payload unless a separately authorized application projection explicitly exposes that payload.

**MUST NOT statements:**

- The command contract MUST NOT add a serialized function.
- The command contract MUST NOT add a new job catalog.
- Diagnostics MUST NOT expose raw SQL.
- Diagnostics MUST NOT expose environment values.
- Diagnostics MUST NOT expose source code.
- Diagnostics MUST NOT expose payload.
- The contract MUST NOT introduce implicit end-all.
- The contract MUST NOT introduce hard deletion.
- The contract MUST NOT introduce ATM team/phase/mandatory-until.
- The contract MUST NOT introduce six-field-only grammar.

### Rationale

This optional library reconciles the new human command requirement with the scheduler core's existing no-product-API rule.

### Success Criteria

**Acceptance Criteria:**

- [ ] Serialization/schema tests demonstrate one canonical versioned start/end/list and Envelope result/error across projection-neutral fixtures.
- [ ] All invalid forms fail before scheduler/store work; no absent name means end-all.

### Implementation Notes

Phase C target, planned and unimplemented.

## REQ-CRNC-0002: Optional executor and actual opt-in UI wiring

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: Unsupported Monty means a Monty target requested without the optional Monty capability enabled in the selected build. CLI means command-line interface. MCP means Model Context Protocol. Standard validation is the version, field, target, limit and cursor validation in REQ-CRNC-0001. Scope propagation passes the live StoreOperationContext into the native operation. Selected generation means cron_commands=true with cron selected. The core is sc-cron's scheduler API, usable without sc-cron-commands. A CLI end projection is the generated end command forwarding its typed request to the same CronExecutor as the other selected command surfaces. The adapter is the `sc-cron-commands` crate. `CronExecutor` is the object-safe executor trait exported by the adapter with default features. `StandardCronExecutor` is the executor provided under its non-default scheduler feature; the standard executor and standard implementation mean it. `CronOperations` is sc-cron's native operations port ([REQ-CRON-0021](../sc-cron/requirements.md)).

Default-surface object-safe CronExecutor accepts sole CronRequest and real
deadline. Scheduler-feature StandardCronExecutor consumes native CronOperations
and explicit ApplicationTargetBindings ports defined in the Phase C contract;
it derives a child operation context, never a separate timeout/ownership policy
inside the standard executor. Applications may implement CronOperations or
CronExecutor through these public seams for different behavior or shutdown
policy; their task/physical-join ownership, persistence truth, authorization
and error mapping responsibilities must be documented rather than inheriting
the bundled cooperative guarantee. Standard validation and scope propagation
remain mandatory for the standard implementation.
Unsupported Monty fails before persistence.
Selected generator configuration installs actual CLI start/end/list, HTTP and
optional MCP projections calling this same executor; a recommended example or
DTO-only library does not satisfy selected wiring. `cron_commands=false` is
the generation default; `true` requires cron selected and does not enable MCP
when MCP is disabled. No production success/unsupported stub satisfies this requirement. Existing
applications can use the core with no commands. The existing adapter README
provides one working selected-default recipe with bundled store, explicit
worker/target registration and startup, commands, and stop/join/outcome handling;
it links the core custom-store example and describes supported replacement.
No consumer-product migration or automatic job start is implied.

**MUST statements:**

- The default-surface object-safe CronExecutor MUST accept the sole CronRequest type.
- The default-surface object-safe CronExecutor MUST accept a real deadline.
- Scheduler-feature StandardCronExecutor MUST consume native CronOperations.
- Scheduler-feature StandardCronExecutor MUST consume explicit ApplicationTargetBindings.
- The standard executor MUST derive a child operation context.
- Custom CronOperations or CronExecutor implementations MUST document task/physical-join ownership responsibilities.
- Custom CronOperations or CronExecutor implementations MUST document persistence truth responsibilities.
- Custom CronOperations or CronExecutor implementations MUST document authorization responsibilities.
- Custom CronOperations or CronExecutor implementations MUST document error mapping responsibilities.
- The standard implementation MUST retain standard validation.
- The standard implementation MUST retain scope propagation.
- Unsupported Monty MUST fail before persistence.
- Selected generation MUST install an actual CLI start projection calling that same executor.
- Selected generation MUST install an actual CLI end projection calling that same executor.
- Selected generation MUST install an actual CLI list projection calling that same executor.
- Selected generation MUST install actual HTTP projections calling that same executor.
- Selected generation MUST install actual optional MCP projections calling that same executor.
- cron_commands MUST default to false.
- Selecting cron_commands MUST require cron.
- Applications MUST be able to use the core without commands.
- The adapter README MUST provide a working selected-default recipe.
- That recipe MUST demonstrate the bundled store.
- That recipe MUST demonstrate explicit worker/target registration.
- That recipe MUST demonstrate startup.
- That recipe MUST demonstrate commands.
- That recipe MUST demonstrate stop handling.
- That recipe MUST demonstrate join handling.
- That recipe MUST demonstrate outcome handling.
- The adapter README MUST link the core custom-store example.
- The adapter README MUST describe supported replacement.

**MUST NOT statements:**

- The standard executor MUST NOT introduce a separate timeout policy.
- The standard executor MUST NOT introduce a separate ownership policy.
- A recommended example MUST NOT satisfy selected generation.
- A DTO-only library MUST NOT satisfy selected generation.
- A production success stub MUST NOT satisfy selected generation.
- A production unsupported stub MUST NOT satisfy selected generation.
- Selecting cron_commands MUST NOT enable MCP when MCP is disabled.

### Rationale

Owner (ANC-014): "it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface." (ANC-018): "it is sc-runtime job to provide an easy documented path that consumers will not accidently misuse the core libraries."

### Success Criteria

**Acceptance Criteria:**

- [ ] The generated-consumer and registry-only proofs consume that recipe using public APIs and documented selection, without manual framework-source edits.

### Implementation Notes

Phase C target, planned and unimplemented.

C-5 implements operations for real Scheduler; c-6 implements the executor with dev-only operation doubles. C-3 prepares injectable projections with test-only executor doubles; c-4 supplies real selected root binding after real c-5/c-6 artifacts.

## NFR-CRNC-0001: Minimal independent dependency graphs

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

Definitions: DTO means data transfer object. CLI means command-line interface. The adapter is the `sc-cron-commands` crate at `crates/sc-cron-commands`. The adapter manifest is `crates/sc-cron-commands/Cargo.toml`. The cron DTO family is the set of public structs and enums in `crates/sc-cron-commands/src/contract.rs`. The DTO dependency set is `serde`, `serde_json` and `sc-command` with `default-features = false`. The schema traits are `schemars::JsonSchema` and `utoipa::ToSchema`. The schema dependencies are the `schemars` and `utoipa` entries of the adapter manifest. The envelope/error model is the `sc-command` types `Envelope<T>`, `OpError` and `ErrorKind`. `CronResponse` is the cron response enum in the cron DTO family. The `server` cargo feature of `sc-command` adds `axum` and `rmcp`. clap is the Rust command-line argument parser crate `clap`. Axum is the Rust HTTP framework crate `axum`. RMCP is the Model Context Protocol crate `rmcp`. The default tree command is `cargo tree -p sc-cron-commands -e normal --prefix none`. The schema tree command is `cargo tree -p sc-cron-commands -e normal --prefix none --features schema`. The default feature query is `cargo tree -p sc-cron-commands -e features --prefix none -i sc-command`. The schema feature query is `cargo tree -p sc-cron-commands -e features --prefix none -i sc-command --features schema`. A generated project with cron commands is a project generated with the `cron_commands` option set to true ([REQ-CRNC-0002](#req-crnc-0002-optional-executor-and-actual-opt-in-ui-wiring)). The CLI tree command is `cargo tree -p <project>-cli -e normal --prefix none`, run in a generated project with cron commands ([NFR-RUN-0001](../requirements.md)). A schema matches the JSON emitted by a `Serialize` implementation when every JSON value that implementation emits validates against the schema and the schema names exactly the fields and variants that implementation emits. A copy of a type is a separate struct or enum with the same serde fields or variants. A generic schema is a schema that accepts any JSON value, such as the schema of `serde_json::Value`.

**MUST statements:**

- The non-optional `[dependencies]` entries of the adapter manifest MUST be exactly the DTO dependency set.
- In the adapter manifest, the `scheduler` feature list MUST be exactly `["dep:sc-cron"]`.
- The adapter manifest MUST enable `sc-cron/monty` only from the `monty` feature.
- With the `schema` feature enabled, `schemars::schema_for!` MUST return a schema for every type in the cron DTO family.
- With the `schema` feature enabled, `utoipa::PartialSchema::schema` MUST return a schema for each type in the cron DTO family.
- Each schema dependency MUST be declared with `optional = true` in the adapter manifest.
- In a generated project with cron commands, the `sc-cron-commands` dependency entry in `crates/daemon/Cargo.toml` MUST list the `schema` feature.
- In a generated project with cron commands, the `sc-cron-commands` dependency entry in `crates/service/Cargo.toml` MUST list the `schema` feature.
- Every schema-trait implementation for `Envelope<T>` MUST be located in `crates/sc-command/src/`.
- Every schema-trait implementation for `OpError` MUST be located in `crates/sc-command/src/`.
- Every schema-trait implementation for `ErrorKind` MUST be located in `crates/sc-command/src/`.
- The schema returned by `schemars::schema_for!(Envelope<T>)` MUST match the JSON emitted by the `Serialize` implementation of `Envelope<T>`.
- The schema returned by `<Envelope<T> as utoipa::PartialSchema>::schema()` MUST match the JSON emitted by the `Serialize` implementation of `Envelope<T>`.
- The schema returned by `schemars::schema_for!(OpError)` MUST match the JSON emitted by the `Serialize` implementation of `OpError`.
- The schema returned by `<OpError as utoipa::PartialSchema>::schema()` MUST match the JSON emitted by the `Serialize` implementation of `OpError`.
- The schema returned by `schemars::schema_for!(ErrorKind)` MUST match the JSON emitted by the `Serialize` implementation of `ErrorKind`.
- The schema returned by `<ErrorKind as utoipa::PartialSchema>::schema()` MUST match the JSON emitted by the `Serialize` implementation of `ErrorKind`.
- With the `schema` feature enabled, `schemars::schema_for!(Envelope<CronResponse>)` MUST return a schema.
- With the `schema` feature enabled, `<Envelope<CronResponse> as utoipa::PartialSchema>::schema()` MUST return a schema.
- Piping the schema tree command into `grep -o '^schemars v[0-9.]*' | sort -u | wc -l` MUST print 1.
- Piping the schema tree command into `grep -o '^utoipa v[0-9.]*' | sort -u | wc -l` MUST print 1.
- Piping the default tree command into `grep '^sc-cron v'` MUST exit with status 1.
- Piping the default tree command into `grep sqlite` MUST exit with status 1.
- Piping the default tree command into `grep '^tokio v'` MUST exit with status 1.
- Piping the default tree command into `grep monty` MUST exit with status 1.
- Piping the default tree command into `grep '^clap v'` MUST exit with status 1.
- Piping the default tree command into `grep '^axum v'` MUST exit with status 1.
- Piping the default tree command into `grep '^rmcp v'` MUST exit with status 1.
- Piping the default feature query into `grep -F 'sc-command feature "server"'` MUST exit with status 1.
- Piping the CLI tree command into `grep '^sc-cron v'` MUST exit with status 1.
- In a generated project with cron commands, piping `cargo tree -p <project>-cli -e features --prefix none -i sc-command` into `grep -F 'sc-command feature "server"'` MUST exit with status 1.
- Piping the schema tree command into `grep '^sc-cron v'` MUST exit with status 1.
- Piping the schema tree command into `grep sqlite` MUST exit with status 1.
- Piping the schema tree command into `grep '^tokio v'` MUST exit with status 1.
- Piping the schema tree command into `grep monty` MUST exit with status 1.
- Piping the schema tree command into `grep '^axum v'` MUST exit with status 1.
- Piping the schema tree command into `grep '^rmcp v'` MUST exit with status 1.
- Piping the schema feature query into `grep -F 'sc-command feature "server"'` MUST exit with status 1.
- In a generated project with cron commands, piping `cargo tree -p <project>-cli -e features --prefix none -i sc-cron-commands` into `grep -F 'sc-cron-commands feature "schema"'` MUST exit with status 1.
- Piping `cargo tree -p sc-command -e normal --prefix none` into `grep '^sc-cron v'` MUST exit with status 1.
- Piping `cargo tree -p sc-command -e normal --prefix none` into `grep '^sc-cron-commands v'` MUST exit with status 1.
- Piping `cargo tree -p sc-runtime -e normal --prefix none` into `grep '^sc-cron v'` MUST exit with status 1.
- Piping `cargo tree -p sc-runtime -e normal --prefix none` into `grep '^sc-cron-commands v'` MUST exit with status 1.

**MUST NOT statements:**

- In the adapter manifest, the `schema` feature MUST NOT enable the `scheduler` feature.
- In the adapter manifest, the `scheduler` feature MUST NOT enable the `schema` feature.
- The `default` feature list in the adapter manifest MUST NOT contain `schema`.
- The `default` feature list in the adapter manifest MUST NOT contain `scheduler`.
- A crate other than `sc-cron-commands` MUST NOT implement a schema trait for a type in the cron DTO family.
- A crate other than `sc-cron-commands` MUST NOT define a copy of a type in the cron DTO family.
- `schemars::schema_for!(CronRequest)` MUST NOT return a generic schema.
- `<CronRequest as utoipa::PartialSchema>::schema()` MUST NOT return a generic schema.
- A crate other than `sc-command` MUST NOT define a copy of a type in the envelope/error model.
- `schemars::schema_for!(Envelope<CronResponse>)` MUST NOT return a generic schema.
- `<Envelope<CronResponse> as utoipa::PartialSchema>::schema()` MUST NOT return a generic schema.

### Rationale

UI CLI clients share DTOs without linking scheduler/server. Keep CLI lean: default dependency graphs stay free of server stacks (ADR-RUN-0003); agents and scripts depend on a small CLI binary.

### Success Criteria

**Acceptance Criteria:**

- [ ] C-3 checks real typed projections.
- [ ] c-6 preserves/tests all feature combinations.
- [ ] c-4 compares actual tools/list and OpenAPI schemas in its existing composition fixture.

### Implementation Notes

Phase C target, planned and unimplemented.

c-1 freezes the exact compatible schemars and utoipa identities with ADR-RUN-0203's real Parameters<T> and Utoipa probe, then commits compileable declarations, implementations and the feature/boundary baseline before c-3/c-6 parallel work.
