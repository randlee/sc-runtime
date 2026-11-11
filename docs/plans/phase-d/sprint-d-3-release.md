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
must_follow_rationale: Consumes d-1's qualified API/dependency manifests and reproducible lockfiles, and d-2's README bodies that must be packaged in those exact crates. Publication cannot precede the candidate code and docs; this is artifact coupling, not a shared version-baseline preference.
owned_paths:
  - tests/registry/phase-d/**
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
PRs merge before this sprint completes. One final candidate combines the two
artifacts. This release is coupled to those artifacts, not unrelated phase work.

## Deliverables

| Deliverable | Requirement served |
| --- | --- |
| Six 0.2.1 packages produced and published through the configured sc-publish kit, preserving their qualified ranges/features and README content | REQ-RUN-0002/0702, ADR-RUN-0002 |
| Registry-only consumer proof for the affected beads-cache configuration: security clearance and compatible updates with unchanged sc-runtime release | REQ-RUN-0002/0005, NFR-RUN-0007; #132/#133 |
| Registry compilation of documented `sc-runtime/test-util` instrument_router usage; registry README verification | REQ-RUN-0702, NFR-RT-0001, ADR-RT-0004; #131 |
| Concise v0.2.1 release notes with supported/unsupported native-link configurations and actual validation outcomes | REQ-RUN-0702 |

The registry proof is a separate consumer workspace under this sprint's owned
test path. Exact sc-* 0.2.1 selection identifies the release being tested; it
does not narrow the libraries' third-party requirements. Avoid local path/patch
substitutes. Use existing source and publishing procedures, not a new release framework.

## Acceptance and validation

1. **integration:candidate** — The final integrated candidate passes
   `just lint && just test` and required minimum/latest dependency CI. Existing
   boundary/production-tool guards exclude sc-compose and ATM crates from
   published normal/build graphs, including transitive identities (ANC-012).
   Run all existing kit preflight checks; refresh controlled-host benchmark
   evidence under its existing procedure if this candidate invalidates it.
2. **integration:packages** — Inspect each actual package archive: correct
   0.2.1 identity, README, features, and qualified dependency requirements.
   Every retained additional upper bound has a TOML source-manifest comment
   immediately beside it naming the concrete break requiring that specific cap
   (ANC-019). Inspect Cargo.toml.orig where Cargo normalizes the package manifest;
   the Monty cap comments identify Rust 1.98 versus supported Rust 1.96.1.
3. **integration:registry** — Publish all six crates in the kit's configured
   order. Verify each registry version and packaged README. A successful publish
   command or file listing alone is not evidence of consumer compatibility.
4. **req:132/133-advisories** — A registry-only consumer using released 0.2.1
   and the affected beads-cache dependency configuration builds and clears
   RUSTSEC-2026-0459 and RUSTSEC-2026-0462 using `cargo deny check advisories`.
   Verify registry identities and the resolved native library; state any other
   advisory findings separately. Unsupported SQLx combinations are not positive proof.
5. **req:133-updates** — With sc-runtime held at 0.2.1, a registry consumer
   selects newer Cargo-compatible third-party dependencies using `cargo update`
   and passes the affected tests. Required registry-proof CI executes this
   consumer twice: minimum allowed shipped dependency set, then latest allowed
   set, running the affected consumer tests at both endpoints (ANC-017). Invoke
   `.github/workflows/crates.yml` with `proof=registry`, wired by d-1; this sprint owns the consumer test
   workspace, not another workflow edit. Record the post-publication CI result;
   inherited source CI or merely documenting the commands is insufficient.
   Never present a source-path consumer as released proof.
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
