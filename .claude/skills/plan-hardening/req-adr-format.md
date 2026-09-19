# REQ / NFR / ADR Format

One format for every repository, so `req-qa`, `arch-qa`, sprint plans and tools
read every repo the same way. Check a repo with:

```sh
python3 .claude/skills/plan-hardening/scripts/validate_req_adr.py
```

## Where entries live

| File | Holds |
|---|---|
| `docs/requirements.md` | repo-level `REQ-*` and `NFR-*`: behaviour that spans crates or lives outside them |
| `docs/architecture.md` | repo-level architecture and `ADR-nnn` under an `## ADR` heading |
| `docs/<crate>/requirements.md` | `REQ-<CRATE>-*` and `NFR-<CRATE>-*` for that crate only |
| `docs/<crate>/architecture.md` | that crate's architecture and `ADR-<CRATE>-nnn` under an `## ADR` heading |

Every crate under `crates/` has both crate files. An entry is owned by exactly
one file and is never restated in another; other files refer to it by id.

## Ids

| Kind | Form | Example |
|---|---|---|
| Functional requirement | `REQ-<AREA>-nnn` | `REQ-TRANSPORT-006` |
| Non-functional requirement | `NFR-<AREA>-nnn` | `NFR-REPO-001` |
| Repo-level ADR | `ADR-nnn` | `ADR-004` |
| Crate-level ADR | `ADR-<CRATE>-nnn` | `ADR-CONFIG-002` |

- `<AREA>` and `<CRATE>` are upper case `A-Z0-9`; a crate's area is its name
  without the common prefix (`sc-config` gives `CONFIG`). `nnn` is three
  digits. A suffix letter (`REQ-X-002A`) refines an existing entry.
- An id is never reused or renumbered. A withdrawn entry stays, marked
  `**Status:** withdrawn` with the reason and any replacement id.

## Requirement entry

```markdown
- `REQ-<AREA>-nnn` **Short title.**
  The obligation, stated so that a reviewer can say yes or no. Name the
  concrete things: functions, files, error codes, commands.
  **Why:** the problem this solves or the decision it comes from, so a reader
  can judge whether a change still honours it.
  **Verified by:** the evidence that proves it: a named test, a command and its
  expected result, or an inspection. "Tests pass" is not evidence for an NFR.
```

All three parts are required. Length follows need: two to six sentences for the
obligation is normal. One line is too short if a reader who has not seen the
design cannot tell what the entry relates to; a page is too long if it restates
the architecture.

A requirement says **what** must be true. **How** it is achieved is an ADR.

## ADR entry

```markdown
### ADR-nnn Title stating the decision

| Field | Value |
|---|---|
| Status | proposed \| accepted \| superseded by ADR-nnn |
| Date | YYYY-MM-DD |
| Source | where the decision came from |
| Relates to | REQ / NFR / ADR ids |

**Context.** The forces and the problem; what made a decision necessary.

**Decision.** What was decided, in the present tense, concrete enough to check
a change against.

**Consequences.** What follows, including the costs and what stays open.

**Rejected.** The alternatives considered and not taken.

**Enforced by.** The mechanism: a lint, a boundary manifest, a test, or a named
reviewer.
```

- Only `accepted` ADRs are binding. A `proposed` ADR names the sprint that must
  accept or amend it, and no other sprint may depend on it.
- An ADR changes only through a later ADR that names the one it amends or
  supersedes. The old entry stays, with its status updated.

## Use in sprint plans

Every sprint doc carries a `requirements` list (every REQ and NFR id it
implements or is constrained by) and an `adrs` list (every ADR id that governs
it). `req-qa` and `arch-qa` reject a sprint doc whose lists are missing or
incomplete, and evaluate every code change and fix against the entries.
