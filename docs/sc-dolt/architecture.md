# sc-dolt proposed architecture

Status: proposed. Scope: a thin reusable branch-aware Dolt access crate maintained
in sc-runtime, consumed by Raptor and other applications. No product code is
changed by this proposal. Source: owner planning conversation of 2026-11-02 and
[pinned Dolt evidence](../research/dolt-v2.6.2/README.md).

## ADR-DOLT-0001: reuse SQLx MySQL and native Dolt operations

The implementation lives at `crates/sc-dolt`, a sibling of the runtime SDKs.
It does not become a dependency of the daemon bootstrap merely because it lives
in this workspace. Applications supply stores through the existing stores seam.
SQLx MySQL owns the wire protocol, pooling primitives and transactions. Dolt
owns branches, commits, merges and conflict/diff computation. Consumer stores own
schema, migrations and domain CRUD. This preserves the framework charter and
avoids a second database driver or portable query framework.

The first contract names validated database, branch and revision types; a
branch-bound session/transaction; a connection credential resolver; native
operation results and stable actionable errors. This is a focused access API,
not a SQL parser or a second command framework. Compile the contract against the
selected SQLx version before freezing signatures; method names in this document
are conceptual until that contract is reviewed.

## ADR-DOLT-0002: selection is explicit and request-local

Clients resolve configuration and attach database/branch to every request.
The library receives the resolved target. It never obtains a remote client's
branch by reading the server environment. A bounded pool registry may reuse
connections only within the same server/database/principal/credential-version/
branch scope; changing a scope requires reinitialization proved by a real test.
A lease owns one SQL connection for its complete transaction and commit outcome.
Do not run checkout on one connection and CRUD on another. Validate/quote
identifiers through the driver; bind ordinary procedure/data values.

Bound resource use and pool retirement are configuration, not performance claims.
Do not promise independent dirty working sets to clients on the same branch:
Dolt branches have shared working sets, while transactions have their own views.
Raptor normally publishes a versioned transaction per mutation; the generic
crate exposes the distinction rather than secretly enabling per-statement commits.

## ADR-DOLT-0003: merge only through native success semantics

Support branch creation, fast-forward merge and clean three-way merge. Native
preview/diff can inform a request but does not authorize a later merge: source
and target may move, and preview does not enumerate every constraint failure.
The actual merge outcome is authoritative. Preserve the observed before/after
revision IDs and native conflict diagnostics. Return a conflict rather than
publishing a conflict-bearing commit or choosing ours/theirs.

The native probe shows that autocommit conflicts roll back, manual transaction
conflicts can be inspected then rolled back, and abort restores the merge state.
It also shows that rollback does not undo a clean merge already published.
The adapter's first real SQLx test must select and lock down one native flow,
including dirty-target protection, constraint failures, concurrent head changes,
cancellation and lost-response behavior. If a requested guarantee cannot be
proved, fail the compatibility gate; do not implement a guessed compensating reset.

## ADR-DOLT-0004: authentication is not attribution

Applications provide SQL credentials by named connection, with pluggable local
storage/environment support. Keep secret values out of context/trace objects.
Human, agent/session, team and repository may be attribution attached to an
operation; none substitutes for the authenticated SQL principal. The application
owns approval of author claims. Reuse the existing sc-observability injection
contract at the selected SDK baseline; do not create a competing logging API.

Remote SQL procedures can use server-managed credentials that are not isolated
per caller. Expose that capability/limitation. Never write environment variables
while serving concurrent clients. Additional remote credential backends require
their own proved binding and can be added without changing record CRUD.

## Proposed boundary map

| Boundary | Owns | Does not own |
| --- | --- | --- |
| sc-dolt | branch leases, native operations, credentials seam, outcomes | domain models, migrations, HTTP commands |
| existing sc-runtime | daemon lock/router/shutdown and stores closure | a mandatory database engine |
| generated/consumer store | table SQL, canonical schema, transactions policy | another MySQL wire driver |
| consumer cache adaptor | immutable revision reads and local publication | Dolt merge or bidirectional reconciliation |

Proposed normal dependencies are SQLx MySQL/Tokio and the existing SDK's
applicable shared error/observability types. Existing boundary manifests and the
chosen SDK baseline decide the exact allowed edges; add the new manifest in the
implementation, not fictitious current edges. Keep the schema crate and CLI free
of server/database runtime dependencies. SQLx/Dolt source/API pins are recorded
in the implementation receipt and real integration job.

## Delivery cut

Execution is tracked in Beads under `run-sc-dolt`, not duplicated in this file.
A branch/session implementation establishes the usable crate and branch-create
path. A native-operations implementation adds clean merge, diagnostics and remote
operations using that capability. A consumer integration proves the complete
API against pinned Dolt and two differently shaped consumers. These form a
single three-step track because the later operations consume the actual branch
lease implementation and the final proof consumes the complete native facade.
No unrelated SDK/template upgrade is pulled into this scope.

The initial source/checkout integration is independent of publication. Publishing
or changing SDK release versions follows the existing maintainer release gate;
this plan does not authorize publishing credentials, tags or crate releases.
