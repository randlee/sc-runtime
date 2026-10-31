"""Run real daemon signal tests only on the dedicated ephemeral CI runner."""

import os
import unittest
import sys
from pathlib import Path

REQUIRED_MARKERS = {
    "SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST": "1",
    "GITHUB_ACTIONS": "true",
    "RUNNER_ENVIRONMENT": "github-hosted",
}


def is_ephemeral_signal_job() -> bool:
    return all(os.environ.get(name) == value for name, value in REQUIRED_MARKERS.items())


def main() -> int:
    if not is_ephemeral_signal_job():
        print(
            "LIFECYCLE.ISOLATION real signal tests require the dedicated GitHub-hosted ephemeral job — run the lifecycle CI job",
            file=sys.stderr,
        )
        return 1
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    suite = unittest.defaultTestLoader.loadTestsFromName(
        "tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_signal_shutdown_isolated_ci",
    )
    return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
