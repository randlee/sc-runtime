# Dolt evidence baseline

Dolt v2.6.2, source `46d8805d8556599eb0a6472892e67e3bb1e0de03`.
This is implementation evidence, not an assertion that every upstream API belongs
in sc-dolt. Three Luna investigations covered branch/session behavior, the full
registered version-control surface, and credentials/identity. Source links pin
this exact commit; cited upstream tests were inspected, not executed here.

- [Branch/session claims](branches.md), with [structured claims](branches.json).
- [API inventory and selected operation contracts](version-api.md),
  [structured inventory](version-api.json), and [compact inventory](version-api.csv).
- [Credentials and identity](identity.md), with [structured claims](identity.json).
- [Executed native compatibility probe](probes/probe.md) and
  [machine-readable results](probes/probe-results.json).
- [Evidence checksums and baselines](manifest.json).

The native probe used an official checksum-verified v2.6.2 binary, isolated
configuration/data, and owned loopback servers. It did not use the installed
2.3.5 binary, the Beads database, port 3306, or global Dolt configuration.
No Rust SQLx wire compatibility proof has been executed; that is an explicit
first implementation acceptance gate, not a presumed result.

## Governing scope

The latest owner instructions supersede earlier exploration recommendations:
create branches and merge when there are no conflicts; use native Dolt diff and
conflict information when there are conflicts. Support concurrent clients with
explicit database/branch context. Retain the earlier push/pull requirement.
Fork and clone are outside the initial implementation. No custom diff engine,
automatic conflict resolver, generic export format, or destination manager.
The full inventory classifies excluded APIs so investigation is not mistaken
for an implementation commitment.

## Findings that change the design

A connection has session-local branch state. An unqualified database can resolve
to a prior checkout, so a reusable pool must not leak one caller's selection.
SQL transactions and Dolt history commits are distinct unless configured.
Native clean merge can publish immediately; later SQL rollback cannot undo it.
A conflict can leave a changed working set without a new HEAD. The executed
probe verifies the chosen rollback/abort cases; concurrency, cancellation and
constraint-conflict variants still require the adapter's real SQLx tests.
Preview is advisory: it can become stale and its summary is not a complete
constraint-violation detector. The native merge result remains authoritative.
Commit author fields are attribution, not proof of authenticated identity.
Remote push/pull credential scope is separate from SQL login credentials.
