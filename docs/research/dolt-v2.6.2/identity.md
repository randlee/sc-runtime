# Dolt identity, authorization, credentials, provenance, and snapshot evidence

Source audited: `doltdb/dolt` at pinned commit `46d8805d8556599eb0a6472892e67e3bb1e0de03` (provided as v2.6.2). All source URLs below are immutable commit URLs with line anchors. Source and test files were inspected; no tests were run. The installed Dolt 2.3.5 binary was not used as evidence.

## Claims

### ID-01 — Commit author fields are attribution metadata, not authenticated actor identity

`user.name` / `user.email` are read from Dolt config, with local repository config available and config writes preferring local when present; `GetNameAndEmail` reads through the config hierarchy. SQL commit procedure constructs commit staged properties and accepts `--author`; explicit `--author` fills author and, for compatibility when no committer is separately set, committer too. The SQL CLI client also seeds `DOLT_AUTHOR_*` and `DOLT_COMMITTER_*` environment values into session variables. These values are mutable attribution inputs. Do not use them as proof of the SQL account, human, agent/session, team, or repo that performed an action.

Evidence: [config scopes and local/global behavior](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/config.go#L39-L48), [config identity read](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/config.go#L236-L254), [SQL client commit identity env overrides](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/cmd/dolt/commands/engine/sqlengine.go#L701-L729), [SQL DOLT_COMMIT attribution handling](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/sqle/dprocedures/dolt_commit.go#L277-L305).

### ID-02 — Dolt has SQL account authentication and branch-scoped authorization

The server's SQL account identity/password and privileges are distinct from commit metadata. The branch-control table carries `database`, `branch`, SQL `user`, `host`, and permission set (`admin`, `write`, `merge`, `read`). Integration tests exercise users with scoped branch grants and denial of unauthorized control-table modifications. This is server authorization policy; do not assume it creates app-level stakeholder/repository attribution or an immutable activity trace.

Evidence: [branch-control table schema and permission values](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/sqle/dtables/branch_control_table.go#L30-L70), [branch-control test scenarios](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/sqle/enginetest/branch_control_test.go#L728-L807), [commit authorization check](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/sqle/dprocedures/dolt_commit.go#L159-L179).

### ID-03 — Remote authentication sources are client/process scoped, with a global-only helper

For remotes API calls, the dial provider can use explicit `UserPassConfig`, stored Dolt JWK credentials, a configured credential helper, or `DOLT_REMOTE_PASSWORD` with a username. `DOLT_REMOTE_PASSWORD` is read from process environment. `UserDoltCreds` reads a key ID from config and then a JWK under the user's credentials directory. The credential-helper lookup explicitly reads **global** config; source tests show local config is ignored. Thus a local per-repository Dolt config does exist, but it does not imply per-database remote secrets or per-remote helper selection. With a shared server process, process environment is shared; do not claim each SQL client has an independent `DOLT_REMOTE_PASSWORD` for pushes. The upstream remotes API test shows server-side SQL accounts/passwords and `CLONE_ADMIN` grants govern remote API authentication.

Evidence: [global credential helper lookup](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/grpc_dial_provider.go#L181-L199), [environment password input](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/grpc_dial_provider.go#L201-L216), [remote credential selection](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/grpc_dial_provider.go#L231-L255), [config key to JWK file](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/environment.go#L1025-L1045), [helper scope tests](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/env/grpc_dial_provider_test.go#L38-L66), [remote API SQL account/grant setup](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/integration-tests/go-sql-server-driver/remotesapi_auth_test.go#L102-L112).

### ID-04 — Upstream SQL command construction parameterizes data values; identifiers still need strict handling

`dolt branch` generates a `CALL DOLT_BRANCH(?,...)` query with positional values. `dolt commit` query construction also uses placeholders for commit message, date, and author. `dolt log` uses query parameters for revision values and options, and its source calls out SQL injection avoidance. These are good patterns for procedure arguments. SQL placeholders do not parameterize database/table identifiers (for example revision-qualified database names like `repo/branch`); application code building SQL for selected databases or table names must use a proper identifier-quoting API or a strict allowlist and still bind ordinary values. No claim is made that all arbitrary user SQL is safe.

Evidence: [branch procedure parameter generation](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/cmd/dolt/commands/branch.go#L441-L460), [commit procedure values](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/cmd/dolt/commands/commit.go#L193-L250), [log interpolation and bound args](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/cmd/dolt/commands/log.go#L193-L245).

### ID-05 — History/status metadata and revision reads are native; app-level human/session/team/repo trace graph is separate

`dolt_log`, `dolt_status`, commit hashes, author/committer, messages and dates provide useful Dolt-native history/status. SQL tests demonstrate `AS OF` for status and historical revisions; integration tests also read branch rows `AS OF 'HEAD'`. `dolt dump` documentation says it dumps the working set, so a refresh pipeline must explicitly choose a pinned commit/revision and export from a read-only snapshot/revision context rather than assume default dump is revision-pinned. Dolt author fields do not encode the requested multi-stakeholder repo graph. Keep app trace graph and actor/session/team/repo attribution in consumer-owned metadata, linked to immutable Dolt commit hash and operation/event IDs.

Evidence: [DOLT_LOG fixed metadata schema description](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/sqle/dtablefunctions/dolt_log.go#L37-L50), [status AS OF tests](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/libraries/doltcore/sqle/enginetest/dolt_queries_dtables.go#L60-L94), [branch snapshot query test](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/integration-tests/go-sql-server-driver/concurrent_multi_branch_test.go#L195-L206), [dump working-set semantics](https://github.com/dolthub/dolt/blob/46d8805d8556599eb0a6472892e67e3bb1e0de03/go/cmd/dolt/commands/dump.go#L65-L76).

## Capability classification

| Concern | Classification | Planning implication |
|---|---|---|
| Local and global commit name/email config | Native capability | Useful default attribution only; allow explicit DOLT_AUTHOR/COMMITTER session overrides and `--author`. |
| SQL authentication and grants | Native capability | Use SQL identity and branch control to authorize; test least privilege per database/branch. |
| Remote credentials | Native with limits | Local JWK identity and env password exist; credential helper is global-config-only in inspected version. Add consumer credential-provider abstraction and per-database secret storage/selection as application policy. |
| Commit history/status and AS OF | Native capability | Persist commit hashes and read by explicit revision; validate exact read-only export strategy. |
| Human/agent/session/team/stakeholder repository attribution graph | Consumer policy / separate app trace graph | Record authenticated actor/session/context and target stakeholder repo IDs in app-owned append-only events; do not infer them from Dolt author. |
| Credential provider semantics | Unresolved consumer design | Define precedence, key scope, rotation, redaction, audit, secret-store plugins, and env override behavior; align with version API investigation. |
| Snapshot export to SQLite refresh | Consumer-owned (outside `sc-dolt`) | Pin hash first in the consumer cache adaptor; export deterministic row/schema snapshot and verify no working-set drift during extraction. Not part of `sc-dolt` acceptance. |

## Proposed acceptance probes

1. Configure distinct SQL users with read/write/merge/admin branch-control entries; verify each `DOLT_COMMIT`, branch mutation, read, and remote operation succeeds/fails at the expected scope.
2. Set distinct local/global `user.name` and `user.email`, then `DOLT_AUTHOR_*`, `DOLT_COMMITTER_*`, and `CALL DOLT_COMMIT(... '--author', ...)`; inspect `dolt_log` and prove this attribution can differ from SQL `CURRENT_USER()`.
3. Configure local and global remote credential helper values and observe resolution; verify expected local/global behavior, env override, missing-secret, rotation, redaction, and two simultaneous consumers using different credentials against a shared server.
4. Pass hostile branch/revision strings through supported procedure value slots; separately test database/table identifier construction using the consumer's quoting/allowlist layer.
5. *(Consumer-owned acceptance, not `sc-dolt`.)* Capture a commit hash, perform concurrent commits on another branch, then export using explicit `AS OF`/revision semantics and compare a deterministic manifest (schema, row count, stable content digest) to ensure refresh is pinned and read-only.
6. Emit application trace events for human, agent, session, team, and several stakeholder repos; verify each event links to authenticated SQL principal plus exact Dolt commit hash, while Dolt commit author remains optional display metadata.
