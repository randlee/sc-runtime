# Phase B publication verification

Verified 2026-11-03 against live crates.io API, downloaded published crate archives, GitHub release metadata and PR103. Read-only investigation; no repository or consumer changes/builds/publication.

## Result

Phase B WAS published, but as **0.1.1**, not **0.2.0**. No 0.2.0 appears in any of the five public package version lists. All five existing versions are non-yanked.

| Package | Published version | Created UTC | Registry evidence |
| --- | --- | --- | --- |
| sc-config | 0.1.1 | 2026-11-02T03:08:41.769545Z | https://crates.io/api/v1/crates/sc-config |
| sc-transport | 0.1.1 | 2026-11-02T03:09:57.673787Z | https://crates.io/api/v1/crates/sc-transport |
| sc-command | 0.1.1 | 2026-11-02T03:11:10.758009Z | https://crates.io/api/v1/crates/sc-command |
| sc-runtime | 0.1.1 | 2026-11-02T03:12:47.321178Z | https://crates.io/api/v1/crates/sc-runtime |
| sc-cron | 0.1.1 | 2026-11-02T03:14:07.962777Z | https://crates.io/api/v1/crates/sc-cron |

These publication timestamps are **November 1 Pacific**. Both sc-runtime-db-sqlite and sc-cron-commands registry endpoints return404. The former is explicitly `publish = false` at v0.1.1; the latter is a new planned package. Absence of the writer crate is therefore not evidence of a failed public release.

## Source provenance proves Phase B association

Every downloaded 0.1.1 crate contains `.cargo_vcs_info.json` with git SHA **5af24c862fe7ed8dae8710b7bd64a84a29506b51**. This is both tag v0.1.1 and the merge commit of [PR103](https://github.com/randlee/sc-runtime/pull/103), whose description explicitly says “Prepare the 0.1.1 release from completed Phase B work on develop.”

[GitHub release v0.1.1](https://github.com/randlee/sc-runtime/releases/tag/v0.1.1) was published 2026-11-02T05:57:34Z (Nov1 21:57 Pacific). It includes Phase B storage, cron port, Monty, logging/lifecycle and environment-policy work through PR96 plus release compatibility work. Release candidate/source ancestry and published archives agree; this is not merely inference from a manifest or tag.

Current fetched origin/develop f5eef8d and integrate/phase-c 725420b both have workspace version0.1.1. The primary checkout is old develop99772cf, version0.1.0; it is not reliable evidence of the latest published release.

## beads-cache blocker

Human report “beads-cache is blocked because0.2.0 is not available” agrees with registry facts. Availability of0.1.1 does not establish that required new APIs exist: the published sc-runtime archive has run_with_cleanup but no run_with_cleanup_and_bound or start_with_cleanup; published sc-cron root exports have no public canonical logging facade. These are explicitly Phase C planned APIs.

The main beads-cache checkout still has its own local sc-cron package version0.2.0 (`path = crates/sc-cron`) and a git-tagged sc-command dependency. That local package version must not be confused with a crates.io release from sc-runtime. It may be stale relative to the active beads-cache worktree, so it is a possible source of naming confusion, not proof of the active consumer configuration.

## Consequence for Phase C planning

The Phase C statement “0.1.1 ->0.2.0” is consistent with current registry state; it is not attempting to republish an occupied0.2.0 version. Do not choose another version on the assumption0.2.0 already shipped. Keep Phase B completed/publication0.1.1 distinct from future Phase C API publication. Version availability must be rechecked at the actual release freeze and publication gate. No plan edits or new release authorization were performed by this investigation.
