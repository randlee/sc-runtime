# sc-dolt requirements

**ID Range:** REQ-DOLT-0001 through REQ-DOLT-0006  
**Status:** Proposed (planning only; on hold per [sc-dolt plan](../plans/maintenance/sc-dolt-plan.md))  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-11  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for the proposed `sc-dolt` library. Repo-level
requirements are in [`../requirements.md`](../requirements.md). Architecture
and ADRs are in [`architecture.md`](architecture.md). Entries follow the shared
SC requirement template. **These requirements do not authorize implementation or
release.**

Raptor is the first intended consumer application, not the API namespace.

## Purpose

`sc-dolt` is a thin, reusable Dolt access crate: explicit branch-scoped SQL
execution, native branch/merge/remote operations, and typed outcomes. Consumer
applications own domain schema, migrations, HTTP/MCP surfaces, and snapshot
export policy.

---

## REQ-DOLT-0001: explicit branch-scoped execution

**Status:** Proposed  

### Requirement Statement

The crate `sc-dolt` (`crates/sc-dolt`, planned) MUST expose branch-scoped
execution only through validated `DatabaseName` and `BranchName` values (or
equivalent validated newtypes documented in
[architecture](architecture.md)). Every mutating or read operation MUST carry
an explicit `(database, branch)` target.

The library MUST NOT silently default to `main`, MUST NOT share a process-global
current branch, and MUST NOT hand consumers an unscoped application-wide pool.
Concurrent clients targeting different branches MUST remain isolated when
connection capacity is reused, including distinct credential identity/version
per pool key.

The public SQL seam is a `BranchLease` (or successor name) that owns one
connection for the duration of a consumer operation. Consumers own table/record
SQL composed against that lease. Application HTTP/MCP policy is outside this
crate ([NFR-RUN-0004](../requirements.md)).

### Rationale

Dolt branch state is SQL-session state. Pool reuse without explicit targets
routes statements to the wrong branch. A dedicated library removes duplicated
session and native-procedure handling in each consumer.

### Success Criteria

1. Plan architecture names the public newtypes and `BranchLease` without
   exporting `sqlx` pool, connection, or transaction types
   ([ADR-RUN-0001](../architecture.md) amendment).
2. A future integration test opens two simultaneous leases on different
   branches and proves CRUD stays scoped.
3. A future test rejects missing/unknown targets instead of silently using the
   server default branch.

---

## REQ-DOLT-0002: native branch and conflict-free merge

**Status:** Proposed  

### Requirement Statement

**Initial delivery** MUST support native branch **create**, **list**, and
session **select** (for example `USE db/branch` or branch-qualified identifiers),
plus **merge** into an explicit destination branch using Dolt's native
implementation. Fast-forward and clean three-way merges MUST succeed when Dolt
reports no conflicts.

Branch **delete**, **rename**, and procedure **`DOLT_CHECKOUT`** are
**deferred**; they remain catalogued in
[version-api evidence](../research/dolt-v2.6.2/version-api.md) only.

Conflicts MUST surface native row/schema/constraint diagnostics and source,
target, and base revision information when Dolt provides them. The library MUST
NOT choose a side, force a merge, or implement a second diff/merge engine. A
clean published merge MUST NOT be blindly rolled back.

Adapter behavior for dirty working sets and `DOLT_CHECKOUT` MUST follow the
executed v2.6.2 probe ([probes/probe.md](../research/dolt-v2.6.2/probes/probe.md))
and reconciled [branches evidence](../research/dolt-v2.6.2/branches.md), preferring
explicit qualification over assumed checkout refusal.

### Rationale

Branch lifecycle and merge semantics are native Dolt responsibilities. A thin
adapter preserves one engine and one conflict vocabulary.

### Success Criteria

1. Native probe results for fast-forward merge, clean three-way merge, conflict
   rollback/abort, and dirty-working-set behavior are cited in architecture.
2. A future pinned SQLx compatibility test proves the selected conflict-cleanup
   flow and dirty-target policy.
3. Plan documents align initial operation list with version-api disposition
   (create/list/select + merge required; delete/rename/checkout deferred).

---

## REQ-DOLT-0003: credentials and attribution

**Status:** Proposed  

### Requirement Statement

SQL credentials MUST be resolved through an application-supplied
`SqlCredentialResolver` (object-safe trait; see architecture). The resolver MAY
back local per-connection storage and environment overrides. Connection identity
MUST include server, database, and SQL principal; pool reuse keys MUST include
credential version and branch.

Secrets MUST NOT appear in `Debug`, error strings, operation receipts, or
injected observability events. SQL authentication, human commit author, and
optional agent/team/repository attribution remain distinct; the application
approves attribution claims.

The crate MUST NOT read Raptor-specific configuration files.

### Rationale

Concurrent consumers on one Dolt server need isolated credentials without
mutating shared server environment.

### Success Criteria

1. Architecture sketches `SqlCredentialResolver` and states object-safety.
2. Future unit tests redact secrets from `Debug` and error display.
3. Future tests prove two pool keys when credentials differ for the same
   database name on different servers.

---

## REQ-DOLT-0004: native remote operations

**Status:** Proposed  

### Requirement Statement

The library MUST expose native remote selection, fetch, push, and pull with
explicit local branch and remote/ref context. It MUST NOT force push/pull,
automatically resolve merge conflicts, or mutate a shared server process
environment for per-client remote credentials.

Native Dolt remote operations are **not** consumer store-to-store row
replication; they are protocol operations on a Dolt server
([ADR-RUN-0301](../architecture.md) and [ADR-RUN-0305](../architecture.md)
amendments in [architecture](architecture.md)).

### Rationale

Consumers need honest remote capability without smuggling replication into the
framework crate graph.

### Success Criteria

1. Architecture amends ADR-RUN-0301/0305 with a named exception for `sc-dolt`
   remote procedure calls.
2. Future tests require explicit remote context and surface privilege failures
   as typed errors.

---

## REQ-DOLT-0005: revision and outcome evidence

**Status:** Proposed  

### Requirement Statement

Operations MUST return actual native outcomes and resulting commit identities.
Typed errors MUST preserve SQL failures and procedure status/conflict counts. An
uncertain network outcome MUST NOT be treated as proof of failure; mutating
commands MUST NOT replay without a consumer-owned idempotency marker.

The library MUST expose native diff/history/status reads needed to investigate
changes and merge conflicts. Domain audit records remain consumer-owned.

### Rationale

Operators need native evidence, not collapsed integer statuses.

### Success Criteria

1. Architecture names `DoltError` with stable codes ([NFR-RUN-0009](../requirements.md)).
2. Future tests assert conflict payloads include native diagnostics fields.

---

## REQ-DOLT-0006: thin multi-consumer integration

**Status:** Proposed  

### Requirement Statement

The crate MUST use the existing SQLx MySQL transport and Dolt native tools. It
MUST NOT own domain schema, requirement IDs, application configuration, a
command registry, or a plugin engine. The `sc-runtime` daemon assembly MUST
remain store-agnostic and MUST NOT gain a mandatory dependency on `sc-dolt`.

Fork, clone, backup administration, automatic conflict resolution, and a second
diff engine are outside initial delivery. **SQLite snapshot export to refresh a
local cache is consumer-owned** and is not part of `sc-dolt` acceptance
([identity evidence](../research/dolt-v2.6.2/identity.md)).

### Rationale

One reusable library reduces duplication without expanding the framework
surface or owning application storage policies.

### Success Criteria

1. `sc-runtime` workspace dependency graph in architecture excludes
   `sc-runtime` → `sc-dolt`.
2. Identity acceptance probe 5 is marked consumer-owned in evidence.
3. Future integration proves two consumers with different table models on pinned
   Dolt via SQLx.

---

## Evidence and validation (future implementation gate)

[Source inventory and executed probe](../research/dolt-v2.6.2/README.md).

When hold lifts, acceptance requires Rust 1.94.1 build/lint/unit tests, a pinned
real Dolt SQLx compatibility test, and two consumers with different table
models. Native probe results are evidence only until the Rust adapter exists.
