"""Documentation-only diffs do not require the runtime CI jobs."""

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def load_ci_scope():
    spec = importlib.util.spec_from_file_location(
        "ci_scope", ROOT / ".github" / "scripts" / "ci_scope.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


ci_scope = load_ci_scope()


class CiScopeTests(unittest.TestCase):
    def test_docs_only_change_skips_runtime(self):
        self.assertFalse(
            ci_scope.requires_runtime(
                [
                    "docs/requirements.md",
                    "docs/plans/phase-b/anchors.md",
                    "README.md",
                ]
            )
        )

    def test_crate_template_and_workflow_changes_require_runtime(self):
        self.assertTrue(ci_scope.requires_runtime(["crates/sc-runtime/src/lib.rs"]))
        self.assertTrue(ci_scope.requires_runtime(["dev/observability-logger-adapter/src/lib.rs"]))
        self.assertTrue(ci_scope.requires_runtime(["template/Justfile"]))
        self.assertTrue(ci_scope.requires_runtime([".github/workflows/crates.yml"]))
        self.assertTrue(ci_scope.requires_runtime(["Cargo.lock"]))

    def test_empty_path_does_not_require_runtime(self):
        self.assertFalse(ci_scope.requires_runtime(["", "docs/architecture.md"]))
