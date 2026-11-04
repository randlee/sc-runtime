# Six proposed anchor statements and fixtures

For human statement/fixture approval. Actual rubric results are pending; nothing
here is locked, recorded or gate-approved. Approval of these drafts can occur
before calibration finishes, but capture/locking still waits for required checks.

IDs below are provisional packet references after checking existing candidates.
Locked records end at ANC-012. ANC-017 is omitted as a duplicate of existing
REQ-CRON-0003. The eventual `add` command allocates actual IDs; it does not reserve
these labels or their gap. Record the packet-reference→actual-ID mapping at
capture rather than silently treating provisional ANC-018/019 as allocated.

## ANC-013 — compatibility before publication

Human answer: "Add a dedicated Phase C compatibility sprint (Recommended)"

Statement: **Phase C must complete its dedicated compatibility sprint before publishing sc-runtime 0.2.0.**

Pass: The dedicated Phase C compatibility sprint completes before the release
handoff publishes sc-runtime 0.2.0.

Fail: The release handoff publishes sc-runtime 0.2.0 before the dedicated Phase C
compatibility sprint is complete.

The exact assistant-authored scope question accepted by this answer is retained
in the matching source-intent descriptor; its words are not attributed to the human.

## ANC-014 — optional standard cron commands

Quote: "it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface."

Statement: **A consumer application must be able to omit the standard cron command set.**

Pass: The consumer can choose whether to expose the standard cron command set
and can omit it entirely.

Fail: Every consumer is required to expose the standard cron command set.

## ANC-015 — shared JSON contract

Quote: "it would be optional set of standard cron commands to set up cron schedule so all ui have similar json interface."

Statement: **Supported cron user interfaces must share the standard JSON command contract.**

Pass: Supported cron user interfaces use the same standard JSON command contract
to configure schedules.

Fail: Each supported cron user interface defines a different, incompatible JSON
command contract for configuring schedules.

"Share the standard contract" proposes a formalization of the user's "similar"
interface; it does not approve a particular DTO, field set or transport wrapper.

## ANC-016 — common verbs across handler kinds

Quote: "yes, we have a few more options i.e. python, monty, rust handler. but the general subcommands should be standard"

Statement: **The standard cron subcommands must be the same for Python, Monty, and Rust handlers.**

Pass: Python, Monty, and Rust cron handlers use the same general subcommands,
with handler-specific choices represented by their options.

Fail: Python, Monty, and Rust cron handlers each require a different set of
general cron subcommands.

## ANC-018 — supported consumer overrides

Quote: "it is not sc-runtime job to guarantee a customer can not misuse the core libraries."

Statement: **sc-runtime must expose supported extension points that let consumers replace default core-library behavior with application-specific behavior.**

Pass: The cron integration accepts an application-provided store with an extended
row schema and a consumer shutdown policy through supported extension points
instead of requiring its bundled defaults.

Fail: Every consumer must use the bundled cron store and default shutdown policy;
custom row schemas and application-specific lifecycle behavior have no supported
integration seam.

The full preceding human hook/override context is retained in the source-intent
descriptor. This does not remove ordinary validation, security or integrity checks.

## ANC-019 — documented correct integration

Quote: "it is sc-runtime job to provide an easy documented path that consumers will not accidently misuse the core libraries."

Statement: **sc-runtime must document a supported integration path that explains correct use of the core libraries and the responsibilities of consumer overrides.**

Pass: The integration guide shows the default cron setup and a supported custom
store or shutdown policy example, including the consumer's validation, lifecycle
and ownership responsibilities.

Fail: Consumers must inspect private scheduler internals to discover override
wiring; the documentation explains neither supported hooks nor consumer
responsibilities.

This is observable guidance, not an absolute guarantee that consumers cannot err.

## Preserved existing requirement and default-path context

Quote preserved without creating duplicate ANC-017:
"yes, we will want to register monty, python and rust jobs w/ cron."
Existing REQ-CRON-0003 governs those already ported targets.

The later human context also says: "A new customer should have enough in place,
cron would work for them out of the box." Treat that as source context for the
existing cron/default integration requirements, not a seventh new anchor here.
The baseline review must assess whether current REQ-CRON-0003, REQ-RUN-0301 and
the existing fixture criteria actually demonstrate that path. This packet claims
neither satisfaction nor a newly invented default auto-start/job policy.

Tracking remains run-x8d.5/.6. No broad plan revision is made before baseline
gate evidence; no project-specific packet content enters the generic oracle.
