# Phase B diagnostic improvement comparison

These are diagnostic Jev comparisons using the installed default planning
questions and unchanged approved ANC-027 through ANC-042. They are not sealed
anchor-tuning gates and do not replace the independent rubric/question seal.
The baseline is commit `7954373`. Source snapshots, SHA-256 digests, and
authority-derived expectations were frozen before their corresponding calls.
Each request contains one complete sprint; anchors are scoped to that sprint.
Threshold remains 0.6. No question wording, anchor meaning, cap, or threshold
was changed to obtain a passing result.

| Sprint | Earlier result | Final diagnostic result | Change justified by approved authority |
|---|---|---|---|
| b-13, ANC-031 | underspecified, min confidence 0.46; first revision 0.49 | satisfied, 0.89 | Explicitly state the existing production tool-crate exclusion while b-16 owns enforcement. |
| b-16, ANC-027..033 | 1 contradicted, 6 underspecified, 0 satisfied | 7 satisfied; final provenance-only rerun minimum 0.70 | Replace fixed/render-only hosts with the application registry, both any-command CLI adapters, production crate boundary, and explicit no-signing MVP. |
| b-17, ANC-034..042 | PLAN.TOO_LARGE before a Jev response; 32,373 normalized chars even after permitted code reduction | 9 satisfied; minimum 0.91 | Consolidate repeated public-model, resolution, delivery, and acceptance prose. Keep the public API, limits, precedence, authorization, redaction, and tests; remove unreleased migration obligations. |
| b-18, ANC-028/034/035/037 | first composition check: 2 satisfied, 2 underspecified | 4 satisfied | Replace obsolete with_monty_host composition with the supplied registry. Replace vague policy-preservation prose with explicit Python/Monty parity and daemon-name-only inheritance. |

The b-16 baseline's definite contradiction is ANC-029: a render-only filter
conflicts with any-command sc_compose_run. Several other baseline cells were
uncertain rather than definite violations; retain their exact raw verdicts.
The frozen expectation table is independently derived and was not relabelled
after those answers. Final outcomes are measured results, not a claim that the
earlier expected baseline labels were exact predictions.

b-17 source shrank from 33,430 to approximately 21,270 UTF-8 bytes by removing
duplicate prose, not by dropping approved behavior or splitting a sprint. Its
earlier refused snapshots and error envelopes remain available. The new full
sprint fits the default request budget without changing the cap.

## Evidence roots

- `frozen-sources-and-expectations.json`: baseline and first revised texts.
- `final-source-expectations.json`: first final b-13/b-16 texts and expectations.
- `final-v2-source-expectations.json`: final b-16 after adding its missing
  REQ-CRON-0005 frontmatter/deliverable citation; no contract change.
- `after-compact-v2-source-expectations.json`: final b-17 text and expectations.
- `after-composition-v2-source-expectations.json`: final b-18 text and expectations.
- `b-13-after-final-tls2-run.json`, `b-16-after-final-v2-tls2-run.json`,
  `b-17-after-compact-v2-tls2-run.json`, and
  `b-18-after-composition-v2-tls2-run.json`: final complete responses and roles.
- `final-plan-sources.json`: final per-file SHA-256 inventory for the six
  continuation sprints, phase index, and three source outlines. Combined
  canonical source-manifest SHA-256:
  `02f1670df421e9d89e3fa5a3fbf16cb7d38db8cf33d3be36dc60770c32578d45`.
  Frontmatter parsing, declared authority IDs, exact owned-path uniqueness,
  same-wave path disjointness, relative links, and plan diff whitespace checks
  all passed at this source pin.

Earlier transport failures are preserved in the initial run/envelope files.
They were retried with the trusted certifi CA bundle; TLS validation remained
enabled. No credentials are stored in the comparison artifacts.

Allowed growth: b-13's added sentence implements pre-existing ANC-031;
b-16's added registration/adapters/boundary acceptance implements pre-existing
ANC-027 through ANC-033. b-18's replacement preserves ANC-028/034/035/037 and
is shorter than the ambiguous paragraph it replaces. The tuner must assess
any remaining question uncertainty independently rather than weaken authority.
