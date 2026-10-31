"""CI build reuse keeps independent proofs and cleans its shared fixture."""

import importlib.util
import os
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]


def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class BuildReuseTests(unittest.TestCase):
    def test_generation_child_uses_selected_target_without_changing_local_default(self):
        runner = load("generation_runner_cache_test", "scripts/run_generation_tests.py")
        observed = []

        class ChildEnvironment(unittest.TestCase):
            def runTest(self):
                observed.append(subprocess.check_output(
                    [sys.executable, "-c", "import os; print(os.environ.get('CARGO_TARGET_DIR', 'unset'))"],
                    text=True, timeout=10,
                ).strip())

        for selected, expected in (("./chosen-cache", str(Path("chosen-cache").resolve())), (None, "local-target")):
            with self.subTest(selected=selected), patch.dict(os.environ, {"CARGO_TARGET_DIR": "local-target"}):
                os.environ.pop("SC_RUNTIME_GENERATED_TARGET_DIR", None)
                if selected:
                    os.environ["SC_RUNTIME_GENERATED_TARGET_DIR"] = selected
                # Exercise main's real child environment without running the full suite.
                with patch.object(runner, "SUITES", ()), patch.object(
                    runner.unittest.TestLoader, "discover", return_value=unittest.TestSuite([ChildEnvironment()])
                ), self.assertRaisesRegex(SystemExit, "0"):
                    runner.main()
            self.assertEqual(observed[-1], expected)

    def test_identical_fixture_is_generated_once_and_cleaned_after_class(self):
        sys.path.insert(0, str(ROOT))
        module = load("observability_cache_test", "tests/integration/generation/test_observability.py")
        generated = []
        seen = []

        def generate(command, **kwargs):
            app = Path(command[2])
            manifest = app / "crates/daemon/Cargo.toml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text('[package]\nname = "widget-app-daemon"\n')
            generated.append(app)

        class TwoAssertions(module.GeneratedObservabilityIntegration):
            def test_first(self):
                seen.append(self.generated_app())

            def test_second(self):
                seen.append(self.generated_app())

        suite = unittest.TestSuite(TwoAssertions(name) for name in ("test_first", "test_second"))
        with patch.object(module, "run_owned", side_effect=generate):
            result = unittest.TestResult()
            suite.run(result)
        self.assertTrue(result.wasSuccessful(), result.errors + result.failures)
        self.assertEqual(len(generated), 1)
        self.assertEqual(seen[0], seen[1])
        self.assertFalse(generated[0].exists(), "class cleanup must remove the temporary app")
        self.assertIsNone(TwoAssertions._generated_fixture)

    def test_workflows_keep_independent_coverage_commands(self):
        template = (ROOT / ".github/workflows/template.yml").read_text()
        crates = (ROOT / ".github/workflows/crates.yml").read_text()
        for command in ("just setup", "just lint", "just test", "just build-cli", "just build-daemon"):
            self.assertIn(command, template.split("- name: Consumer recipes", 1)[1])
        for selector in ("generated-workspace", "generated-general-db-off", "lifecycle-signal"):
            self.assertIn("run: just test " + selector, template)
        for selector in ("owned-process-windows", "windows-hook-default", "windows-hook-monty", "windows-host-monty"):
            self.assertIn("run: just test " + selector, crates)
        self.assertIn("name: just lint and just test", crates)
        self.assertIn("template-${{ matrix.fixture }}-", template)
        self.assertIn("target: workspace", crates)


if __name__ == "__main__":
    unittest.main()
