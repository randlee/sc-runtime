#!/usr/bin/env bash
# The hosted release runner has Python but no generation dependencies installed.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENV="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/sc-release-contracts.XXXXXX")"
trap 'rm -rf -- "$VENV"' EXIT

python3 -m venv "$VENV"
"$VENV/bin/python" -m pip install --disable-pip-version-check -r "$ROOT/requirements-generation.txt"
cd "$ROOT"
"$VENV/bin/python" -m unittest tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_release_contract_sources
