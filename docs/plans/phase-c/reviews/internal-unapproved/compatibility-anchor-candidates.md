# INTERNAL: unapproved compatibility and cron anchor candidates

Preparation only. These five candidates are not in `anchors.json`, are not locked, have no tracker records, and have not been rubric-tested or approved. Highest allocated existing ID is ANC-012; current locked set is ANC-001/003/004/006/007/008/009/010/011/012. IDs ANC-013…017 below are provisional until the main session captures them.

The main session owns capture and adoption. Before presenting these candidates for approval, run the actual live testability rubric after the independent generic instrument has been accepted. These project-specific facts and candidate fixtures must not be supplied to the independent generic calibrator or its oracles.

## Candidate ANC-013 — compatibility before publication

Verbatim human response:

> Add a dedicated Phase C compatibility sprint (Recommended)

Proposed statement: “Phase C must complete its dedicated compatibility sprint before publishing sc-runtime 0.2.0.”

Kind: directive.

Proposed scope: “Phase C compatibility sprint planning and the sc-runtime 0.2.0 publication handoff.”

Provenance context: The parent reports this was the human's selection in an assistant-authored scope question that listed four compatibility capabilities before 0.2 publication. The quoted option is the human response; the capability list and publication condition came from the assistant question approved by that selection. The full question is not reproduced here because its exact wording was not supplied. Preserve the actual question/response record in the main session rather than attributing a reconstructed capability list to the human. This candidate states only the approved sequencing; concrete capabilities remain in their governing sprint and REQ/ADR records.

Pass fixture: “The dedicated Phase C compatibility sprint completes before the release handoff publishes sc-runtime 0.2.0.”

Fail fixture: “The release handoff publishes sc-runtime 0.2.0 before the dedicated Phase C compatibility sprint is complete.”

Deterministic check to pair later: the compatibility sprint's completion evidence precedes the 0.2.0 publication step in the release dependency chain and release record.

## Candidate ANC-014 — opt-in cron command set

Verbatim human quote:

> it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface.

Proposed statement: “A consumer application must be able to omit the standard cron command set.”

Kind: directive.

Proposed scope: “The standard cron CLI command-set contract and consumer adoption or composition of that command set.”

Pass fixture: “The consumer can choose whether to expose the standard cron command set and can omit it entirely.”

Fail fixture: “Every consumer is required to expose the standard cron command set.”

Deterministic check to pair later: an application fixture composes its CLI with the optional set and another composes without it. The choice of feature flag, constructor, registration API, or other mechanism is not fixed by this quote.

## Candidate ANC-015 — common JSON interface

Verbatim human quote:

> it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface.

Proposed statement: “Supported cron user interfaces must share the standard JSON command contract.”

Kind: directive.

Proposed scope: “The standard cron command contract and supported user interfaces consuming it.”

Pass fixture: “Supported cron user interfaces use the same standard JSON command contract to configure schedules.”

Fail fixture: “Each supported cron user interface defines a different, incompatible JSON command contract for configuring schedules.”

Provenance context: The human says “similar json interface”; “share the standard JSON command contract” formalizes that consistency aim. It does not freeze exact public Rust types, request fields, wire envelopes, or transport protocols. The reviewer must check that this formalization preserves the intended degree of consistency before capture; it is a candidate, not an already approved schema equality rule.

Deterministic check to pair later: conformance examples for the supported interfaces consume the same declared JSON command contract. Which interfaces are supported and the contract's fields remain explicit planning decisions, not new facts asserted by this candidate.

## Candidate ANC-016 — common verbs across handler kinds

Verbatim human quote:

> yes, we have a few more options i.e. python, monty, rust handler. but the general subcommands should be standard

Proposed statement: “The standard cron subcommands must be the same for Python, Monty, and Rust handlers.”

Kind: directive.

Proposed scope: “The standard cron subcommand vocabulary and handler-kind selection.”

Pass fixture: “Python, Monty, and Rust cron handlers use the same general subcommands, with handler-specific choices represented by their options.”

Fail fixture: “Python, Monty, and Rust cron handlers each require a different set of general cron subcommands.”

Deterministic check to pair later: command-model/conformance tests show the common declared verbs accept each supported handler kind. The candidate neither chooses the verb spellings nor freezes option grammar.

## Candidate ANC-017 — application registration of three job targets

Verbatim human quote:

> yes, we will want to register monty, python and rust jobs w/ cron.

Proposed statement: “Applications must be able to register Monty, Python, and Rust jobs with sc-cron.”

Kind: directive.

Proposed scope: “Application registration of cron jobs and the compatibility proof for supported execution targets.”

Pass fixture: “An application registers jobs targeting Monty, Python, and Rust through the supported sc-cron composition.”

Fail fixture: “Applications can register only Rust jobs; Monty and Python jobs cannot be registered with cron.”

Deterministic check to pair later: application registration fixtures exercise all three existing execution targets through the reviewed contract.

Existing authority overlap: `REQ-CRON-0003: Rust, Python, or Monty hook` already calls sc-cron a general-purpose scheduler for application-defined jobs and permits an in-process `Job`, Python module, or optional Monty source. This is preservation and consumer registration of existing targets, not authorization to create another job engine. Before adding ANC-017, decide whether to preserve this user quote as provenance for the existing REQ-CRON-0003 anchor instead of duplicating the same invariant under a new ANC. Do not allocate both as separately counted obligations if compliance is identical.

## Additional original human provenance, not a new candidate

> there should be a standard/recommended cli surface for cron in sc-runtime. we have already implemented cron interface in atm-cron crate (in atm-core) and I think I specified in one other place as well.

This identifies existing `atm-cron` in `atm-core` as a reference to inspect while defining the standard surface. It does not itself require copying exact public types or declaring another team's phase, and this preparation does not turn it into a concrete migration implementation mandate.

## Duplicate and authority review

- None of the ten active Phase C anchors governs compatibility-sprint sequencing, cron command-set optionality, common cron JSON semantics, common subcommands across handler kinds, or three-target registration; the new candidates do not repeat their installation/version/boundary/branch constraints.
- ANC-014 and ANC-015 share one original sentence but impose independent constraints: an optional command set can have inconsistent JSON, and a consistent JSON contract can be mandatory. Keep those duties separate.
- ANC-015 and ANC-016 are distinct: one fixes cross-interface JSON consistency, the other fixes general command vocabulary across handler kinds. Neither implies the other.
- ANC-017 substantially overlaps the existing REQ-CRON-0003 target invariant; retain the existing requirement and make the adoption proof explicit. New user provenance is not, by itself, a reason to double-count existing register authority.
- ANC-013 sequences the dedicated compatibility work but does not invent its four-capability checklist. The exact approved assistant question must remain attributable as assistant-authored scope accepted by the human.
- No candidate freezes API names/types, timing budgets, ATM team/phase identity, cron field count/grammar, or a new scheduling engine. Those details belong to existing or deliberately amended REQ/ADR contracts and their implementation proofs.

No live rubric, capture, lock, approval request, tracker write, question tuning, or repository edit was performed for this internal preparation.
