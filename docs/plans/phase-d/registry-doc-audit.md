# Registry documentation audit (2026-11-11)

**Scope:** Published crate docs for the sc-runtime workspace vs crates.io (HTTP API + published crate artifacts).  
**Authority for publish set:** [`release/sc-publish-consumer-input.json`](../../release/sc-publish-consumer-input.json).  
**Planning authority:** deliverables and acceptance criteria live only in [sprint d-2](sprint-d-2-consumer-docs.md); published-artifact proof belongs to [d-3](sprint-d-3-release.md); this file is evidence/inventory.
**Evidence run:** [Composer registry doc audit](8e70524e-4434-4614-a2de-a8184248a41b) (confirmed README tarball byte-match @ 0.2.0).

## Method

1. **crates.io JSON API** (`curl`, `User-Agent: sc-runtime-doc-audit/1.0`):  
   `GET https://crates.io/api/v1/crates/<name>` and `GET .../<name>/0.2.0` for metadata (`description`, `rust_version`, `license`, `repository`, `homepage`, yanked state, downloads).
2. **Published README body:** downloaded `https://static.crates.io/crates/<name>/<name>-0.2.0.crate`, extracted `README.md` from the tarball, SHA-256 compared to `crates/<name>/README.md` in this checkout.
3. **`Cargo.toml` one-liners:** compared local `description =` to API `description` on the 0.2.0 version record.
4. **PyPI:** read `python_packages` from the same consumer input manifest (no live PyPI calls).

**API quirks observed**

- Version JSON returns `readme: null` for all six 0.2.0 releases (length field empty in API); README still ships inside the `.crate` artifact and renders on crates.io crate pages.
- `https://static.crates.io/readmes/<crate>/<crate>-<ver>.md` returned **404** for every crate; use the tarball (or the crate page) for README truth, not that URL pattern.

**Workspace version note:** `[workspace.package] version = "0.2.0"` in root `Cargo.toml`. Several crate READMEs still say **0.1.0** in their opening publish banner; crates.io **max_version** is **0.2.0** for all published crates.

---

## PyPI

From `release/sc-publish-consumer-input.json`:

```json
"python_packages": [],
"python_distributions": []
```

**Finding:** This repository's sc-publish kit does **not** publish Python packages for the 0.2.x line. **No PyPI documentation audit or d-2 PyPI action** unless a separate product adds entries here later.

---

## crates.io — published @ 0.2.0

All six kit crates exist on crates.io, `yanked: false`, `rust_version: 1.94.1`, MIT, `repository` / `homepage` → `https://github.com/randlee/sc-runtime`. First publish wave: **2026-11-06** (UTC).

| Crate | `readme` in `Cargo.toml` | Local README = 0.2.0 tarball README | API `description` = local `Cargo.toml` | Stale **0.1.0** banner in README | Framework blurb |
| --- | --- | --- | --- | --- | --- |
| sc-config | yes | **yes** (byte-identical) | yes | **yes** (line 3) | **missing** |
| sc-transport | yes | **yes** | yes | **yes** | **missing** |
| sc-command | yes | **yes** | yes | **yes** | **missing** |
| sc-runtime | yes | **yes** | yes | **yes** | **missing** |
| sc-cron | yes | **yes** | yes | no version banner | **missing** |
| sc-cron-commands | **no** `readme` key | **yes** | yes | no (uses 0.2.0 in dep example) | **partial** (generator path only) |

`sc-runtime-db-sqlite`: **not on crates.io** (`GET .../crates/sc-runtime-db-sqlite` → `"crate does not exist"`). Local `Cargo.toml` has `publish = false`. README line 3 falsely claims crates.io publication @ 0.1.0 — workspace-only doc fix for d-2.

---

## Local README vs registry (substance)

**Sync status:** For every **published** crate, the README in git **is** the README consumers see on crates.io @ 0.2.0. There is no drift between the repository and the registry artifact; gaps are **content and framing**, not publish lag.

**Registry one-liners (`description`)** match local `Cargo.toml` and match the per-crate role text. They do **not** carry the kit **project** description from consumer input:

> Scaffolding framework: config, transport, command envelope, and daemon assembly crates plus a cargo-generate template

That sentence appears in `release/sc-publish-consumer-input.json` → `project.description` and in release tooling as the product frame, but **nowhere** on crates.io today (only per-crate assembly lines).

---

## Scaffolding blurb gaps (`sc-runtime` and siblings)

**Target framing** (from kit + architecture): sc-runtime is a **multi-consumer scaffolding framework** — library crates plus a `cargo-generate` template for JSON-first CLI/daemon projects ([`docs/framework-charter.md`](../../framework-charter.md), [`docs/architecture.md`](../../architecture.md)).  
`readme_dependency_crate = "sc-runtime"` in consumer input names the crate whose README should anchor version examples in release checks; it does **not** inject framework copy into sibling crate READMEs today.

### Per-crate gaps (published @ 0.2.0)

| Crate | What registry shows | Missing vs scaffolding story |
| --- | --- | --- |
| **sc-runtime** | Daemon assembly, builder API, health/openapi/MCP | No mention of **template**, **`just new` / cargo-generate**, or **sibling crates** as a framework stack; opening line still says **0.1.0**. Should be the **primary** “Part of sc-runtime / this is the framework” narrative (`readme_dependency_crate`). |
| **sc-config** | Config loader only | No link that this crate is **generated into consumer projects** from the template; **0.1.0** banner; no “Part of sc-runtime”. |
| **sc-transport** | Transport/endpoints | Same as sc-config. |
| **sc-command** | Envelope + optional server | Same; no cross-link to template MCP/CLI story. |
| **sc-cron** | Long scheduler README | No ecosystem framing; no publish-order / template wizard pointer; does not mention **sc-cron-commands** sibling. |
| **sc-cron-commands** | “When generating from sc-runtime, select `cron=true`…” | Closest to framework language but **no** standardized **Part of sc-runtime** section; `readme = "README.md"` absent in `Cargo.toml` (README still packaged via default discovery). |

### Root `README.md`

Still describes **four crates @ 0.1.0** and “intended for separate crates.io publication once release authorization completes,” while **six** crates are already on crates.io @ **0.2.0** (`sc-cron`, `sc-cron-commands` omitted from the table). This is the main **repository-level** scaffolding gap: consumers landing on GitHub see a different product shape than registry consumers.

---

## Recommended standard section (all six published READMEs)

d-2 requires this block (sprint plan). Insert after the title or replace stale publish banners:

```markdown
## Part of sc-runtime

This crate is part of [sc-runtime](https://github.com/randlee/sc-runtime), a Rust
scaffolding framework for JSON-first daemons, CLIs, generated stores, and optional
MCP. Publish order and template usage are documented in the repository README.
```

**`sc-runtime` README** should expand this section (or lead with it) with one short paragraph on the **template** and pointer to root `README.md` → **Generate a project**.

### Optional `Cargo.toml` description suffix

Keep README as primary; if descriptions are touched at publish time:

> Part of the sc-runtime Rust scaffolding framework.

(Stay within crates.io description length norms.)

---

---

## Evidence snapshot (API)

| Crate | max_version | downloads (crate) | 0.2.0 version downloads |
| --- | --- | --- | --- |
| sc-config | 0.2.0 | 319 | 197 |
| sc-transport | 0.2.0 | 168 | 40 |
| sc-command | 0.2.0 | 270 | 62 |
| sc-runtime | 0.2.0 | 157 | 35 |
| sc-cron | 0.2.0 | 243 | 45 |
| sc-cron-commands | 0.2.0 | 10 | 10 |

Descriptions (0.2.0 API, all match local `Cargo.toml`):

- sc-config — Synchronous JSON configuration loader with environment overrides.
- sc-transport — Endpoint resolution and local HTTP for a daemon and its CLI.
- sc-command — Lossless JSON CLI/API response envelope and REST/MCP conversions.
- sc-runtime — Daemon assembly: singleton lock, one router, and graceful shutdown.
- sc-cron — SQLite-backed cron scheduler with generation fencing, jitter and missed-tick replay.
- sc-cron-commands — Optional versioned cron command DTOs and injectable execution contracts.
