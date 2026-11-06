# sc-cron-commands requirements

Status: approved Phase C target, planned and unimplemented. This optional
library reconciles the new human command requirement with the scheduler core's
existing no-product-API rule. It is not an ATM implementation or dependency.

## REQ-CRNC-0001: One standard versioned cron command contract

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

## REQ-CRNC-0002: Optional executor and actual opt-in UI wiring

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
C-5 implements operations for real Scheduler; c-6 implements the executor with
dev-only operation doubles. Unsupported Monty fails before persistence.
Selected generator configuration installs actual CLI start/end/list, HTTP and
optional MCP projections calling this same executor; a recommended example or
DTO-only library does not satisfy selected wiring. `cron_commands=false` is
the generation default; `true` requires cron selected and does not enable MCP
when MCP is disabled. C-3 prepares injectable projections with test-only executor
doubles; c-4 supplies real selected root binding after real c-5/c-6 artifacts.
No production success/unsupported stub satisfies this requirement. Existing
applications can use the core with no commands. The existing adapter README
provides one working selected-default recipe with bundled store, explicit
worker/target registration and startup, commands, and stop/join/outcome handling;
it links the core custom-store example and describes supported replacement.
The generated-consumer and registry-only proofs consume that recipe using
public APIs and documented selection, without manual framework-source edits.
No consumer-product migration or automatic job start is implied.

## NFR-CRNC-0001: Minimal independent dependency graphs

DTO-only defaults depend on serde/serde_json and defaults-off sc-command;
exclude cron, SQLite, Tokio, Monty, clap, Axum, RMCP and command/server. The
non-default scheduler feature adds only approved cron/async edges; Monty remains
separately explicit. No sc-command→cron/adapter cycle or sc-runtime→cron/adapter
edge is allowed. UI CLI clients share DTOs without linking scheduler/server.

Optional `schema = ["dep:schemars", "dep:utoipa", "sc-command/schema"]` supplies JsonSchema and
ToSchema on the same adapter-owned DTO family. Dependencies remain optional;
c-1 freezes exact compatible identities with ADR-RUN-0203's real Parameters<T>
and Utoipa probe, then commits compileable declarations/impls and feature/boundary
baseline before c-3/c-6 parallel work. Schema is independent of scheduler and
adds no cron/SQLite/Tokio/Monty/Axum/RMCP/command-server edge. Defaults activate
neither feature. Selected daemon/service projections enable schema on their
own edges; isolated CLI defaults do not. No foreign orphan impl, second wire DTO
or generic schema substitutes for the typed canonical contract. C-3 checks real
typed projections; c-6 preserves/tests all feature combinations; c-4 compares
actual tools/list and OpenAPI schemas in its existing composition fixture.
The command schema feature supplies canonical owner-local Envelope<T>, OpError
and ErrorKind traits matching their existing custom serialization exactly.
Typed Envelope<CronResponse> and all nested schema types compile under the
same identities; no copied envelope/error model or generic response schema.
