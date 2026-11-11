---
id: d-2
phase: d
status: planned
closure_type: docs
target_boundary: runtime-ecosystem-consumer-docs
track: consumer-docs
wave: 1
branch: sprint/d-2-consumer-docs
worktree: sprint/d-2-consumer-docs
pr_target: integrate/phase-d
recommended_agent: cipher
recommended_model: luna
requirements:
  - REQ-RUN-0702
  - NFR-RT-0001
adrs:
  - ADR-RUN-0002
  - ADR-RT-0004
github_issues: [131]
parallel_safe:
  - d-1
parallel_safe_rationale: README-only ownership is disjoint from every d-1 manifest, code, CI, and policy path. No manifest edit is performed here.
must_follow: []
owned_paths:
  - README.md
  - crates/sc-config/README.md
  - crates/sc-transport/README.md
  - crates/sc-command/README.md
  - crates/sc-runtime/README.md
  - crates/sc-cron/README.md
  - crates/sc-cron-commands/README.md
  - crates/sc-runtime-db-sqlite/README.md
paths_to_delete: []
---

# Sprint d-2 — consumer documentation

## Goal

Give consumers accurate adoption, feature, compatibility, and template guidance
for the 0.2.x framework. The [registry audit](registry-doc-audit.md) records the
observed README defects. This sprint closes documentation only; it performs no
version, dependency, build, or publication mutation.

## Deliverables

| Deliverable | Requirement served |
| --- | --- |
| Root README lists six published crates, 0.2.x status, generation entrypoints and supported Rust versions | REQ-RUN-0702 |
| Each published crate README links its role to the framework/template; remove stale 0.1.0 publication claims | REQ-RUN-0702 |
| sc-runtime README documents the restored public `testing::instrument_router` signature behind `test-util`, using PR131's actual API | REQ-RUN-0702, NFR-RT-0001, ADR-RT-0004 |
| SQLite writer README states publish=false/workspace-only status and the SQLx/rusqlite limitation; distinguish supported direct-crate consumers from unsupported combined graphs | REQ-RUN-0702 |

Suggested common framing is “Part of sc-runtime”, with a link to the repository
and a short statement that the crates and template scaffold JSON-first CLI/daemon
applications. Preserve each crate's useful existing guidance. Do not describe
unqualified generated feature combinations as supported or turn documentation
examples into restrictive third-party dependency pins.

## Acceptance and validation

1. **doc:framework** — Root and six published READMEs consistently describe
   the shipped crate set and framework entrypoints; no stale publication banners.
2. **doc:helper** — The documented test-util feature and instrument_router
   usage match the merged public API. Compilation against the released SDK is
   verified by d-3, not claimed by this documentation review.
3. **doc:compatibility** — Consumer examples preserve compatible dependency
   upgrades (ANC-016). The workspace-only writer and known native-link limitation
   are explicit; docs do not imply a fixed advisory applies to unsupported SQLx graphs.
4. **doc:validation** — Run `git diff --check` and the bounded relative-link
   command below. Review version/feature/API examples against source declarations;
   illustrative text is not live registry evidence. No new docs-lint tool or
   permanent docs-consistency CI gate is introduced.

```bash
git diff --check
python3 - <<'PY'
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit
readmes = [Path("README.md"), *Path("crates").glob("*/README.md")]
for path in readmes:
    assert path.is_file(), path
    for target in re.findall(r"\]\(([^\s)]+)", path.read_text()):
        link = urlsplit(target)
        if not link.scheme and link.path:
            assert (path.parent / unquote(link.path)).exists(), (path, target)
print("README whitespace and relative file-link checks complete")
PY
```

## This sprint does not close

d-1 owns all Cargo.toml changes, including `readme = "README.md"` and release
version requirements. d-3 owns packaging, registry rendering, published examples,
advisory checks, and release closure. This sprint can finish independently of d-1.
