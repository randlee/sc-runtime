# C5 integration of the C1 prerequisite

Merged C1 `8a34950ed5f6701d355fd12bae129e36723b206d` into C5 based on
`cc15f4aae10d7be763b9dc138becf3a145f6af94` before the next development gate.

The shared C5 controller remains the implementation of `WorkerStopControl`.
It already supplies C1’s worker-independent factory and application completion
acknowledgement, with additional registered-work and framework-owner checks.
The duplicate C1 controller implementation is therefore replaced by that existing
implementation. C1’s external ordinary-feature consumer and public-factory test
are retained. Query-unsupported errors receive the same provider recovery guidance.
The boundary retains every C1 export plus C5’s observability facade.

Library lifecycle and transaction tests, the external application-control and
cron consumers, custom context, public schema, normal dependency graphs, budget
privacy doctests, strict Clippy, formatting and exact sc-lint 0.8.4 gates were
checked locally. Detailed receipts remain outside Git. These are macOS checks;
independent QA and hosted cross-platform CI remain separate gates.

C5’s SQLite version/features, advanced shutdown implementation, canonical owned
process helpers, orchestration history and ANC-011/012 records remain unchanged.
The approved ANC-007 v3 and ANC-008 v4 records come from the C1 prerequisite.
