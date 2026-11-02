"""Run real daemon signal tests only on the dedicated ephemeral CI runner."""

import os
import sys
import unittest
from pathlib import Path


def main() -> int:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from tests.integration.generation.test_observability import signal_test_enabled

    try:
        enabled = signal_test_enabled(os.environ)
    except ValueError:
        enabled = False
    if not enabled:
        print(
            "LIFECYCLE.ISOLATION real signal tests require the dedicated GitHub-hosted ephemeral job — run the lifecycle CI job",
            file=sys.stderr,
        )
        return 1
    suite = unittest.defaultTestLoader.loadTestsFromName(
        "tests.integration.generation.test_observability.GeneratedObservabilityIntegration.test_signal_shutdown_isolated_ci",
    )
    return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
