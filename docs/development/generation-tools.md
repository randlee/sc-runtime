# Generation and plan-anchoring CLIs

External executables used by this repository for **template generation** and
**plan anchoring**. They are not workspace crate dependencies.

## Canonical pin

[`.sc/generation-tools.json`](../../.sc/generation-tools.json) is the single pin
for the **`sc-compose` CLI** (currently **1.8.0**, matching plan-anchoring 0.11.0’s
minimum). CI reads it in
[`.github/actions/rust-build-cache`](../../.github/actions/rust-build-cache/action.yml).
Bump the JSON when maintainers adopt a newer release binary.

## Tool vs crate

| Concern | What it is | Where it lives |
| --- | --- | --- |
| **sc-compose tool** | `sc-compose` on `PATH` or `SC_COMPOSE` for `scripts/new_project.py`, Template CI, and `plan-anchoring`’s `change_report.py` | `.sc/generation-tools.json`, release install in CI |
| **sc-compose crates** | `sc-compose` / `sc-composer` as **Cargo dependencies** | Forbidden on production release crates; boundary lint and ADR-RUN-0017. Dev/test/benchmark references only. |

Do not conflate the CLI pin with crates.io crate versions inside consumer
`Cargo.toml` files. Generated applications choose their own tool via `SC_COMPOSE`.

## Local setup

Install **sc-compose ≥ 1.8.0** from [randlee/sc-compose releases](https://github.com/randlee/sc-compose/releases)
(same tarball + `checksums.txt` pattern as CI). Confirm:

```bash
sc-compose --version   # sc-compose 1.8.0 or newer
```

CI details: [ci.md](ci.md).
