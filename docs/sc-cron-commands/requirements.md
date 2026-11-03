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
it derives a child operation context, never a separate timeout/ownership policy.
C-5 implements operations for real Scheduler; c-6 implements the executor with
dev-only operation doubles. Unsupported Monty fails before persistence.
Selected generator configuration installs actual CLI start/end/list, HTTP and
optional MCP projections calling this same executor; a recommended example or
DTO-only library does not satisfy selected wiring. `cron_commands=false` is
the generation default; `true` requires cron selected and does not enable MCP
when MCP is disabled. C-3 prepares injectable projections with test-only executor
doubles; c-4 supplies real selected root binding after real c-5/c-6 artifacts.
No production success/unsupported stub satisfies this requirement. Existing
applications can use the core with no commands.

## NFR-CRNC-0001: Minimal independent dependency graphs

DTO-only defaults depend on serde/serde_json and defaults-off sc-command;
exclude cron, SQLite, Tokio, Monty, clap, Axum, RMCP and command/server. The
non-default scheduler feature adds only approved cron/async edges; Monty remains
separately explicit. No sc-command→cron/adapter cycle or sc-runtime→cron/adapter
edge is allowed. UI CLI clients share DTOs without linking scheduler/server.
