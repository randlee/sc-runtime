# Logging patch proposal status

Source: exact git archive 725420b5980e4a0e4a85db3d77a227c9b18a8c88 of sc-runtime.
See provenance.json for archive/file hashes, immutable v0.1.1 tag identity, and
registry 0.1.1 byte comparisons. pristine/ is untouched; source/ has exactly the
one reexport module plus the independent five-file consumer fixture.

Owner-review artifacts:

- logging-reexports.patch: only the public module/type reexports.
- logging-consumer-fixture.patch: separate resolver-root receiver/run/graph proof.
- logging-proposal.patch: both proposals, applicable at the upstream repository root.

Validation complete: all three patches pass git apply --check against pristine;
Rust syntax/format check passes; standalone TOML has no direct sc-command,
defaults or patch override; three fast checker tests pass. Parent granted the
single heavy-build slot after C4 QA released it. The actual isolated receiver
compiled and ran, asserted attached/launched/finished scheduler events and a
successful persisted outcome. The actual locked normal/build graph passes.

Restoring pristine lib.rs causes the consumer compile to fail specifically with
E0432, missing sc_cron::observability. Restoring the proposed module makes the
same locked receiver run pass. One target directory and CARGO_INCREMENTAL=0
were used throughout. The first online attempt was stopped after DNS failure;
successful compilation and metadata resolution used cached dependencies offline.
See validation.json for exact patch/metadata/lock hashes and all log paths;
consumer.Cargo.lock preserves the validated independent resolve outside the patch.
The single heavy-build slot has been released; no builds remain running.

No patch has been applied to the real upstream checkout or beads-cache;
no consumer path/git/patch dependency has been added. All five C1 findings remain
OPEN, including the separately coordinated lifecycle API/proof matrix.
