# Issue #133 — dependency range sweep

**Code truth:** `develop` @ `c3144cf` (Cargo.toml / Cargo.lock unchanged after that commit).  
**Crates.io snapshot:** 2026-11-11 (`max_stable` from the crates.io API).  
**GitHub:** [#133](https://github.com/randlee/sc-runtime/issues/133). Companion: [#132](https://github.com/randlee/sc-runtime/issues/132).  
**Sqlite ruling:** [sprint-d-1-rusqlite-0-41.md](sprint-d-1-rusqlite-0-41.md) (frozen). This file is the inventory. It does not retarget d-2.

Sweep covered all 28 `Cargo.toml` files under the repo root, `crates/`, `template/`, and `tests/`.

Range policy follows the approved consumer intent and ANC-016/017/019:

- Declare a tested minimum using Cargo's normal SemVer-compatible range.
  A requirement such as `"3.9.0"` means `>=3.9.0, <4`; it is not an exact pin.
- Test the minimum and latest allowed dependency sets in CI. A wider range
  across compatibility lines requires API, native-link, and toolchain proof.
- Exact pins, tilde restrictions, and extra upper bounds need a concrete
  compatibility reason in the owning manifest. Normal Cargo compatibility
  boundaries do not assert that an unpublished major has an observed defect.
- The original #133 wording requested blanket `>=`. The user's later intent
  explicitly concerns non-breaking upgrades. This plan implements compatible
  upgrades and qualified broader envelopes, rather than promising support for
  arbitrary future breaking versions. Preserve the original issue wording as
  provenance; the final report shows this policy clarification for approval.
- Raise broad caret floors to versions CI actually tests; neither a caret nor
  an explicit `>=` makes an untested floor truthful.

## 1. Executive summary

#132 moves **rusqlite** off the `0.39` / `libsqlite3-sys` 0.37 pin so a consumer can take the SQLite advisory (RUSTSEC-2026-0459, fixed in `libsqlite3-sys` ≥ 0.39, first shipped by **rusqlite 0.41**). #133 is the same rule for **every other dependency** in this repo, including the template and the test workspaces:

- Replace unjustified restrictive pins with a tested floor and a supported compatibility range.
- Explain extra restrictions in manifest comments; preserve normal Cargo compatibility boundaries.
- Run CI at both ends: `cargo +nightly update -Z minimal-versions` and `cargo update`.
- Do that in the generated template, then publish so `cargo update` in a downstream crate picks up a fixed dependency without waiting for another sc-runtime release.

What #133 adds beyond #132:

| Addition | Why it is not #132 |
| --- | --- |
| **rmcp ≥ 3.9.0** | RUSTSEC-2026-0462. Published crates and the template daemon are on `~3.8`; two test manifests are `=3.8.0`. Latest is **3.9.0**. |
| **Exact pins already sitting on latest** | `utoipa =6.0.0`, `utoipa-axum =0.3.0`, `schemars =1.2.3`, `sc-observability` / `sc-observability-types =1.5.0`. No newer release exists. The `=` still blocks the next release of those crates. There is **no utoipa 7** on crates.io. |
| **Floors for wide carets** | `tokio = "1"`, `serde = "1"`, `tracing = "0.1"`, and the same pattern elsewhere. Latest already resolves. Minimal-versions does not, until the floor is explicit. |
| **Template workspace** | `template/Cargo.toml` and `template/crates/**` are a second resolve (no checked-in template lockfile). #132 does not edit them. |
| **Dual-end CI for the whole graph** | d-1's check is `cargo update -p rusqlite --precise 0.41.0` plus the high 0.41.x pin. #133 asks for minimal-versions and `cargo update` across manifests. |
| **Publish** | Stays in d-2. #133's "consumer can cargo update" is true only after **0.2.1** is on crates.io. |

#133 does not lift the frozen sqlx ruling. sqlx 0.9.0 is already crates.io latest, and `sqlx-sqlite` 0.9.0 still requires `libsqlite3-sys >=0.30.1, <0.38.0`. Rusqlite 0.41 (libsqlite3-sys ^0.39) and 0.42 (^0.40) cannot share a process with template `store-sqlite` on this line.

**Rusqlite compatibility envelope.** d-1 tests 0.41.0 and the snapshot's
0.42.0 in both the root and consumer workspaces. If both qualify, declare
`>=0.41.0, <0.43`, spanning the two tested pre-1.0 compatibility lines. The
upper boundary excludes the next unqualified Cargo compatibility line. If
0.42 fails, keep `"0.41.0"` and document the actual incompatibility; no
unbounded future-line support is claimed. d-1 owns this decision and the
matching manifest comments, tests, and ADR amendment.

## 2. Inventory

`BLOCKED?` means 0.2.1 cannot take the crates.io latest in the "latest" column. Lock versions are from the root `Cargo.lock` unless noted. Template crates are not in that lockfile.

### Registry dependencies

| Crate / path | Dependency | Current spec | Latest crates.io | Proposed range | BLOCKED? | Blocker reason |
| --- | --- | --- | --- | --- | --- | --- |
| `crates/sc-cron`, `crates/sc-runtime-db-sqlite`, `tests/integration/cron-consumer` | rusqlite | `"0.39"` (lock 0.39.0 → libsqlite3-sys 0.37.0) | **0.42.0** (0.41.0 → libsqlite3-sys ^0.39; 0.42.0 → ^0.40) | `>=0.41.0, <0.43` after 0.42 qualification; otherwise `"0.41.0"` with the observed failure documented | Conditional | Same-process sqlx is already unsupported at 0.41 (d-1 ruling), so sqlx does not cap rusqlite. 0.42 is same-day and outside the frozen `"0.41"` caret. d-1 must qualify hooks and consumer type/native-link compatibility before admitting the 0.42 line. |
| `template/crates/store-sqlite`, `store-postgres`, `store-mysql`; `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve` | sqlx | `"0.9.0"` | **0.9.0** | `"0.9.0"`; retain the supported 0.9 compatibility line and explain the native-link limitation. Delete sqlx-resolve in d-1. | Yes (transitive) | `sqlx-sqlite` 0.9.0 requires `libsqlite3-sys >=0.30.1, <0.38.0` (crates.io dependency API). No newer sqlx release lifts that. Postgres/MySQL rows are the same crate major, called out in-manifest as a per-engine direct pin (beads `run-ze5` is a later hoist, not a bump). |
| `crates/sc-runtime` (dep + dev), `crates/sc-command`, `template/crates/daemon` | rmcp | `~3.8` (lock 3.8.0) | **3.9.0** | `"3.9.0"` | No | Advisory floor, not a ceiling. The normal 3.x compatibility boundary admits compatible fixes without promising 4.x support. d-1b must recompile `StreamableHttpService`, `ServerHandler`, `Tool::new`, `schema_for_input`, `tool` / `tool_router` macros. |
| `tests/contracts/phase-c/application-control`, `tests/integration/cron-consumer/schema` | rmcp | `=3.8.0` | **3.9.0** | `"3.9.0"` | No | Same advisory. These manifests are separate lockfiles and must match the published range. |
| `crates/sc-runtime` (dep + dev), `crates/sc-command`, `crates/sc-cron-commands`, `template/Cargo.toml` `[workspace.dependencies]` | utoipa | `=6.0.0` | **6.0.0** | `"6.0.0"` | No | Already latest. No 7.x. The `=` is the defect. The normal Cargo 6.x boundary preserves compatibility; an extra tighter cap needs a known break. |
| `crates/sc-runtime` (dep + dev), `template/crates/daemon` | utoipa-axum | `=0.3.0` | **0.3.0** | `"0.3.0"` | No | Already latest. Pairs with utoipa 6. |
| `crates/sc-runtime/tests/fixtures/utoipa5-router` | utoipa-axum | `=0.2.0` | 0.3.0 (this fixture must not follow it) | Keep `=0.2.0` | Yes | Negative compile fixture. Comment in `src/lib.rs`: "Intentionally fails: runtime 0.2 accepts the Utoipa 6 router identity." Moving it to 0.3 makes the rejection test pass for the wrong reason. |
| `crates/sc-command`, `crates/sc-cron-commands` | schemars | `=1.2.3` | **1.2.3** | `"1.2.3"` | No | Already latest. |
| `template/Cargo.toml` workspace | schemars | `"1"` (caret) | **1.2.3** | `"1.2.3"` | No | Align the template with the published crates so minimal-versions does not select 1.0.0. |
| `crates/sc-command`; `template/Cargo.toml` workspace (`sc-observability` and `sc-observability-types`) | sc-observability, sc-observability-types | `=1.5.0` (command overrides the workspace caret) | **1.5.0** | `"1.5.0"` | No | Already latest. Command comment cites ADR-RUN-0017 / issue #97 for the **edge**, not for the `=` operator. Root workspace spec is already `"1.5.0"` (caret). |
| `crates/sc-runtime`, `crates/sc-command`, `crates/sc-transport`, `template/crates/daemon` | axum | `"0.8.9"` (lock 0.8.9) | **0.8.9** | `"0.8.9"` | No | Latest equals the pin. 0.9 is unpublished. |
| `crates/sc-runtime`, `sc-command` (dev), `sc-transport`, `sc-cron`, `sc-cron-commands` (dev), cron-consumer, custom-context, logging_consumer, application-control, `template/Cargo.toml` workspace | tokio | `"1"` (root lock **1.54.1**) | **1.54.1** | `"1.54.1"` | No | Latest equals the lock. Spec `"1"` makes minimal-versions select 1.0.0. |
| Root and template manifests using `serde` / `serde_json` `"1"` | serde, serde_json | `"1"` (lock 1.0.229 / 1.0.154) | **1.0.229** / **1.0.154** | `"1.0.229"` and `"1.0.154"` | No | Same minimal-versions floor problem. Latest equals the lock. |
| `sc-runtime`, `sc-cron`, template workspace | tracing | `"0.1"` (lock 0.1.44) | **0.1.44** | `"0.1.44"` | No | 0.1 line is the published tracing line. Floor must not be 0.1.0 under minimal-versions. |
| `sc-cron`, template daemon/cli, cron-consumer, logging_consumer | async-trait | `"0.1"` (lock 0.1.92) | **0.1.92** | `"0.1.92"` | No | Latest equals the lock. |
| `sc-runtime`, `sc-cron`, template stores/cli/daemon dev | tempfile | `"3"` (lock 3.27.0) | **3.27.0** | `"3.27.0"` | No | Latest equals the lock. |
| `sc-runtime` dev, template not directly | tower | `"0.5"` (lock 0.5.3) | **0.5.3** | `"0.5.3"` | No | Latest equals the lock. |
| `sc-runtime` dev, `sc-transport`, `template/crates/cli` dev | reqwest | `"0.13"` (lock 0.13.5) | **0.13.5** | `"0.13.5"` | No | Latest equals the lock. `"0.13"` is a 0.x minor (`<0.14`). |
| `sc-cron`, cron-consumer dev, template cli dev, logging_consumer | chrono | `"0.4"` (lock 0.4.45) | **0.4.45** | `"0.4.45"` | No | Latest is still 0.4.x. |
| `sc-cron` | cron | `"0.17"` (lock 0.17.0) | **0.17.0** | `"0.17.0"` | No | Latest equals the lock. Single 0.x minor today; `>=` lets a later 0.17 patch or a reviewed newer minor through. |
| `sc-cron` | getrandom | `"0.4"` (lock also has transitive 0.2.17 and 0.3.4) | **0.4.3** | `"0.4.3"` on the direct dep | No | Direct requirement already allows 0.4.3. Older getrandom copies are transitive; do not try to collapse them in 0.2.1. |
| `sc-runtime` | fd-lock | `"4"` (lock 4.0.4) | **4.0.4** | `"4.0.4"` | No | Latest equals the lock. |
| `sc-cron` dev | libc | `"0.2"` (lock 0.2.191) | **0.2.191** | `"0.2.191"` | No | Latest equals the lock. `"0.2"` minimal-versions would select an ancient 0.2.0. |
| `sc-cron`, cron-consumer dev | process-wrap | `"10"` (lock 10.0.1) | **10.0.1** (`max_stable`; 8.2.2 / 9.1.1 are older majors touched the same day) | `"10.0.1"` | No | Already on the latest stable major. `<11` matches the major the code is written against (Unix session / job-object features). 8.x/9.x are not an upgrade target. |
| `sc-cron` optional | monty, monty-types | `"1.4.0"` (caret, lock **1.4.0**) | **1.6.0** | `>=1.4.0, <1.6` | Yes for 1.6.0 | monty 1.4.0 and 1.5.0 declare rust-version **1.96**. **1.6.0 declares rust-version 1.98.** Workspace monty CI is `+1.96.1` (`.github/workflows/crates.yml`). `cargo update` on the current caret pulls 1.6.0 and breaks that job. 1.5.0 is the highest version inside the toolchain. Comment the cap. |
| `sc-cron` and cron-consumer, Windows dev | win32job | `=2.0.3` | **2.1.0** | `"2.0.3"` | No | Exact pin on a test-only crate. 2.1.0 is a minor. Prove it on the existing Windows hook job; cap only if that job fails. |
| same | waitpid-any | `=0.3.0` | **0.3.0** | `"0.3.0"` | No | Exact pin equals latest. |
| `template/crates/store-sqlite`, `store-postgres`, `store-mysql` | futures-util | `"0.3.31"` | **0.3.34** | `"0.3.31"` | No | Caret already allows 0.3.34. Not in the root lockfile. |
| `template/Cargo.toml` workspace | clap | `"4.5"` | **4.6.7** | `"4.5.0"` | No | Caret already allows 4.6.7. Explicit floor keeps minimal-versions off 4.0.0. |
| `template/Cargo.toml` workspace (`toml = "0.8"`), used by `template/crates/schema` and store-adaptor | toml | `"0.8"` | **1.2.1+spec-1.1.0** (0.8 line ends at **0.8.23**; 0.9.12 exists) | Keep `"0.8.23"` | Yes | `toml::from_str` in `template/crates/schema/tests/common/membership_toml.rs` and the adaptor membership path. 0.9 and 1.x are breaking lines. Out of 0.2.1. |
| `template/Cargo.toml` workspace | sc-config, sc-transport, sc-command, sc-runtime, sc-cron, sc-cron-commands | `"0.2.0"` | published 0.2.0; this repo's next release is 0.2.1 | d-2 sets `"0.2.1"` | n/a | These are this repo's own crates. A `>=0.2.0` range would let a generated app resolve a future 0.3 during 0.2.1. d-2 publish tail bumps them with the release. |

### In-repo path versions (not a crates.io sweep)

Path dependencies repeat `version = "0.2.0"` (workspace crates) or `version = "0.1.1"` (template-internal crates: schema, stores, daemon, service, api-types, adaptors). `tests/contracts/phase-c/application-control` uses `=0.2.0` on path `sc-cron` and `sc-cron-commands`. Those strings are package identity for publish and cargo-generate. d-2 bumps the workspace side to 0.2.1. They are not #133 registry ranges.

Manifests with no direct third-party pin of their own: `crates/sc-cron-commands/tests/fixtures/minimal`, `template/crates/api-types`, `template/crates/service`, `template/crates/store-adaptor`, `template/crates/store-adaptor-sqlite-postgres`, `template/crates/store-adaptor-sqlite-mysql`. They inherit workspace specs or path crates only.

## 3. Sprint placement

### d-1 (with rusqlite) — libsqlite3-sys lockfile only

Owns the crates that select `libsqlite3-sys` in a lockfile shared with rusqlite:

| Path | Change |
| --- | --- |
| `crates/sc-cron/Cargo.toml` | rusqlite line only → `>=0.41.0, <0.43` after 0.42 qualification, otherwise `"0.41.0"` with the known incompatibility documented. Replace the 0.39 / sqlx-sharing comment with the frozen ruling. |
| `crates/sc-runtime-db-sqlite/Cargo.toml` | same, features stay `bundled` without `hooks`. |
| `tests/integration/cron-consumer/Cargo.toml` | rusqlite line only. Separate `Cargo.lock`. |
| `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve/` | **Delete.** Its lockfile is the coexistence gate d-1 removes. |

d-1 does not retarget axum, tokio, rmcp, monty, or template sqlx. High/low proof for this graph: minimal end rusqlite **0.41.0**, high end **0.42.0** when qualified (libsqlite3-sys 0.40); otherwise latest compatible 0.41.x. The final range and both endpoint resolves must agree.

`tests/integration/cron-consumer/Cargo.lock` and the root `Cargo.lock` both move. Template `store-sqlite` is a different workspace and stays on sqlx 0.9.

### d-1b (wave-1 sibling) — rmcp and the other non-sqlite ranges

Parallel to d-1 on issue ownership. Both edit lockfiles; merge-forward the second onto `integrate/phase-d` before the d-2 publish tail. d-1b owns all non-rusqlite manifest ranges, including the sc-cron lines below; without the Monty cap, `cargo update` in d-1b will pull **monty 1.6.0** and fail the 1.96.1 job.

| Bucket | Paths | Work |
| --- | --- | --- |
| MCP stack | `crates/sc-runtime/Cargo.toml`, `crates/sc-command/Cargo.toml`, `template/crates/daemon/Cargo.toml`, `tests/contracts/phase-c/application-control/Cargo.toml`, `tests/integration/cron-consumer/schema/Cargo.toml` | `rmcp = "3.9.0"` (normal compatible range). Recompile the call sites named in the table. |
| OpenAPI / schema exact pins | `sc-runtime`, `sc-command`, `sc-cron-commands`, `template/Cargo.toml`, `template/crates/daemon` | Normal compatible requirements at tested floors: utoipa 6.0.0, utoipa-axum 0.3.0, schemars 1.2.3. Leave `utoipa5-router`'s `=0.2.0` alone. |
| Observability exact pins | `crates/sc-command/Cargo.toml`, `template/Cargo.toml` | Normal compatible requirements at floor 1.5.0 for both observability crates. |
| Floors that minimal-versions would otherwise sink | Every `"1"` / `"0.1"` / `"0.4"` / `"0.5"` / `"0.13"` / `"3"` / `"4"` row in the table, including template `clap` and `futures-util` | Raise to the locked or latest-on-line floor. |
| monty cap | `crates/sc-cron/Cargo.toml` optional deps only | `>=1.4.0, <1.6` plus the rust-version comment. Optional: lock the high end at 1.5.0 if that compile is clean on 1.96.1. |
| Windows test exact pins | `sc-cron` and `tests/integration/cron-consumer` target dev-deps | Normal compatible requirements at win32job 2.0.3 and waitpid-any 0.3.0; qualify Windows tests. |
| CI | `Justfile`, and a job or step in `.github/workflows/crates.yml` | Section 4. |

d-1b does not edit rusqlite lines and does not publish.

### d-2 publish — unchanged

README, #131 docs, workspace version **0.2.1**, template `sc-*` version strings, kit input, crates.io publish. No third-party range work. The publish tail consumes the merged d-1 and d-1b lockfiles. #133 is done for consumers only after that publish.

### Defer (out of 0.2.1)

| Item | Why it waits |
| --- | --- |
| sqlx major, or any sqlx that accepts `libsqlite3-sys` ≥ 0.39 | No such release. Frozen ruling: same-process template store + writer is unsupported on 0.2.1. Follow-on sprint. |
| Template `store-*` manifest edits whose only purpose is a sqlx range comment | d-1 already states the ruling in ADR-RUN-0012 and deletes sqlx-resolve. Avoid a second editor on template sqlx inside 0.2.1. |
| toml 0.9 / 1.x | Breaking line. Template tests call `toml::from_str`. Stay on 0.8.23. |
| monty 1.6.0 | Needs rustc 1.98. Cap in d-1b; bump the toolchain in a later sprint. |
| utoipa-axum 0.2 fixture | Stays an exact negative pin. |
| Workspace hoist of sqlx (beads `run-ze5`) | Comment in `store-postgres` / `store-mysql`. Not a version bump. |
| In-repo path `0.2.0` / `0.1.1` identity | d-2 for the published 0.2.1 numbers only. |
| getrandom 0.2 / 0.3 transitive copies | Not direct requirements. |

## 4. CI / process

What exists today:

- `.github/workflows/crates.yml` job `crates` ("just lint and just test") installs stable **1.94.1** and **1.96.1**, then `just lint` and `just test`. `just lint` always runs `just supply-chain` (`cargo deny check`, `cargo audit`) from the root `Justfile`. There is no dependency-resolution matrix.
- Monty tests in that workflow and in `Justfile` `test windows-hook-monty` / `windows-host-monty` use `cargo +1.96.1`. A lockfile that resolves monty 1.6.0 fails those steps.
- d-1's written check is sqlite-only: `cargo update -p rusqlite --precise 0.41.0`, test `sc-cron` and `sc-runtime-db-sqlite`, restore the high pin, repeat. It does not run `-Z minimal-versions`.
- `.github/workflows/template.yml` job `generate` renders each `wizard/fixtures/*.json` and runs `just test` in the generated tree (`SQLX_OFFLINE=true`). The `sqlite-mcp.json` cell also runs `just test generated-general-db-off`. That is a **latest-within-range** resolve. The template has no `Cargo.lock`, so nothing pins a low end.
- `Justfile` `upgrade` is the sc-lint bootstrap upgrade, not `cargo update`.

How to extend, without a second workflow file:

1. **d-1 keeps a named sqlite pair** until the final tested rusqlite range is declared. After that, the workspace job below is the pair: minimal-versions resolves 0.41.0, `cargo update` resolves 0.42.0 (or 0.41.x if the cap stays). Document both commands in the d-1 PR. Do not commit the minimal lockfile.
2. **d-1b adds two `Justfile` recipes** the crates workflow can call. Sketch:

```just
# High end. Refuses to run if monty resolves to 1.6 while CI is on 1.96.1.
deps-latest:
    cargo update
    just test

# Low end. Nightly only. Restores Cargo.lock afterward.
deps-minimal:
    cargo +nightly update -Z minimal-versions
    just test
```

3. **Wire them from `crates.yml`**, same job family as `just lint` / `just test`, as two extra steps or a two-cell matrix (`resolution: [locked-latest, minimal]`). The minimal cell checks out a clean lock, runs the nightly update, tests, and does not upload the lock. The latest cell runs `cargo update` then the existing `just test` (including the 1.96.1 monty tests). Gate the new steps on `needs.ci-scope.outputs.runtime == 'true'` like the current lint step.
4. **Honest minimal-versions limit.** `-Z minimal-versions` also drops transitive crates to their minimum. Direct floors at the current lock remove the `tokio` 1.0.0 failure mode and still may fail in a transitive crate this repo does not claim to support. If that happens, record the crate and the error in the PR. The fallback that still matches #133's "direct floor is tested" claim is `-Z direct-minimal-versions` (direct deps at the floor, transitive resolve normal). Use it only after full minimal-versions has a named transitive failure. Do not weaken the recipes in advance.
5. **Template.** Extend the existing `sqlite-mcp.json` cell (it already builds daemon + MCP) with one low-end invocation inside the generated app: `cargo +nightly update -Z direct-minimal-versions` then `cargo test` for the daemon/MCP packages, and one high-end `cargo update` before the cell's current `just test`. Generated `store-sqlite` stays on sqlx 0.9.0 at both ends, because that is latest. rmcp's low end in that cell must be 3.9.0, not 3.8.0.
6. **Separate lockfiles** that `cargo update` at the repo root does not touch, and that d-1b must include in the same recipes: `tests/integration/cron-consumer/Cargo.lock`, `tests/integration/cron-consumer/schema/Cargo.lock`, `tests/contracts/phase-c/application-control/Cargo.lock`, `crates/sc-runtime/tests/fixtures/utoipa5-router/Cargo.lock` (leave its utoipa-axum pin exact), cron-consumer `custom-context` and `sc-cron-commands` minimal fixture if their tokio floor changes.
7. **Deletion condition for the extra CI.** Remove the nightly minimal-versions step when Cargo supports a stable minimal-versions flag and the ranges have a release of history on 0.2.x. The `cargo update` high end stays; it is ordinary lockfile maintenance, not a special mode.

d-1 and d-1b should not both rewrite `Cargo.lock` on the integration branch without a merge-forward. The dual-end job runs on `integrate/phase-d` after both land (d-2 publish tail already waits on that lockfile). Each sprint PR still runs the pair for the packages it owns.

## 5. Cannot upgrade to latest

Evidence is the manifest comment, the crates.io dependency requirement, or the frozen sprint ruling.

1. **sqlx cannot follow libsqlite3-sys to 0.39 / 0.40.**  
   `crates/sc-runtime-db-sqlite/Cargo.toml` (on `develop`):

   > 0.39 resolves libsqlite3-sys 0.37, which sqlx 0.9 sqlite can link (`>=0.30.1, <0.38.0`). 0.40 pulls libsqlite3-sys 0.38 and cannot share a process with template store-sqlite.

   `crates/sc-cron/Cargo.toml`:

   > 0.39 resolves libsqlite3-sys 0.37, which can share a process with sqlx 0.9.

   Confirmed on crates.io: `sqlx-sqlite` 0.9.0 depends on `libsqlite3-sys` **`>=0.30.1, <0.38.0`**. `sqlx` `max_stable` is **0.9.0**. Rusqlite 0.41.0 depends on `libsqlite3-sys` ^0.39; rusqlite 0.42.0 depends on ^0.40. There is no published sqlx to upgrade to that accepts the advisory libsqlite3-sys.

2. **Template `store-sqlite` stays on that sqlx.**  
   `template/crates/store-sqlite/Cargo.toml` depends on `sqlx` 0.9.0 with the `sqlite` feature (no extra comment; the window is the writer crate's comment plus #132). `store-postgres` and `store-mysql` repeat `sqlx` 0.9.0 and say the pin is per-engine because features differ and the workspace manifest is not hoisted (beads `run-ze5`). Frozen d-1 ruling: same-process sharing of sqlx 0.9 sqlite and the writer at rusqlite 0.41 is **not supported** on 0.2.1. A project that enables both needs a later alignment sprint. Phase D proves cron + writer only.

3. **`sqlx-resolve` is removed, not bumped.**  
   `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve/Cargo.toml` description: "Resolve gate: rusqlite 0.39 and sqlx 0.9 sqlite share one libsqlite3-sys." It depends on path `sc-runtime-db-sqlite` plus `sqlx` 0.9.0 `runtime-tokio` + `sqlite`. d-1 deletes the fixture and the architecture text that treats one `libsqlite3-sys` as a supported configuration. #133 does not replace it with a newer sqlx.

4. **monty 1.6.0.**  
   Current spec `"1.4.0"` is a caret, lockfile is 1.4.0, crates.io latest is 1.6.0 with `rust_version` 1.98. CI installs 1.96.1 specifically because "sc-cron Monty hooks (`monty` 1.4) require rustc 1.96". 1.5.0 is still rust-version 1.96 and is the furthest high end this toolchain can claim. Cap `<1.6`.

5. **toml 1.2.1 (and 0.9.x).**  
   Template workspace `toml = "0.8"` resolves at most **0.8.23**. Call site is `toml::from_str` in `template/crates/schema/tests/common/membership_toml.rs`, used by schema tests and `template/crates/store-adaptor`. 0.9 and 1.x are new major lines (`1.2.1+spec-1.1.0`). Defer.

6. **utoipa-axum 0.2 fixture, on purpose.**  
   `crates/sc-runtime/tests/fixtures/utoipa5-router/Cargo.toml` pins `utoipa-axum = "=0.2.0"`. Production code is already on latest utoipa **6.0.0** and utoipa-axum **0.3.0**. Do not move the fixture to latest.

7. **Rusqlite 0.42 is not in the frozen d-1 snippet.**  
   This is a process block, not an sqlx block. `"0.41"` cannot resolve 0.42.0. d-1 either qualifies 0.42 and publishes `>=0.41.0, <0.43`, or retains the normal `"0.41.0"` range with the actual incompatibility documented. Shipping another bare `"0.41"` fails #133's done-when for this dependency.

Not on this list: **rmcp 3.9.0** (that *is* latest; the work is to leave `~3.8`), **utoipa 6 / schemars 1.2.3 / sc-observability 1.5.0 / axum 0.8.9 / tokio 1.54.1** (already latest; the work is to delete `=` and to stop bare carets from collapsing under minimal-versions).

## Low-hanging fruit: dependency elimination

**Ask:** delete dependencies from published normal graphs (`cargo tree -p <crate> --edges normal` at `c3144cf`), separate from the `>=` floors above. Only high-confidence, small diffs. Published crates: `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`.

**ELIMINATE count: 2.**

The section 2 row that floors `tracing` at `>=0.1.44` still applies to the **template** workspace (`template/crates/daemon/src/routes.rs` calls `tracing::info_span`). It does not apply to `sc-runtime` or `sc-cron` if the deletions below land: those two crates have no `tracing` token outside `Cargo.toml`.

### Default direct edges (normal, depth 1)

| Crate | Direct normal deps |
| --- | --- |
| `sc-config` | `serde`, `serde_json` |
| `sc-transport` | `reqwest`, `serde`, `serde_json`, `tokio` (`axum` only with `server`) |
| `sc-command` | `sc-observability-types` `=1.5.0`, `serde`, `serde_json` (`axum`+`rmcp` with `server`; `schemars`+`utoipa` with `schema`) |
| `sc-runtime` | `sc-transport` (`server`), `sc-command` (`server`), `sc-observability-types`, `axum`, `tokio`, `fd-lock`, `utoipa`, `utoipa-axum`, `serde`, `rmcp`, `tracing` (`tempfile` with `test-util`) |
| `sc-cron` | `sc-command` (defaults off), `sc-observability-types`, `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `rusqlite`, `serde`, `serde_json`, `tempfile`, `tokio`, `tracing` (`monty`+`monty-types` with `monty`) |
| `sc-cron-commands` | `sc-command`, `serde`, `serde_json` (`schemars`+`utoipa` with `schema`; `sc-cron` with `scheduler`) |

`cargo tree -p sc-cron -i tracing` shows `sc-cron` as the only parent. `cargo tree -p sc-runtime -i tracing` also shows `axum`, `rmcp`, `hyper-util`, and `reqwest`.

### ELIMINATE

| Item | Cargo.toml edit | Risk | Sprint | Why it is safe |
| --- | --- | --- | --- | --- |
| `sc-cron` direct `tracing` | Delete `tracing = "0.1"` in `crates/sc-cron/Cargo.toml`. Drop `"tracing"` from `boundaries/sc-cron/sc-cron.toml` `allowed_dependencies` and from the NFR-CRON-0001 sentence in `docs/sc-cron/requirements.md`. | low | **defer** (rider on d-1's `crates/sc-cron/Cargo.toml` edit, or the commit after d-1). Keep it out of d-1b: that sprint does not own this file, and d-1 already edits it for rusqlite. | b-15 removed the calls; the phase-b plan left the manifest for a later cleanup. Removing the line drops `tracing`, `tracing-attributes`, and `tracing-core` from a consumer's `sc-cron` normal tree. |
| `sc-runtime` direct `tracing` | Delete `tracing = "0.1"` in `crates/sc-runtime/Cargo.toml`. Drop `"tracing"` from `boundaries/sc-runtime/sc-runtime.toml`. Update the NFR-RT-0003 exact list and the "request-id span" row in `docs/sc-runtime/requirements.md`, plus the matching sentence in `docs/sc-runtime/architecture.md`. | low | **d-1b** (`crates/sc-runtime/Cargo.toml` is already an owned path). Ship the boundary and NFR edits in the same change so the exact-set text matches the manifest. | b-14 replaced the span with canonical request events. The resolved daemon tree still contains `tracing` through `axum` and `rmcp`; this removes the unused direct edge and the stale NFR row. |

### DEFER (API or feature change)

| Item | What would have to change | Risk | Sprint |
| --- | --- | --- | --- |
| Reqwest on the daemon graph | `sc-runtime` enables `sc-transport` `server` and therefore links unconditional `reqwest` 0.13 (`tower-http`, `url`, `hyper-util`) beside `axum`. Production `sc-runtime` calls `bind` and the resolver. `Client` is used in `testing.rs` (`test-util`). Split `client = ["dep:reqwest"]` into `default`, `#[cfg(feature = "client")]` on `mod client` and `pub use client::Client`, then set sc-runtime to `default-features = false, features = ["server"]` and add `sc-transport/client` to `test-util`. `serde_json` is referenced only from `client.rs`, so it moves with `client`; NFR-TRN-0001's default direct set (`tokio`, `reqwest`, `serde`, `serde_json`) stays true while `client` remains a default feature. `tests/dependencies.rs` asserts those exact direct names. ADR-TRN-0001 keeps `Client` on reqwest. | med | defer |
| `async-trait` on `sc-cron` | Public `Job` and the other async traits are `#[async_trait]`. Native async fn in traits (rust-version 1.94) drops the proc-macro and changes downstream impls. | med | defer |
| One schema stack | `schema` on `sc-command` and `sc-cron-commands` implements both `schemars::JsonSchema` and `utoipa::ToSchema` (`envelope.rs`, `error.rs`, `contract.rs`, all `#[cfg(feature = "schema")]`). Dropping either crate removes a published schema impl. d-1b only widens the existing floors. | med | defer |

### BLOCKED (boundary or ADR)

| Edge | Why it stays |
| --- | --- |
| `sc-cron` → `sc-command` (defaults off) and `sc-command`'s `sc-observability-types = "=1.5.0"` | ADR-RUN-0017 and issue #97. Both Cargo.toml comments say to restore the forbidden cron→command edge after upstream proof. `error.rs` uses the types on the default envelope path. |
| `sc-cron` → `sc-observability-types` | Same ADR lists it as a required edge. `observability.rs` re-exports the crate, so the direct dep is the public surface. |
| `sc-runtime` `utoipa` + `utoipa-axum` | NFR-RT-0003. The routes closure returns `OpenApiRouter`, and `router.rs` names `utoipa::openapi::OpenApi` for `/openapi.json`. This is the daemon API, including the template. |
| `rmcp` on `sc-runtime` and `sc-command` `server` (`axum` + `rmcp`) | `.mcp(StreamableHttpService)` and sealed `IntoMcp`. `rmcp` 3.8 also pulls `schemars` into the `sc-runtime` tree with `schema` left off; that transitive crate is upstream. |
| `rusqlite` on `sc-cron` | The scheduler store. The sqlx 0.9 same-process cap stays with d-1. |
| Three direct `axum` edges (`sc-runtime`, `sc-transport` `server`, `sc-command` `server`) plus `utoipa-axum` | One resolved `axum` 0.8.9. Each crate names axum types in its own API (`Router` / `serve::Listener` / `IntoResponse`). Reqwest stays the CLI client (ADR-TRN-0001). |

### Checked, no elimination

- `sc-config` is already `serde` + `serde_json`.
- `sc-cron-commands` default graph stops at `sc-command`, `serde`, and `serde_json`. `schema` and `scheduler` are off by default. `tokio` is dev-only.
- `utoipa` and `schemars` are already off the default graphs of `sc-command` and `sc-cron-commands`.
- Dev-dependencies stay off `--edges normal`. `sc-runtime`'s `reqwest`, `tower`, and `serde_json` are dev-only (`tower::ServiceExt` is inside `#[cfg(test)]` in `mcp_input.rs`). `sc-cron`'s `libc`, `win32job`, and `waitpid-any` are dev-only. `tempfile` on `sc-runtime` is `test-util`. `tempfile` on `sc-cron` is production (`TempCwd` in `process.rs`).
- `sc-transport` `[dev-dependencies]` repeats `reqwest`, `serde`, and `serde_json` with the same features and adds `rt-multi-thread` on `tokio` for tests. Deleting the identical lines leaves the published normal graph unchanged.
- Template `async-trait` is already behind the `cron` feature. Template `tracing` stays (see the floor note at the top of this section).
- Workspace `path` + `0.2.0` edges are the publish coupling in the pin inventory above. The deprecated edge is the ADR-RUN-0017 cron→command path, which stays blocked until #97.
