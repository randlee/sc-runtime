# Beads Issue Tracking

This repository uses Beads for durable task, dependency, blocker, and handoff
state. Run `bd prime` for the current workflow and `bd ready` for unblocked
work. Local execution checklists are not a substitute for the shared ledger.

Issue records live in the configured **shared Dolt server**. The versioned
`metadata.json` selects the server mode and database; `config.yaml` enables
`dolt.shared-server` and declares the sync remote:

```text
https://doltremoteapi.dolthub.com/synaptic-canvas/sc-runtime
```

Use `bd dolt remote list` to inspect the active Dolt replication configuration.
Use `bd dolt pull` and `bd dolt push` to synchronize issue data with DoltHub.
The code Git remote and its branches are separate from this Dolt remote.
A code push does not carry the issue database; any hook-driven Beads
replication still uses the configured Dolt remote. This repository uses the
DoltHub target above rather than Git `refs/dolt/data` replication.
Do not use `bd import` for ordinary synchronization.

Version `config.yaml`, `metadata.json`, the `.gitignore` rules, this README,
and the hook wrappers in `hooks/`. They describe the chosen setup without
committing database contents or credentials. Inspect these files and the active
Dolt remote instead of inferring issue storage from the code Git remote.

Dolt database files, per-machine runtime state, credentials, and passive
`issues.jsonl` exports are ignored. An export may be stale or incomplete; it
is neither the issue source of truth nor the wire protocol and should not be
staged in a code PR. Query current issues through `bd` against the configured
server. Local files are preserved when ignored; ignoring an export does not
import, migrate, delete, or synchronize its contents.
