# Registry documentation audit (2026-11-11)

**Auditor:** parent orchestrator (+ [Composer explore audit](8e70524e-4434-4614-a2de-a8184248a41b) in flight).  
**Authority for publish set:** `release/sc-publish-consumer-input.json`.  
**Planning consumer:** [sprint d-2](sprint-d-2-readme-and-release.md), Grok phase-d planning.

## PyPI

- `python_packages`: **[]** — this repository's sc-publish kit does **not** publish Python packages for the 0.2.x SDK.
- **No PyPI action** for phase d unless a separate product adds a package later.

## crates.io (published @ 0.2.0)

| Crate | `readme` in `Cargo.toml` | Local README | crates.io `description` | Framework blurb in README |
| --- | --- | --- | --- | --- |
| sc-config | yes | yes (stale **0.1.0** line) | Synchronous JSON configuration… | **missing** |
| sc-transport | yes | yes (stale **0.1.0**) | Endpoint resolution and local HTTP… | **missing** |
| sc-command | yes | yes (stale **0.1.0**) | Lossless JSON CLI/API envelope… | **missing** |
| sc-runtime | yes | yes (stale **0.1.0**) | Daemon assembly… | **missing** |
| sc-cron | yes | yes (no version banner) | SQLite-backed cron scheduler… | **missing** |
| sc-cron-commands | **no** `readme` key | yes (long) | Optional versioned cron DTOs… | **partial** (mentions generating from sc-runtime) |

API note: `crates.io` version JSON returns `readme` length 0 for these versions
(API quirk); crate pages still render README from the published crate artifact.
**Re-verify on crates.io UI** after 0.2.1 publish.

### Recommended standard section (all six published READMEs)

Insert after the title (or replace stale publish banner):

```markdown
## Part of sc-runtime

This crate is part of [sc-runtime](https://github.com/randlee/sc-runtime), a Rust
scaffolding framework for JSON-first daemons, CLIs, generated stores, and optional
MCP. Publish order and template usage are documented in the repository README.
```

### Cross-cutting gaps

1. Root `README.md` still describes **four crates @ 0.1.0** — does not mention
   `sc-cron`, `sc-cron-commands`, or 0.2.0.
2. No crate README uses "scaffolding framework" language consistently.
3. `sc-cron-commands/Cargo.toml` should set `readme = "README.md"` so the next
   registry upload ships the same doc as siblings.
4. `sc-runtime-db-sqlite` is **unpublished** (`publish = false`) but has a README
   with stale 0.1.0 publish claim — fix in d-2 as **workspace-only** doc (no
   crates.io).

## Phase d — d-2 actions (from audit)

1. Add the **Part of sc-runtime** section to all six published crate READMEs.
2. Normalize version banners to **0.2.x** (and 0.2.1 after release).
3. Set `readme = "README.md"` on `sc-cron-commands`.
4. Update root README crate table + framework one-liner (matches
   `release/sc-publish-consumer-input.json` project description).
5. Post-0.2.1: spot-check each `https://crates.io/crates/<name>` page for README
   render and description text.

## Suggested `Cargo.toml` description suffix (optional)

Append to each one-line `description =` before publish:

> "Part of the sc-runtime Rust scaffolding framework."

(Keep under crates.io description length norms; README section remains primary.)
