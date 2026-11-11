# Intent — Phase D: easy adoption and compatible upgrades

Updated 2026-11-11 following the user's planning handover. This is the working
intent for revising [the phase plan](phase-d-plan.md); sprint and anchor
reconciliation follows this revision.

## Purpose

Phase D makes sc-runtime easy for consumers to adopt, build, and maintain.
Published libraries and generated applications should fit into a consumer's
dependency graph and let that consumer take non-breaking fixes and upgrades on
their own schedule. The framework should remove avoidable integration work and
dependency restrictions throughout that lifecycle.

The immediate failure is concrete: **beads-cache is blocked by the rusqlite /
SQLite security advisory** because published `sc-cron` 0.2.0 prevents resolution
to a fixed SQLite dependency. [Issue #132](https://github.com/randlee/sc-runtime/issues/132)
records RUSTSEC-2026-0459 and the native-library conflict. Phase D must deliver a
published **0.2.1** that unblocks that consumer, together with a dependency policy
that prevents the same avoidable restriction from recurring.

## Consumer outcomes

- A consumer can use the released crates and build a generated application
  using the documented features and supported Rust toolchain.
- A consumer can use `cargo update` to take compatible dependency fixes without
  waiting for another sc-runtime release or adding local patches to its crates.
- The released dependency graph clears the rusqlite advisory blocking
  beads-cache and the rmcp advisory identified in
  [#133](https://github.com/randlee/sc-runtime/issues/133).
- Manifest floors describe versions that are actually tested. CI exercises the
  supported minimum and latest allowed dependency sets, including the relevant
  template features and separate consumer workspaces.
- README examples, feature instructions, and package metadata describe the
  published product. This includes the `test-util` / `instrument_router` helper
  restored in [PR #131](https://github.com/randlee/sc-runtime/pull/131).
- Maintainers and consumers can choose generation-tool versions independently.
  CI can reproduce its chosen sc-compose release from immutable binaries;
  compatible consumer overrides through `SC_COMPOSE` or PATH remain supported.
  Distribution improvements are tracked in
  [sc-compose#1041](https://github.com/randlee/sc-compose/issues/1041).

## Dependency policy direction

The proposed default is a tested minimum with Cargo's normal SemVer-compatible
upgrade range. Exact pins, narrower ranges, and toolchain restrictions need a
specific compatibility reason. Reproducibility belongs in application/CI
lockfiles and immutable tool installs while published library requirements leave
room for compatible consumer upgrades.

Cargo's [dependency guidance](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
explains that a requirement such as `1.2.3` already permits compatible upgrades;
an unbounded `>=1.2.3` also admits future breaking versions. Pre-1.0 minor
versions cross Cargo compatibility boundaries. Public dependency types and
native-library `links` constraints also require attention when widening a range;
see [dependency resolution](https://doc.rust-lang.org/cargo/reference/resolver.html).

The existing #133 wording and [sweep inventory](issue-133-dependency-sweep.md)
request blanket `>=` ranges. Reconcile that prescription with this consumer
compatibility goal before finalizing manifest changes and acceptance criteria.
Support across a Cargo compatibility boundary, including rusqlite 0.41 / 0.42,
requires explicit validation of the affected APIs and dependency graph.

The rusqlite/sqlx native-library conflict needs an explicit supported
configuration and consumer migration guidance. The current d-1 proposal drops
same-process coexistence with sqlx 0.9. Its adequacy must be checked against the
actual beads-cache unblock; deleting the coexistence fixture alone cannot prove
that outcome.

## Proof and scope

Success includes a registry consumer using released 0.2.1 crates, resolving the
fixed dependencies, passing the relevant advisory checks, and building/running
the affected integration. Consumer proof must exclude local path/patch
substitutes. Repository tests accompany this proof. Regression tests use
controlled inputs and deterministic synchronization.

This maintenance phase covers dependency constraints, generated defaults,
consumer documentation, reproducible generation tooling, and the release needed
to deliver those improvements. Breaking dependency migrations and new product
features require separate justification. A reported upstream incompatibility
stays explicit until a supported consumer path is demonstrated.

## User provenance and planning status

The user stated during the 2026-11-11 planning handover:

> The intention of phase-d is to make sc-runtime easy for consumers to use.
> today beads-cache is blocked because of the security warning on rusqlite.
> we need to be following 'best practices' for a scaffolding framework which does not block consumers from building or upgrading dependencies w/ non-breaking changes/upgrades.

Existing [anchors](anchors.json) and [their rendered register](anchors.md) retain
their prior records, including retired ANC-008. This intent revision does not
re-lock anchors or certify existing sprint plans. Reconcile their wording and
acceptance criteria with this intent before implementation begins.
