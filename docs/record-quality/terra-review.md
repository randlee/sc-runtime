# Terra record rewrite review

Eleven records were assigned to eleven Terra agents, one record per agent. Each returned complete Raptor authoring variables as fenced JSON. The author compared every changed field with the supplied original and current record. These are internal editing checks; they are not consuming-project stakeholder approvals.

The complete authoring variables are in `terra-rewrites.json`. Field-level old/new changes and reasons are in `terra-changes.json`. All 176 original IDs, statuses and metadata remain present. Raptor extraction after these changes reports 176 records and zero diagnostics.

## Live checks

`terra-quality-plus-10.jsonl` contains eleven successful Jev requests, exactly one request per record. Each request includes the unchanged native quality choices for previously failing or changed statements plus ten diagnostic questions. Nine records passed those targeted quality checks. REQ-RUN-0603 failed an undefined-term question; NFR-RUN-0003 failed an undefined-term question. Both received source-grounded clarifications. `terra-correction-2.jsonl` contains one request per corrected record with ten diagnostic questions; both targeted checks pass.

The extra diagnostic answers sometimes call the complete record compound while native questions classify the individual statements as passing. Those diagnostic answers are preserved; they are not substituted for the native quality policy.

These checks are targeted rechecks, not a fresh whole-corpus native-quality result. The last native whole-corpus run at 32fe478 reported 165 yes and 11 no. A later attempted one-request-per-record bundle of every native question was refused before sending because its answer event exceeded the logger’s 65536-byte limit. No limit, quality tree or passing criterion was changed. Whole-corpus acceptance remains open.

## Rewrites and material review

- REQ-RUN-0001: name the path table and preserve its existing examples/spike exemption after v0.1.0.
- REQ-RUN-0004: identify the three recipe rows and six numbered test entries; preserve all commands and CI exceptions.
- REQ-RUN-0203: separate success and failure envelope fields without changing their values, surfaces or equality rules.
- REQ-RUN-0301: replace “It” with config/local.json; retain the unresolved question about rendering that file.
- REQ-RUN-0306: name the rendered Cargo workspace and project-name argument; retain direct-generation exclusions and shared template.
- REQ-RUN-0327: retain Draft/nonbinding status; state the copy and host-aggregate outcomes already specified in its success criterion.
- REQ-RUN-0603: state file-path and byte equality, retaining every existing exclusion. Define the checkout patch using scripts/new_project.py:308–326 and tests/integration/wizard/test_output_equivalence.py:93–101.
- REQ-CMD-0003: separate the prohibition on caller-supplied version from crate-supplied envelope version; retain the unresolved version type/value.
- REQ-CRON-0001: identify the historical five-library set without adding publication order; preserve versions, feature isolation and cleanup conditions.
- NFR-RUN-0003: use the existing Job/Crate table and documented-API reuse outcome; preserve governing exceptions and prohibited-list amendments. Correct the dangling list reference.
- NFR-TRN-0002: separate UDS and TCP requirements, naming sc-transport explicitly; retain reqwest 0.13 and every connector/raw-stream prohibition.

## Disclosed outcome tightenings

- NFR-RUN-0003: old text called for “standard, widely used” building blocks. New text requires reuse of the listed crate capability through its documented API, subject to the existing governing exceptions. The record supplies the concrete table and API outcome but no popularity measure. No numeric popularity threshold was invented.
- REQ-RUN-0327: old text used “same or similar” schemas. New statements require the same schema for copy generation and support for one-sided tables and columns for host aggregation. These outcomes come from its existing success criterion. Other schema differences are not prohibited, and the record remains deferred and nonbinding.

Actual upstream stakeholder reviews remain pending. No agent response is counted as stakeholder approval.
