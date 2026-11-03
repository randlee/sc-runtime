# sc-dolt requirements

Status: proposed; planning only. The owner requests reusable multi-branch Dolt
access within the sc-runtime package. These requirements do not authorize an
implementation or a release. Raptor is the first consumer, not the API namespace.

## REQ-DOLT-0001: explicit branch-scoped execution

Every consumer CRUD execution must carry an explicit validated database and
branch. The library must not silently select main, share a global current branch,
or hand consumers an unscoped application-wide pool. Concurrent clients targeting
different branches must remain isolated, including when connection capacity is
reused. A branch-bound execution capability is the consumer's SQL seam; the
consumer owns record/table SQL and cannot submit arbitrary client SQL through
Raptor's HTTP API. See branch-session and branch-qualified claims in the evidence.

## REQ-DOLT-0002: native branch and conflict-free merge

Create/list/select branches and merge into an explicit destination branch using
Dolt's native implementation. Both fast-forward and clean three-way merges must
work. Conflicts produce native row/schema/constraint diagnostics and source,
target and base revision information when available. Do not choose a side,
force a merge, or implement a second diff/merge engine. Do not confuse unchanged
HEAD with an unchanged working set. The real SQLx test must prove the conflict
cleanup policy and that a clean published merge is not blindly retried/rolled back.

## REQ-DOLT-0003: credentials and attribution

Accept SQL credentials through an application-supplied resolver or resolved secret
handle, supporting locally stored per-connection/per-database credentials and
environment overrides. A connection identity includes server, database and SQL
principal; the same database name on two servers is not the same credential key.
Pool reuse must include credential identity/version and branch. Do not read
Raptor-specific configuration in this library. SQL authentication, human commit
author and optional agent/team/repository attribution remain distinct. Secrets
must not enter Debug, errors, operation receipts or observability events.

## REQ-DOLT-0004: native remote operations

Expose the native remote selection, fetch, push and pull operations needed by
consumers. Require explicit local branch and remote/ref context. Use native
privilege checks and report unsupported authentication arrangements honestly.
Do not mutate a shared server process environment to emulate per-client remote
credentials. SQL login credentials and Dolt remote credentials are separate.
Do not force push/pull or automatically resolve conflicts. A native ff-only policy
is available where the consumer selects it; it is not a substitute for clean
three-way branch merge support.

## REQ-DOLT-0005: revision and outcome evidence

Return actual native outcomes and resulting commit identities. Preserve typed
SQL failures and procedure status/conflict results. An uncertain network outcome
is not proof that an operation failed; do not blindly replay a mutating command.
Provide native diff/history/status reads needed to investigate changes and merge
conflicts. Consumers own domain audit records and stakeholder relationships.

## REQ-DOLT-0006: thin multi-consumer integration

Use the existing SQLx MySQL transport and Dolt tools. The crate owns no domain
schema, requirement IDs, Raptor configuration, command registry or plugin engine.
The existing daemon assembly remains store-agnostic. Fork, clone, backup/admin
maintenance, automatic conflict resolution and a new diff implementation are
outside the initial delivery. Catalogue them as evidence, not hidden promises.

## Evidence and validation

[Source inventory and executed probe](../research/dolt-v2.6.2/README.md).
Acceptance requires Rust 1.94.1 build/lint/unit tests, a pinned real Dolt SQLx
compatibility test and two consumers with different table models. Native probe
results are not a claim that the Rust adapter already exists or passes tests.
