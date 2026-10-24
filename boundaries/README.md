# Boundary manifests (`sc-lint-boundary`)

Each library crate owns a TOML manifest under `boundaries/<crate>/`, enforced
by `sc-lint lint sc-boundary` ([REQ-RUN-0005](../docs/requirements.md)).

| Crate | Sprint that creates manifest |
| --- | --- |
| `sc-config` | a-2 |
| `sc-transport` | a-3 |
| `sc-command` | a-4 |
| `sc-runtime` | a-5 |

Manifest format and forbidden edges are normative in `docs/requirements.md`
(REQ-RUN-0005) and `docs/architecture.md`.

a-1 records that `sc-lint lint sc-boundary` exits 0 on a disposable
single-crate tree with no root `[workspace]`. a-2, a-3, and a-4 run that
recorded command on their own crate `Cargo.toml` and manifest. That leaf
gate exits 0. It does not wait for the root workspace, and it does not keep
analyzer output in a pull request. a-5 creates the root `Cargo.toml` and is
the first sprint whose `just lint gates` covers all four manifests.
Workspace-wide `just lint gates` is not the leaf gate.
