# Phase B directory correction

Approved predecessor: e5eda22e81f724e3024fdaa1fd2b5ea4c511406c.

The literal `doc/plan/phase-current` placeholder was incorrect. Phase B records
now live alongside the sprint plans in `docs/plans/phase-b`. The old physical
directory is removed. `plan_dir` is committed with that actual phase path.

The three repository harness installs use plan-anchor 0.8.6 from the exact
phase-folder fix in https://github.com/randlee/plan-anchor/pull/59 at
26112a8fd1d5e414e59cf62d8528b939ad132894. Existing `bd` selection and remote
judge consent are preserved; global plugin caches are not modified.

Migration integrity checks:

- 143 tracked files and 8,127 physical files moved without overwriting a
  destination; 8,124 files retain identical bytes.
- Only the live intent heading, anchor-document heading/path, and top-level
  anchor plan/path labels change. All 70 anchor records, including statements,
  quotes, scopes, versions, questions, fixtures, and exclusions, are identical.
- Approved sprint plans, canonical requirements/ADRs, and historical raw
  responses, archives, and checksums are unchanged. Historical payloads retain
  their original source locations; map the old prefix to the new directory
  when resolving those references rather than rewriting signed/hash-bound
  historical evidence.
- The existing CI-scope fixture now names the actual anchor document path.
  Its three tests pass. The moved anchor record validates successfully.

`migration-summary.json` records the exact checks and approved source hashes.
`migration-inventory.json.gz` preserves the complete pre-move file inventory;
`approved-anchors-before.json.gz` preserves the original live record.

The next steps are the requested quality-mgr content-integrity review and
fresh plan-anchoring gates on the pushed migration commit. Their verdicts will
be recorded here; the earlier approvals remain historical source approvals.
