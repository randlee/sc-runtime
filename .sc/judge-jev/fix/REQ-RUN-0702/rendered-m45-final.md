# Requirement: Coordinated public release and registry consumers

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12  
**Owner:** randlee  
**ID Range:** REQ-RUN-0702 through REQ-RUN-0702  
---

## REQ-RUN-0702: Coordinated public release and registry consumers

**Status:** Active  
### Requirement Statement

Definitions: The software development kit (SDK) in this record is the nine public crates identified by REQ-RUN-0002; `sc-storage` is its shared storage crate and `sc-monty` its Monty interpreter crate. The fixture matrix is the template CI workflow that generates one project per `wizard/fixtures/*.json` file and runs `just setup`, `just lint`, `just test`, `just build-cli`, and `just build-daemon` (REQ-RUN-0701). Registry-only resolution means the generated project's SDK crates come from crates.io without checkout paths, Git dependencies, or patch substitutes. A registry consumer proof is the build and test of a project generated from published packages with that resolution. The qualified publishing kit is the sc-publish kit pinned by `release/sc-publish-pin.toml`, together with its existing preflight and package checks. Archive rehearsal is the verified multi-package Cargo dry run and inspection of normalized `.crate` manifests and contents. A controlled-host benchmark is the workspace campaign run on `atmbench@rand-m5` with committed JSON, HTML, and report-index evidence reviewed by the quality manager. An isolated daemon lifecycle test starts a daemon binary on an isolated virtual machine or ephemeral CI runner (NFR-RUN-0010). Human release authorization and credentials are the release authorization and registry credentials described in `docs/releases/v0.1.0.md`. The release tag is the Git tag naming the source commit used for publication and registry consumer proof. The newest-tested `Cargo.lock` records dependency versions actually tested; `.sc-runtime/tested-dependencies.json` records those tested dependencies. A generated populated repository is a project generated from the delivered template with its selected profile rendered.

This requirement governs the coordinated 0.3.0 release of the nine public crates identified by REQ-RUN-0002 and delivery of the generated template.

**MUST statements:**

- The release MUST publish all public crates at the coordinated version.
- Publication MUST follow the dependency order recorded in the final manifests.
- The first publication wave MUST include the independent crates `sc-storage` and `sc-monty`.
- The released template MUST resolve SDK crates from crates.io without checkout path, Git or patch substitutes.
- The template fixture matrix MUST pass against the released registry packages.
- The release tag MUST identify the same source used for publication and registry consumer proof.
- The release MUST use the qualified publishing kit.
- The release MUST pass the kit's existing preflight.
- The release MUST pass the kit's existing package checks.
- Exact packaged archives MUST pass rehearsal before upload.
- Actual post-publication consumers MUST pass with registry-only SDK resolution.
- The release MUST preserve the existing human-authorization and credential requirements for publication.
- Existing controlled-host benchmark and isolated daemon lifecycle requirements MUST remain release prerequisites.
- Each delivered template release MUST include its newest-tested `Cargo.lock`.
- Each delivered template release MUST include matching `.sc-runtime/tested-dependencies.json`.
- Generated populated repositories MUST receive the release lock.
- Generated populated repositories MUST receive tested-dependency metadata.
**MUST NOT statements:**

- Checkout-patched consumer tests MUST NOT be reported as registry proof.
- Publication MUST NOT change source or manifests between the qualified archives and the registry consumer proof.

### Rationale

Published registry packages permit consuming projects to upgrade the SDK through Cargo. A tested lock and dependency metadata recommend a known combination without restricting consumer dependency choices.


### Success Criteria

The coordinated packages are visible in crates.io, the exact-archive rehearsal and registry-only consumer runs pass, the fixture matrix is green, and the release tag identifies the qualified source. Generated profiles receive the matching tested lock and metadata from their release build/test run.


### Implementation Notes

ADR-RUN-0017 governs the retained observability shim. Under ANC-037, Phase D published/template observability requirements are >=1.5.0 with no upper bound. Normal builds and tests validate compatibility with selected versions and replace registry-availability refresh, the isolated signature/direct-Arc probes, and the forced shim-mode-fail cleanup gate. No cleanup work follows merely from an upstream release becoming available.


### Related Documents

**Architecture Decisions:**

- ADR-RUN-0017 — Initial release amendment — 2026-10-31
