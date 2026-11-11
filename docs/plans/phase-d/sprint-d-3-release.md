---
id: d-3
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-release-0-2-1
track: consumer-release
wave: 2
branch: sprint/d-3-release
worktree: sprint/d-3-release
pr_target: integrate/phase-d
recommended_agent: arch-ctm
recommended_model: terra
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-RUN-0702
  - NFR-RUN-0007
  - NFR-RT-0001
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
  - ADR-RUN-0017
  - ADR-RT-0004
github_issues: [131, 132, 133]
parallel_safe: []
must_follow:
  - d-1
  - d-2
  - d-4
must_follow_rationale: Consumes d-1's qualified API/dependency manifests and reproducible lockfiles, d-2's README bodies packaged in those crates, and d-4's qualified generation CLI/action used by the kit generated-consumer preflight. Publication cannot precede the candidate code and docs; this is artifact coupling, not a shared version-baseline preference.
owned_paths:
  - tests/registry/phase-d/**
  - .github/workflows/phase-d-registry.yml
  - docs/releases/v0.2.1.md
  - docs/project-plan.md
  - site/reports/**
paths_to_delete: []
---

# Sprint d-3 — published consumer closure

## Goal and inputs

Deliver the qualified 0.2.1 packages to consumers and prove they unblock the
actual dependency problem. Consume the integrated d-1 source/manifests/locks and
d-2 README content. This sprint does not edit those artifacts; send a source
defect back to its owner and requalify before publishing. No third-party pin or
tighter range is introduced by this release sprint (ANC-016).

Merge-forward parent development when pushed before each dev/fix round; parent
PRs merge before this sprint completes. One final candidate combines these three
artifacts. This release is coupled to those artifacts, not unrelated phase work.

## Deliverables

| Deliverable | Requirement served |
| --- | --- |
| Six 0.2.1 packages produced and published through the configured sc-publish kit, preserving their qualified ranges/features and README content | REQ-RUN-0002/0702, ADR-RUN-0002 |
| Registry-only consumer proof for the affected beads-cache configuration: security clearance and compatible updates with unchanged sc-runtime release | REQ-RUN-0002/0005, NFR-RUN-0007; #132/#133 |
| Registry CI workflow and consumer entrypoint owned together; minimum/latest positive runs after publication, missing release/consumer and failed checks exit nonzero | REQ-RUN-0002, NFR-RUN-0007 |
| Registry compilation of documented `sc-runtime/test-util` instrument_router usage; registry README verification | REQ-RUN-0702, NFR-RT-0001, ADR-RT-0004; #131 |
| Concise v0.2.1 release notes with supported/unsupported native-link configurations and actual validation outcomes | REQ-RUN-0702 |

The registry proof is a separate consumer workspace under this sprint's owned
test path. Exact sc-* 0.2.1 selection identifies the release being tested; it
does not narrow the libraries' third-party requirements. Avoid local path/patch
substitutes. Use existing source and publishing procedures, not a new release framework.

## Consumer subject and invocation

The SQLite proof is a recorded reduction of randlee/beads-cache Phase C at
`02d795cac6261f1f26e3a3429485e10f328fb651`: root Cargo.toml and the
bd-cache-store/bd-cache-sync/bd-cache-cli manifests. Preserve the dependency
features used there; only select the new released SDK and fixed rusqlite line:

```toml
# tests/registry/phase-d/beads-cache/Cargo.toml (standalone workspace)
[workspace]
[package]
name = "phase-d-beads-cache-proof"
version = "0.0.0"
edition = "2024"
[dependencies]
sc-cron = { version = "=0.2.1", default-features = false }
sc-command = { version = "=0.2.1", default-features = false }
rusqlite = { version = "0.41.0", features = ["bundled", "hooks", "functions", "trace"] }
tokio = { version = "1.54.1", features = ["rt", "rt-multi-thread", "macros", "time", "sync"] }
```

Exercise a real SQLite connection plus sc-cron schedule-store/job execution and
sc-command envelope usage derived from that revision's store/sync tests. Do not
add SQLx or the private writer. Assert one libsqlite3-sys identity >=0.39.0
(0.39.x for rusqlite0.41) and registry sources for every sc-* dependency.
The legacy source used rusqlite0.39 and sc-cron/sc-command=0.2.0; those are the
intentional changed requirements in the reduction, not a claim that upstream
beads-cache has already migrated. Record the source revision and differences in
the consumer README. Separate `server` and `helper` consumers under this same
path enable sc-runtime0.2.1 (plus sc-command/server for server and
sc-runtime/test-util for helper); the server graph must resolve rmcp>=3.9.0.
The actual beads-cache reduction has no rmcp dependency. It proves #132; the
server graph supplies positive #133 rmcp coverage, rather than inferring it
from an absent dependency.

This sprint owns both `.github/workflows/phase-d-registry.yml` and
`tests/registry/phase-d/run.py`. The workflow matrix invokes exactly:
`python3 tests/registry/phase-d/run.py --endpoint minimum|latest --version 0.2.1 --source crates-io`.
The runner checks all three manifests, creates isolated endpoint lockfiles,
selects minimum allowed direct dependencies or latest allowed dependencies,
asserts the selections and sources with Cargo metadata, builds/tests, then runs
`cargo deny check advisories` without ignoring the two target advisories.
Use the standard crates.io registry index; no local path, Git dependency,
patch, source replacement or alternate registry is allowed for final proof.
Exit0 means every assertion/test/audit passed; missing consumer/release,
resolution failure, wrong identity or failed check exits nonzero. Tests for
missing-input and failed-command paths are deterministic; the positive proof
is the actual post-publication CI run. No d-1 workflow branch waits on a future
d-3 artifact. Retire this dedicated workflow when the kit's standard
post-publish hook executes the same consumer endpoint checks.

## Publication rehearsal and recovery

Before uploading anything, use the kit's package/preflight procedure to produce
all six `.crate` archives and inspect their normalized manifests and originals.
Unpack exactly those archives in a temporary rehearsal workspace and run the
same consumers/tests with rehearsal-only patches pointing to those unpacked
archives. Record archive hashes. This verifies package contents and cross-crate
API behavior; it is explicitly rehearsal, not registry evidence. Discard the
patches/config and resolve the final proof afresh against crates.io after upload.
The exact uploaded archives must match the rehearsed hashes.

On mid-order transport/index failure, stop the sequence, query which exact
versions/hashes are already published, and retry only missing uploads of the
same approved archives. Do not republish an occupied version or overwrite a
package. A real package/consumer defect stops the remaining uploads; the release
owner yanks newly published defective versions (and new siblings requiring them)
when leaving them selectable would break consumers or retain a target advisory.
Do not yank merely for a transient CI/network failure or unrelated audit entry.
An occupied defective0.2.1 requires a coordinated next patch release of all six
crates; d-1's manifest owner owns that bump and lock/source fix, d-2 owns any doc
correction, and d-3 repeats rehearsal/publication/proof at the replacement
version. Update release notes/version arguments together; never claim the
requested0.2.1 succeeded or close132/133 on a partial/failed release. Recovery
reopens source work, not a preplanned extra serial wave.

## Acceptance and validation

1. **integration:candidate** — The final integrated candidate passes
   `just lint && just test` and required minimum/latest dependency CI. Existing
   boundary/production-tool guards exclude sc-compose and ATM crates from
   published normal/build graphs, including transitive identities (ANC-012).
   Run all existing kit preflight checks; run `python3 scripts/benchmark/check_release_evidence.py`. It selects the
   newest applicable `site/reports/benchmark-monty/*.campaign.json` and HTML twin;
   any source difference beyond the checker's allowed report-only changes requires
   a fresh controlled-host campaign under the existing benchmark procedure.
   Planning/source/version changes are not exempted by ancestry alone.
2. **integration:packages** — Inspect each actual package archive: correct
   0.2.1 identity, README, features, and qualified dependency requirements.
   Every retained additional upper bound has a TOML source-manifest comment
   immediately beside it naming the concrete break requiring that specific cap
   (ANC-019). Inspect Cargo.toml.orig where Cargo normalizes the package manifest;
   the Monty cap comments identify Rust 1.98 versus supported Rust 1.96.1.
3. **integration:registry** — Publish all six crates in the kit's configured
   order. Verify each registry version and packaged README. A successful publish
   command or file listing alone is not evidence of consumer compatibility.
4. **req:132/133-advisories** — The exact `tests/registry/phase-d/beads-cache/Cargo.toml`
   consumer and separate server consumer defined above use released0.2.1, build
   and clear
   RUSTSEC-2026-0459 and RUSTSEC-2026-0462 using `cargo deny check advisories`.
   Verify registry identities and the resolved native library; state any other
   advisory findings separately. Unsupported SQLx combinations are not positive proof.
5. **req:133-updates** — With sc-runtime held at 0.2.1, a registry consumer
   selects newer Cargo-compatible third-party dependencies using `cargo update`
   and passes affected tests. Run the owned registry workflow/entrypoint defined
   above at both minimum/latest endpoints (ANC-017) after publication; record the
   positive CI result. Assert libsqlite3-sys>=0.39 in the beads-cache graph,
   rmcp>=3.9 in the server graph, and absence of RUSTSEC-2026-0459 and
   RUSTSEC-2026-0462 in audit output without target-advisory ignores.
   Inherited source CI, archive rehearsal or documented commands are insufficient.
6. **req:131-helper** — A registry consumer enables `sc-runtime/test-util`
   and compiles the documented instrument_router usage with PR131's restored
   public signature. Document presence alone does not pass this criterion.

Keep concise results and exact source/version identities in release notes.
The consumer is the release reviewer; these records gate issue closure and
prevent the observed “published crate still blocks the fix” defect. Supersede
them with the next release's qualification rather than adding permanent ledgers.
Tests use controlled inputs and causal synchronization, never sleep/timing races.

## Closure

Close #132/#133 only after the released-consumer criteria pass. Update the
project plan to the actual release state. Failures remain open with their source
owner; publication, source tests, and README checks cannot substitute for the
positive consumer outcome. No unrelated engine upgrade or new product feature.
