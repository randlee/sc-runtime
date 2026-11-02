# CI builds and cache reuse

The required job names and `just lint` / `just test` interfaces stay unchanged.
Crate CI retains the workspace, individual default-feature, server/test-util,
Monty, dependency-boundary, generation and Windows hook checks. Template CI
retains every fixture, its independent consumer recipes (ADR-RUN-0401), plain
cargo-generate, general-db-off and isolated real-signal proofs. Live SQL jobs
retain their services, migrations, query checks and no-skip assertions.

Crate CI's generation suite runs sequentially with a dedicated Cargo target,
separate from the root workspace target. Temporary generated projects reuse
compiled dependencies even after their source directories are removed. Cargo
still validates source, dependency, compiler and feature fingerprints. The
three observability assertion groups share one lazily generated sqlite-mcp
fixture; each actual Rust assertion still runs. The independent no-edits
snapshot and all four wizard equivalence/negative-control generations remain.
Do not parallelize generated Cargo commands against this shared target; give
concurrent suites separate target partitions instead.

The local `rust-build-cache` composite action caches Cargo registry sources,
index and archives, Git dependency databases, and the job's build targets.
Keys include OS, architecture, job/fixture coverage, exact toolchain versions,
manifest/lock/profile inputs and commit. Restore prefixes omit only the commit,
so source changes reuse dependencies and successful runs refresh immutable
caches. Matrix fixture namespaces prevent concurrent cache writers competing
for the same build entry. Cargo's internal debug/release and target directories
retain profile/target separation; incremental compilation is disabled in CI to
limit cache size. Cache misses run the same checks; caches never decide whether
to test. The cache action saves after successful jobs and scopes entries by
branch as documented by [actions/cache](https://github.com/actions/cache).

A separate exact-version cache installs sc-compose 1.7.0 with `--locked` into a
runner-temporary tool root. Every restored binary must report the exact version
before entering PATH. No Cargo credentials, generated application source,
databases, logs or Python environments are cached. Jobs have separate runner
filesystems; competing saves of the same immutable tool key cannot share a live
build directory. Windows skips this POSIX generation tool cache.

Measured baseline at `d224c62` ([Crate CI 41236706275](https://github.com/randlee/sc-runtime/actions/runs/41236706275)):
Linux job 56m25s, lint 3m49s, test 49m26s. The test step's buffered output does
not attribute that duration to individual tests. Same-head Template fixture
generation took 178–371s, while the required second consumer recipe pass took
4–6s. These are baseline observations, not predicted savings. Compare later
completed CI runs and cache hit/save sizes before claiming an improvement.

The heavy Linux crate job disables dev/test debug-symbol data through
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`; debug assertions
and every lint/test command remain enabled. Its `nodebug` cache namespace
separates those artifacts from older debug-symbol builds. Windows and fixture
cache partitions retain their existing profiles and keys.

Before restoring/building, this Rust/Python-only job reclaims the unused
Android and .NET preinstalls at `/usr/local/lib/android` and
`/usr/share/dotnet`, guarded to GitHub-hosted Linux runners. It does not remove
workspace or Cargo artifacts. Bounded `df`/`du` observations before cleanup
and after checks (including failures) expose runner headroom and build-target
sizes without masking test failures. PR103's earlier runner failure reported
`No space left on device`; it did not establish that a cache restore caused
the exhaustion or that an assertion failed. The next hosted run must verify
the fix under the complete workload.
