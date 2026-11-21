# sc-dolt proposed architecture

**Status:** Proposed (on hold per [sc-dolt plan](../plans/maintenance/sc-dolt-plan.md))  
**Scope:** thin reusable branch-aware Dolt access in `crates/sc-dolt`  
**Source:** owner planning conversation 2026-11-02, QA-1 findings 2026-11-11,
[pinned Dolt evidence](../research/dolt-v2.6.2/README.md)

No product code ships under this proposal. **Root** documents (`docs/requirements.md`,
`docs/architecture.md`) change only when implementation is authorized and the
amendments below are applied verbatim.

---

## Public surface sketch (binding on implementation)

These names are planning contracts. They MUST NOT wrap or re-export `sqlx` pool,
connection, or transaction types ([ADR-RUN-0001](../architecture.md) point 4
amendment).

| Type | Role |
| --- | --- |
| `DatabaseName`, `BranchName`, `RevisionId` | Validated newtypes; parse/validate at construction |
| `BranchTarget` | Pair of validated database + branch |
| `BranchLease` | Owns one SQL connection and session state for one target; drop/rollback policy documented at implementation |
| `DoltError` | Crate error enum with stable `code()` strings ([NFR-RUN-0009](../requirements.md)) |
| `SqlCredentialResolver` | Object-safe trait: resolve SQL principal + secret handle for a `BranchTarget`; no secret in `Debug` |

Consumers execute SQL through methods on `BranchLease` that accept only bound
parameters and qualified identifiers assembled by the consumer store layer.
`BranchLease` MAY hold a private `sqlx` connection internally; that type MUST
NOT appear in public signatures.

**Typestate (planned):** `Unbound` → `CheckedOut` → `InTransaction` → `Published`.
The lease typestate is also the capability token: only `InTransaction` may issue
CRUD; only `Published` may release the connection to a pool.

---

## ADR-DOLT-0001: reuse SQLx MySQL and native Dolt operations

**Status:** Proposed  
**Decision Date:** 2026-11-02  
**Source:** [sc-dolt requirements](requirements.md) REQ-DOLT-0001, REQ-DOLT-0006  
**Amends:** [ADR-RUN-0001](../architecture.md) point 4 (public session types)  

### Context

Applications need Dolt branch semantics without a second driver. SQLx MySQL
handles the wire protocol; Dolt owns branches, merges, and conflict math.

### Decision

1. Implementation lives at `crates/sc-dolt`, a workspace sibling of the runtime
   SDKs. It does not become a dependency of the daemon bootstrap.
2. SQLx MySQL owns pooling primitives; Dolt owns native branch/merge/remote
   procedures. Consumer stores own schema and domain CRUD SQL.
3. Public API is the validated newtypes and `BranchLease` above—not a SQL parser
   or command registry.
4. Signatures compile against the selected SQLx version before the ADR moves to
   Active.

### Consequences

Consumers depend on `sc-dolt` from generated store crates, not from `sc-runtime`.

---

## ADR-DOLT-0002: selection is explicit and request-local

**Status:** Proposed  
**Decision Date:** 2026-11-02  
**Source:** REQ-DOLT-0001, [branch evidence](../research/dolt-v2.6.2/branches.md)  

### Decision

1. Every operation carries an explicit validated `BranchTarget`.
2. Pool registry keys include server, database, principal, credential version,
   and branch.
3. A lease owns one connection end-to-end for an operation; no checkout on one
   connection and CRUD on another.
4. Prefer `USE db/branch` or qualified identifiers over `DOLT_CHECKOUT` in the
   initial delivery (checkout deferred per REQ-DOLT-0002).

### Consequences

Branch working sets may be shared across sessions; transactions retain isolated
views.

---

## ADR-DOLT-0003: merge only through native success semantics

**Status:** Proposed  
**Decision Date:** 2026-11-02  
**Source:** REQ-DOLT-0002, [probes](../research/dolt-v2.6.2/probes/probe.md)  

### Decision

Support create/list/select, fast-forward merge, and clean three-way merge.
Return conflicts with native diagnostics; never publish conflict-bearing commits.
The pinned SQLx test must lock one native conflict-cleanup flow before release.

---

## ADR-DOLT-0004: authentication is not attribution

**Status:** Proposed  
**Decision Date:** 2026-11-02  
**Source:** REQ-DOLT-0003  

### Decision

1. Applications implement `SqlCredentialResolver`.
2. Attribution metadata is optional and application-owned.
3. Observability uses **`sc-observability-types` only** at the injection seam
   (see ADR-RUN-0004 amendment). `sc-dolt` MUST NOT depend on `sc-observability`
   or `sc-observability-otlp`.

---

## Binding root amendments (apply when hold lifts)

### Amendment to REQ-RUN-0001 / workspace members

Add one workspace member row (same pattern as `sc-cron-commands`):

| Member path | Crate name | Purpose |
| --- | --- | --- |
| `crates/sc-dolt` | `sc-dolt` | Dolt branch/merge/remote access (planned; not yet a member) |

`sc-runtime` MUST NOT depend on `sc-dolt`.

### Amendment to ADR-RUN-0003 (edge row)

| Crate | Workspace crates it depends on | Forbidden edges |
| --- | --- | --- |
| `sc-dolt` | none at workspace level; **may** depend on crates.io `sqlx` (MySQL) and Tokio; **may** depend on `sc-observability-types` only among workspace observability crates | `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-runtime-db-sqlite`, `sc-cron`, `sc-observability`, `sc-observability-otlp` |

`sc-runtime` and existing store crates remain **`sqlx`-free** (cargo tree unchanged).
`allowed_dependents` for `sc-dolt`: empty at introduction; consumer store crates
add edges through their manifests when integrated.

### Amendment to ADR-RUN-0004

`sc-observability-types` is allowed on `sc-command`, `sc-runtime`, `sc-cron`, and
**`sc-dolt`**. Injection contract is unchanged: projects construct canonical
events; `sc-dolt` emits through injected types only.

### Amendment to ADR-RUN-0001 point 4

`sc-dolt` MAY expose `BranchLease` and validated string newtypes. Those types
MUST NOT be wrappers that re-export sqlx, Axum, or rmcp types. Internal use of
sqlx is private to the crate.

### Amendment to ADR-RUN-0301 point 6 / ADR-RUN-0305 point 9

Framework crates still do **not** move rows between consumer stores. **Exception:**
`sc-dolt` issues native Dolt `fetch` / `push` / `pull` against a Dolt server
using Dolt's protocol. That is not store-to-store replication inside the
generated adaptor; row copy between SQLite and Dolt remains consumer-owned.

---

## Proposed boundary map

| Boundary | Owns | Does not own |
| --- | --- | --- |
| sc-dolt | branch leases, native operations, credentials seam, `DoltError` | domain models, migrations, HTTP commands |
| sc-runtime | daemon lock/router/shutdown and stores closure | mandatory database engine |
| generated consumer store | table SQL, canonical schema | second MySQL driver |
| consumer cache adaptor | immutable revision reads, **SQLite snapshot export** | Dolt merge or bidirectional reconciliation |

---

## Delivery cut

Execution stays in Beads epic `run-sc-dolt` (`.1` branch sessions, `.2` native ops,
`.3` two-consumer proof). See [sc-dolt plan](../plans/maintenance/sc-dolt-plan.md).

Publication of any crate version follows the maintainer release gate; this plan
does not authorize tags or registry upload.
