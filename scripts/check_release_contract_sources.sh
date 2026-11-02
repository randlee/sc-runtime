#!/usr/bin/env bash
# Compatibility entry point; the Python runner owns and bounds its child tree.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec python3 "$ROOT/scripts/check_release_contract_sources.py"
