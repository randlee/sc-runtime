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
single-crate tree with no root `[workspace]`, and records the negative
fixture runs (prefix match, transitive versus direct) in
`docs/architecture.md`. a-2, a-3, and a-4 run that recorded command on their
own crate `Cargo.toml` and manifest. They do not wait for the root
workspace, and they do not keep analyzer output in a pull request. Leaf QA
for a-2, a-3, a-4, and a-5 re-runs that command at the reviewed commit and
records only the exit code in the QA report. That re-run is the binding
evidence. a-5 creates the root `Cargo.toml` and is the first sprint whose
`just lint gates` covers all four manifests. Workspace-wide `just lint gates`
is not the leaf gate. The a-2 manifest waits on gate bead
`triage:a-2-boundary-evidence`.
