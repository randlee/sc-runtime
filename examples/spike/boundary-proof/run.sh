#!/bin/sh
# Copy each fixture out from under examples/spike (itself a workspace) and
# run the single-crate boundary command. Exit codes are the evidence.
set -u
src=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
dest=$(mktemp -d /tmp/boundary-run.XXXXXX)
cp -R "$src"/. "$dest"/
for case in no-workspace exit-ok prefix-match direct-forbidden transitive-only direct-only-transitive direct-only-direct external-prefix external-direct; do
  echo "===== $case lint ====="
  sc-lint lint sc-boundary --root "$dest/$case"
  echo "lint_exit=$?"
  echo "===== $case analyze ====="
  sc-lint-boundary analyze --root "$dest/$case" --format text
  echo "analyze_exit=$?"
done
echo "DEST=$dest"
