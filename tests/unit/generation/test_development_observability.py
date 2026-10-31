"""The development renderer changes only the logger bridge and manifests."""

import importlib.util
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def driver():
    spec = importlib.util.spec_from_file_location("development_generation", ROOT / "scripts/new_project.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DevelopmentObservabilityTests(unittest.TestCase):
    def test_real_adapter_selection_is_explicit_and_repeatable(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            source = output / "crates/daemon/src/observability.rs"
            source.parent.mkdir(parents=True)
            manifest = output / "crates/daemon/Cargo.toml"
            manifest.write_text('[dependencies]\nsc-observability.workspace = true\n')
            (output / "Cargo.toml").write_text((ROOT / "template/workspace.template.toml").read_text())
            source.write_text((ROOT / "template/crates/daemon/src/observability.rs").read_text())
            self.assertIn("let shared: SharedCanonicalLog = logger.clone();", source.read_text())
            self.assertNotIn("sc-observability-dev-adapter", manifest.read_text())
            generation = driver()
            generation.apply_development_observability(output, ROOT)
            once = {path: path.read_bytes() for path in (source, manifest, output / "Cargo.toml")}
            generation.apply_development_observability(output, ROOT)
            self.assertEqual(once, {path: path.read_bytes() for path in once})
            self.assertIn("sc_observability_dev_adapter::LoggerAdapter::new(Arc::clone(&logger))", source.read_text())
            self.assertNotIn("let shared: SharedCanonicalLog = logger.clone();", source.read_text())
            deps = tomllib.loads((output / "Cargo.toml").read_text())["workspace"]["dependencies"]
            self.assertEqual(deps["sc-observability"]["version"], "=1.5.0")
            self.assertEqual(Path(deps["sc-observability-dev-adapter"]["path"]), ROOT / "dev/observability-logger-adapter")
            self.assertTrue(tomllib.loads(manifest.read_text())["dependencies"]["sc-observability-dev-adapter"]["workspace"])


if __name__ == "__main__":
    unittest.main()
