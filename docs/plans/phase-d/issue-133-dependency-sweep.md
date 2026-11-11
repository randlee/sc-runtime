# Issue #133 — dependency range sweep (planning)

**GitHub:** [#133](https://github.com/randlee/sc-runtime/issues/133) (label `0.2.1`)  
**Companion:** [#132](https://github.com/randlee/sc-runtime/issues/132) rusqlite / RUSTSEC-2026-0459 (owned by [sprint d-1](sprint-d-1-rusqlite-0-41.md))  
**Authority for implementation:** sprint docs below; this file is inventory + blocker analysis.

## Executive summary

#133 asks for a **repo-wide** move from tight pins (`=`, `~`, bare `0.39`-style minors) to **`>=` tested floors** with **latest-compatible** resolution, proved by **lowest + highest** lockfile CI. #132 is one high-impact instance (rusqlite / `libsqlite3-sys`). #133 adds at least **rmcp ≥ 3.9** (RUSTSEC-2026-0462), **utoipa/schemars** range hygiene on published surfaces, template **workspace.dependencies** ranges, and contract/integration test manifests that still use `=3.8.0` / `=0.2.0`.

**Not everything can move to “latest on crates.io” in 0.2.1.** The phase d-1 **frozen sqlx ruling** caps how far **rusqlite / libsqlite3-sys** can move while template **sqlx 0.9** remains. A **sqlx major** or **in-process sqlx + writer** alignment is **out of Phase D** (see [phase-d-plan.md](phase-d-plan.md)).

## Sprint placement (Phase D)

| Sprint | Issues | Scope |
| --- | --- | --- |
| **d-1** | #132 | rusqlite `0.41`, ADR/sqlx ruling, cron-consumer-proof |
| **d-1b** | #133 (supply-chain) | `rmcp` floor ≥ 3.9, MCP stack, `>=` ranges on published crates + template where not blocked |
| **d-2** | #131, #132 publish, #133 done-when release | READMEs, **0.2.1**, publish proves swept ranges on registry |

**Lockfile:** d-1 and d-1b are `parallel_safe` on **paths** but both touch `Cargo.lock`; merge-forward whichever lands second before `integrate/phase-d` publish (d-2 publish tail).

## Pin inventory (high signal)

| Location | Dependency | Current | Proposed (0.2.1) | Blocked to “latest”? | Reason |
| --- | --- | --- | --- | --- | --- |
| `crates/sc-cron`, `sc-runtime-db-sqlite` | `rusqlite` | `0.39` | `0.41` exact + documented floor (d-1) | **Yes** vs sqlx 0.9 same-process | d-1 ruling: no sqlx 0.9 + writer 0.41 in one process; sqlx bump deferred |
| `template/crates/store-*/` | `sqlx` | `0.9.0` | Keep `0.9.x` with `>=` floor after audit | **Yes** for 0.10+ | Phase D out of scope: sqlx major |
| `sc-runtime`, `sc-command`, `template/.../daemon` | `rmcp` | `~3.8` | `>=3.9, <4` (test floor 3.9.0) | No for 3.9 advisory | RUSTSEC-2026-0462; API check in d-1b |
| `tests/contracts/...`, `cron-consumer/schema` | `rmcp` | `=3.8.0` | `>=3.9.0` aligned with workspace | No | Must match published range story |
| `sc-command`, `sc-cron-commands` | `utoipa` | `=6.0.0` | `>=6.0.0, <7` + comment if 7 blocked | Maybe | Major 7 needs compile proof |
| same | `schemars` | `=1.2.3` | `>=1.2.3, <2` | Maybe | Schema stability |
| `sc-runtime`, `sc-transport`, `sc-command` | `axum` | `0.8.9` | `>=0.8.9, <0.9` | TBD | Verify 0.8.latest in d-1b |
| `template/Cargo.toml` | `sc-observability*` | `=1.5.0` | `>=1.5.0` or kit-pinned comment | Partial | Template kit may require exact until release train says otherwise |
| Workspace path deps | `sc-*` `0.2.0` | path + version | `0.2.0` until d-2 bumps **0.2.1** | N/A | Intentional workspace coupling |

Full path-by-path sweep: background inventory [Issue #133 dependency sweep](502c3615-37ca-4de3-a7aa-8a616c45d1dc) (may extend this table).

## Cannot upgrade to latest (0.2.1 line)

1. **rusqlite / libsqlite3-sys beyond sqlx 0.9 window** — architecture + d-1 ruling; not fixed by `#133` alone.
2. **sqlx 0.10+** on template stores — requires phase beyond D (or explicit sqlx sprint).
3. **Same-process sqlx store + sc-runtime-db-sqlite writer** — explicitly unsupported on 0.2.1 until sqlx/rusqlite alignment sprint.
4. **Dependencies whose latest major breaks published API** — d-1b must record `>= floor, <next-major` with a **comment citing failing test or ADR** (utoipa 7, etc.) after compile matrix.

## CI / validation (#133 done-when)

Extend Phase D validation (d-1b + d-2 integration):

1. **Lowest allowed:** `cargo +nightly update -Z minimal-versions` (or repo-standard recipe if added to `Justfile`) on `integrate/phase-d` after d-1 + d-1b merge.
2. **Latest compatible:** `cargo update` then `just lint && just test` (already d-2 AC).
3. Document commands in sprint PR bodies; avoid new workflow unless sprint names consumer, defect, and retirement per guidelines.

Repo today has **no** checked-in `minimal-versions` recipe (grep clean); d-1b deliverable includes **one named Justfile recipe or workflow step** referenced from sprint doc.

## #133 done-when mapping

| Criterion | Owner |
| --- | --- |
| No unjustified `= / ~ /` pins on published + template deps | d-1b |
| CI tests lowest + highest dependency sets | d-1b (recipe) + d-2 (integrate proof) |
| Consumer can `cargo update` advisory fixes without waiting | **0.2.1 publish** (d-2) after ranges land |
