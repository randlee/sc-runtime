# Boundary manifests (`sc-lint lint sc-boundary`)

Each library crate owns a TOML manifest under `boundaries/<crate>/`, enforced
by `sc-lint lint sc-boundary` ([REQ-RUN-0005](../docs/requirements.md)).
`[dependencies].allowed_dependents` is empty for `sc-config` and
`sc-runtime`, and `sc-runtime` only for `sc-transport` and `sc-command`.

| Crate | Sprint that creates manifest |
| --- | --- |
| `sc-config` | a-2 |
| `sc-transport` | a-3 |
| `sc-command` | a-4 |
| `sc-runtime` | a-5 |

Manifest format and forbidden edges are normative in `docs/requirements.md`
(REQ-RUN-0005) and `docs/architecture.md`.

a-1 records that `sc-lint lint sc-boundary` exits 0 on a disposable
single-crate tree that is its own `[workspace]` root (`members = ["."]`,
`[workspace.package].version`, inherited `[workspace.package]` fields on the
member; fixture `exit-ok`). A tree with no root `[workspace]` exits 3
(fixture `no-workspace`). Each tree needs `boundaries/planning.toml` with
`current_sprint = "<phase>.<step>"` and each boundary manifest needs `name`.
Negative fixture runs (prefix match, transitive versus direct) are in
`docs/architecture.md`. a-2 created the single-member root lint tree
(`Cargo.toml` with `members = ["crates/sc-config"]`). a-3 and a-4 append a
workspace member; they do not create that file. a-5 extends the same
workspace to four members and is the first sprint whose `just lint gates`
covers all four manifests. a-2, a-3, and a-4 run the recorded command on
their own crate `Cargo.toml` and manifest. They do not keep analyzer output
in a pull request. Leaf QA for a-2, a-3, a-4, and a-5 re-runs that command
at the reviewed commit and records only the exit code in the QA report.
That re-run is the binding evidence. Workspace-wide `just lint gates` is
not the leaf gate. The a-2 manifest waits on gate bead
`triage:a-2-boundary-evidence`.

`[public] facade` is a comma-separated list of exported paths. sc-lint 0.8.0
parses the list and does not compare it to the crate's `pub use` items.
Fixture `examples/spike/boundary-proof/facade-mismatch` exports `present`
and `extra` while the manifest names only `facade_mismatch::present`.
`sc-lint lint sc-boundary` on a copy of that tree exits 0. The sc-config
test `facade_lists_every_export` is the check that the manifest lists every
export.
