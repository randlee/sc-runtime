# Native Dolt v2.6.2 compatibility probe

## Provenance and isolation

- Source checkout/tag: `dolthub/dolt` v2.6.2, commit `46d8805d8556599eb0a6472892e67e3bb1e0de03`.
- Official archive: `dolt-darwin-arm64.tar.gz` from the [v2.6.2 release assets](https://github.com/dolthub/dolt/releases/expanded_assets/v2.6.2). Published and downloaded SHA-256 both: `deb095a13f8b75a7c7f1fdb04eed8d8ffdd5602e327945e88523e598594ed3c6`. Extracted binary SHA-256: `333ccb0e04af731f16b05e94956f13b1e4fa6f3b88553c63f1443b5d70ea3817`.
- `dolt version --verbose`: `dolt version 2.6.2`; the command also warned that it could not query the latest version. The local v2.6.2 tag resolves to the pinned source commit.
- Five empty databases, Dolt root/config, and a PyMySQL 2.2.8 virtual environment live under this `probes/` directory. Global Dolt config was not touched. SQL servers listened only on `127.0.0.1`, ports 64429 and 64644; neither server used port 3306. Both owned server PIDs (41900, 43681) were terminated and waited; exit code 0.
- A GitHub API ref lookup returned HTTP 403 once. The local tag commit matched source and the official release asset page exposed a published digest; the downloaded archive verified successfully.

Exact SQL statements, rows, errors, commit hashes, server PID/port, archive checksums and cleanup status are in [probe-results.json](./probe-results.json). Full captured outputs are [probe-output.txt](./probe-output.txt), [supplement-output.txt](./supplement-output.txt), and server logs.

## Results

- **Fast-forward:** `main` HEAD advanced to the feature branch's exact commit hash; the feature row appeared.
- **Clean three-way:** `main` and `feature` added different rows. Preview returned no conflict rows; hash-pinned `dolt_diff(main_hash, feature_hash, 't')` showed each branch's added row. `CALL DOLT_MERGE('feature')` succeeded and all three rows appeared. A subsequent SQL `ROLLBACK` did not undo the already-published merge or restore the prior HEAD hash.
- **Conflict under autocommit:** preview returned `t, 1 data conflict, 0 schema conflicts`. `CALL DOLT_MERGE('side')` errored that the autocommit transaction was rolled back. HEAD and the main working row remained unchanged, and there were no conflict rows. `--abort` then correctly returned “There is no merge to abort.”
- **Conflict in explicit SQL transaction:** merge returned `conflicts found` with one conflict and left HEAD unchanged. SQL `ROLLBACK` preserved the main row and HEAD and removed conflict state. In a separate attempt, `CALL DOLT_MERGE('--abort')` returned `merge aborted`; after COMMIT, the main row and HEAD were still unchanged.
- **Dirty working set protection:** a main working edit that would be stomped by merging side caused `DOLT_MERGE` to error “local changes would be stomped”; the row and dirty status remained. A branch checkout with a dirty main working edit changed the active view to side but retained the edit in `dirty/main`'s working set when queried afterward.

No native SQLx/Rust driver test was attempted. This probe establishes behavior for the verified Dolt binary and temporary PyMySQL/MySQL wire client only.
