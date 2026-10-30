# Independent anchor question tuning: final handoff

## Outcome

All 32 fixture-inclusive per-anchor planning and development tournaments are accepted under the updated locked ANC authority packets. Each final response was rendered with the installed `anchor-tuning-result.md.j2` through `sc-compose --strict --check-render`, schema validated, and contract validated. The seven-case planning oracles gained one approved pass and one approved fail fixture (9 cases total); nine-case development oracles likewise gained two fixtures (11 cases total). Every selected set is role-complete with exact role cells, exact semantic verdicts, stable first repeats, and zero regressions. The 0.8 confidence reference remains diagnostic; production confidence remains 0.6.

The first installed testability corpus had two fidelity defects: a logs/errors quote was rewritten as stdout/stderr, and JSON was narrowed to JSON objects. The versioned corpus corrected those two statements without changing expected labels; its sealed five-role rubric was 85/85 exact, confident, and stable. All predecessor bytes, failed transport records, oracle versions, candidate pools, requests, responses, and histories remain in this folder.

## Final question sets

| Anchor | Planning selected bundle | Development selected bundle | Authority version |
|---|---:|---:|---:|
| ANC-027 | 2 | 0 | 2 |
| ANC-028 | 2 | 0 | 2 |
| ANC-029 | 2 | 0 | 2 |
| ANC-030 | 2 | 1 | 3 |
| ANC-031 | 2 | 0 | 3 |
| ANC-032 | 1 | 0 | 3 |
| ANC-033 | 3 | 3 | 3 |
| ANC-034 | 2 | 0 | 2 |
| ANC-035 | 1 | 0 | 2 |
| ANC-036 | 2 | 0 | 2 |
| ANC-037 | 1 | 0 | 3 |
| ANC-038 | 4 | 4 | 2 |
| ANC-039 | 2 | 0 | 2 |
| ANC-040 | 1 | 0 | 2 |
| ANC-041 | 1 | 0 | 2 |
| ANC-042 | 1 | 1 | 2 |

The machine-readable mapping of all 32 selected result paths, canonical digests, authority versions, oracle digests, question-set digests, and tuning-history digests is `final-tuning-inventory.json`. Gate-specific per-anchor override maps are `planning-final-question-overrides.json` and `development-final-question-overrides.json`. Each override was merged against the installed gate question set and passed role validation.

## What changed during calibration

- Versioned probe corrections made silence distinct from an explicit conflicting choice, and made a planning sentence distinct from a description of delivered work. These are instrument fixes, not edits to the active plan.
- ANC-029 probes were clarified to test arbitrary command forwarding, not one unusual subcommand.
- ANC-031 preserves the production crate ban and its test/benchmark exception.
- ANC-032 treats `render_template` as optional, never as a removal requirement.
- ANC-033 targeted wording distinguishes the complete absence of both MVP signing and separate approval gates from satisfying only one half.
- ANC-038 targeted wording requires deliberately supplied plaintext values for both secret and nonsecret classes; partial nonsecret-only support remains conflicting.
- ANC-041 tests reading configured environment during source initialization, not script-side mutation.

No product-scope ambiguity or owner decision arose from the frozen authority. No active plan text was used as an oracle or edited by this tuner. Findings about actual plan implementation belong to the independent plan/development review, not these question tournaments.

## Remaining gate work

1. Run `jev_judge.py check-fixtures` against the current locked ANC records at both plan and development gates using the exact gate-specific override maps. Preserve the fixture run files and confirm every fixture verdict.
2. Use the override maps for production reviews. Writing selected questions into anchor records changes their authoritative digest and requires another append-only authority rebind; passing `--overrides-file` keeps these validated authority versions intact.
3. Apply only results whose authority record digest matches the current record. The inventory provides each digest for this check.

## Evidence

The v4 authority rebind reused 694 planning and 864 development raw response files only after canonical request-body equality and byte-for-byte file-copy verification; the binding files document each source and destination. Fresh calls covered every unmatched fixture or revised probe body. The two v5 controls reused 120 further byte-identical raw response files and made 12 fresh first-pair calls plus three required third confirmations. No oracle case was altered after its round began.
