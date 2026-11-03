# Plan fixes before the second gate iteration

All authorities below were locked before the initial gate. Question wording,
thresholds, and anchor scopes remain unchanged. These are preservation and
acceptance clarifications, not new product features.

| Sprint / anchor | Existing sentence or location | Action and replacement intent | Pre-existing authority |
| --- | --- | --- | --- |
| b-13 / ADR-RUN-0002 | `This sprint updates manifests and lockfile only; it does not publish any crate.` | Replace with the same release limit and explicit template-plus-published-library architecture. | ADR-RUN-0002 |
| b-13 / ANC-027, ANC-033, ADR-RUN-0013, REQ-CRON-0003 | `it does not reopen or restate the host allowlist, program-selection, or recovery rules.` | Replace stale fixed-host wording with preservation of general-purpose Rust/Python/Monty jobs and no MVP signing/separate approval; b-16 owns the extension seam and adapters. | ANC-027, ANC-033, ADR-RUN-0013, REQ-CRON-0003 |
| b-13 / REQ-CRON-0005 | Public signatures section | Add one explicit rule that every fallible public sc-cron function returns CronError; signatures already follow it. | REQ-CRON-0005 |
| b-14 / ANC-031 | Acceptance boundary-lint sentence | Add the exact production compose/ATM crate exclusion and test/benchmark exception. | ANC-031 |
| b-17 / ANC-027, ADR-RUN-0002 | Goal and relations | Add the existing general-purpose library/template architecture, with no restriction to built-in jobs. | ANC-027, ADR-RUN-0002 |
| b-17 / ANC-028 | `all registered host calls are denied during initialization.` | Replace to distinguish invocation timing from unchanged application registration and later dispatch. | ANC-028 |
| b-17 / ANC-029 | `They retain b-16's unrestricted argv forwarding` | Replace with explicit any-command/subcommand forwarding for both adapters. | ANC-029, ANC-030 |
| b-17 / ANC-032 | Checked example description | Add that render_template is not required product behavior. | ANC-032 |
| b-17 / ANC-033 | `The application supplies source without an MVP signing or approval gate.` | Replace with no signature verification, separate approval check, or consumer option enabling either. | ANC-033 |
| b-16, b-17 / REQ-RUN-0503 | Generated-project acceptance | Require the already defined fresh, unedited project's setup/lint/test/build-cli/build-daemon sequence; retain feature checks. | REQ-RUN-0503 |
| b-15 / ANC-027, ANC-033, ADR-RUN-0013, REQ-CRON-0001, REQ-CRON-0003 | Goal preservation sentence | Add explicit optional general-purpose Rust/Python/Monty job semantics, no MVP signing/approval option, and no runtime-to-cron dependency; host changes remain b-16-owned. | Named locked anchors |
| b-18 / ADR-RUN-0002, ANC-031 | Goal and shared Cargo inventory sentence | State template/published-library architecture and preserve the production tool-crate boundary with test exceptions. | ADR-RUN-0002, ANC-031 |
| b-18 / ANC-028, ANC-030 | Completed registry forwarding paragraph | Replace with explicit preservation of custom handlers without editing sc-cron and both adapters' any-command/subcommand forwarding. | ANC-028, ANC-029, ANC-030 |
| b-18 / ANC-036, ANC-038, ANC-040, ANC-041 | Hook policy paragraph and integration proof | Preserve explicit Set/Clear, intentional plaintext SQLite values, absolute dotenv loading without captured persistence, and initialization-time reads in the existing integration case. | Named locked anchors |

b-13 does not change the template or generator: its crate versions are
consumed by later template sprints. REQ-RUN-0503 is therefore an explicit
item exclusion there, rather than duplicating the later generated-project
acceptance work. Likewise, logger propagation does not implement the b-16
host extension or b-17 environment policy. Item exclusions record these
ownership boundaries instead of copying those feature requirements into b-13.

## Size triage after plan clarification

b-13 and b-17 were refused before any live request: respectively 24,082 and
24,514 bytes including the largest anchor/question envelope (limit 24,000).
All work remains linked to pre-existing authority; no unapproved deliverable
was found. The cause is repeated prose: b-13 repeats its host-successor
ownership in Goal and deliverable 1; b-17's deliverable list restates the
preceding complete model, resolution, runtime, and schema sections. Remove
that duplication, retaining every public type, limit, precedence rule, error
code, platform case, source path, and acceptance command. This editorial cut
also makes the plans easier to maintain. No request cap or sprint count changes.

## Independent exclusion audit

The adversarial reviewer rejected two proposed exclusions on actual ownership:
b-16 edits subprocess creation, so ANC-035 still governs the child environment;
it edits a template crate manifest, so REQ-RUN-0329 still governs package names.
Before rerunning, add explicit empty child environments pending b-17's filtered
snapshot, and preserve `<project>-<role>` generated names. These additions are
required by those pre-existing authorities, not newly requested features.
The reviewer separately accepted excluding ANC-033 on b-13/b-15 because neither
logger propagation nor event emission changes source acceptance, signatures,
approval decisions, or Monty authorization policy. Its obligation remains
judged on b-16/b-17/b-18. b-17/ANC-032 remains judged because that sprint edits
the checked Monty example.

b-17's remaining ANC-032 cell is uncertain about relevance, not compliance.
Replace the passive goal sentence about the checked example with the active
instruction and exact owned path `template/examples/monty/cron.py`: update it
to demonstrate the environment policy through CLI adapters while keeping
render_template an optional internal helper, not a required product API.
This clarifies existing deliverable 4 and ANC-032; it adds no new example.

b-17's Relations paragraph repeats must_follow, Cargo/example ownership, and
the no-new-sprint statement; consolidate it without changing dependencies.
b-18's deliverable 3 repeats the preceding cron::open code and policy wiring;
retain its unique limits mapping and refer to that fixed contract. Its
acceptance 6 repeats deliverables 6–8; retain the executable test names and
assertions while removing the second narrative of the same publication and
generated-project sequence. These are duplicate-prose cuts, not work removal.

## b-18 integration clarification

The first reviewed b-18 result left ANC-028/029/030/040 underspecified.
For ANC-028, add an explicit assertion in the existing integration test that
registers and invokes an application closure through the real cron::open
argument: the wrapper must preserve handlers, not merely accept a registry
parameter. For ANC-040, replace the general authorized-dotenv phrase with the
absolute-path load and named daemon file/root configuration already defined
by b-17. Both additions trace to the named locked anchors, predating the run.
Independently audit whether adapter forwarding anchors 029/030 actually govern
this wrapper sprint before adding duplicate b-16 adapter tests.

Independent review confirms ANC-029/030 do not govern b-18's change: it forwards
an opaque prebuilt registry and changes neither CLI adapter, argv dispatch,
nor adapter examples. Record those two item exclusions with that reason;
b-16's successful gate retains both obligations. The plan-only rerun cleared
ANC-028 and ANC-040 and left only those two relevance cells failing.

Final source review corrected b-16's historical claim of existing b-10
child-environment protection: preserve the landed deadline/output cap and
apply the newly explicit environment restriction above, rather than claiming
the baseline already supplied it. This is a wording correction under ANC-035;
the planned behavior is unchanged. Rerun b-16 on that exact final text.
