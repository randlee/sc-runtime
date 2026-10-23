# Boundary manifests (`sc-lint-boundary`)

Each library crate owns a TOML manifest under `boundaries/<crate>/`, enforced
by `sc-lint lint sc-boundary` through `just lint` ([REQ-RUN-0005](../docs/requirements.md)).

| Crate | Sprint that creates manifest |
| --- | --- |
| `sc-config` | a-2 |
| `sc-transport` | a-3 |
| `sc-command` | a-4 |
| `sc-runtime` | a-5 |

Manifest format and forbidden edges are normative in `docs/requirements.md`
(REQ-RUN-0005) and `docs/architecture.md`.

Until the workspace and first manifest exist, `just lint gates` may fail
(`Cargo.toml` arrives in a-5; manifests from a-2 onward). That is expected
before those sprints land.
