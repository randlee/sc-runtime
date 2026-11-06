"""Generated Monty feature and checkout/release inventory contracts."""
import ast
import hashlib
import importlib.util
import os
import re
import sys
from pathlib import Path
import shutil
import tempfile
import tomllib
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
INTERNAL = {"sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron", "sc-cron-commands"}


def driver():
    spec = importlib.util.spec_from_file_location("monty_generation", ROOT / "scripts/new_project.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


from tests.integration.generation.generate_owned import cargo_generate, run_generation


class MontyExampleTests(unittest.TestCase):
    def test_failed_generation_reaps_ready_pipe_writer(self):
        from tests.integration.generation.test_owned_process import OwnedProcessTests

        # The generator exits only after its child acknowledges readiness. That
        # child retains captured output, so pipe-only capture cannot finish.
        OwnedProcessTests().witness(True, runner=run_generation)

    def test_stalled_generation_times_out_and_reaps_ready_pipe_writer(self):
        from tests.integration.generation.test_owned_process import OwnedProcessTests

        OwnedProcessTests().witness(False, runner=run_generation)

    def test_manifest_inventory_and_default_off_features(self):
        for name in ("Cargo.toml", "workspace.template.toml"):
            dependencies = tomllib.loads((ROOT / "template" / name).read_text())["workspace"]["dependencies"]
            self.assertEqual({name for name in dependencies if name in INTERNAL}, INTERNAL)
            for name in INTERNAL:
                self.assertEqual(dependencies[name]["version"], "0.2.0")
            for name in ("sc-observability", "sc-observability-types"):
                self.assertEqual(dependencies[name]["version"], "=1.5.0")
        generation = driver()
        with tempfile.TemporaryDirectory(prefix="runtime-default-features-") as temporary:
            base = Path(temporary)
            for db in ("none", "sqlite"):
                values = base / f"{db}.toml"
                values.write_text(f'[values]\ndb="{db}"\nmcp=true\ncron=false\ncron_commands=false\nlicence="MIT"\n')
                app = base / db
                generation.cargo_generate(ROOT, app, "default-off-probe", values)
                manifest = tomllib.loads((app / "crates/daemon/Cargo.toml").read_text())
                self.assertEqual(set(manifest["features"]["cron"]), {"dep:sc-cron", "dep:async-trait"})
                self.assertEqual(manifest["features"]["monty"], ["cron", "sc-cron/monty"])
                self.assertNotIn("cron", manifest["features"]["default"])
                self.assertNotIn("monty", manifest["features"]["default"])
                self.assertNotIn("cron-commands", manifest["features"]["default"])
                self.assertEqual(manifest["dependencies"]["sc-cron"], {"workspace": True, "optional": True})
        for path in (ROOT / "template/crates").glob("*/Cargo.toml"):
            if path.parent.name != "daemon":
                self.assertNotRegex(path.read_text(), r"(?m)^sc-cron\s*(?:=|\.)", str(path))

    def test_local_maintainer_lock_is_not_copied_into_generated_consumer(self):
        with tempfile.TemporaryDirectory(prefix="runtime-stale-template-lock-") as temporary:
            base = Path(temporary)
            source = base / "source"
            source.mkdir()
            shutil.copytree(ROOT / "template", source / "template",
                            ignore=shutil.ignore_patterns("target", "__pycache__"))
            (source / "template/Cargo.lock").write_text(
                'version = 4\n[[package]]\nname = "stale-maintainer-identity"\nversion = "9.9.9"\n')
            values = base / "values.toml"
            values.write_text('[values]\ndb="none"\nmcp=false\ncron=false\ncron_commands=false\nlicence="MIT"\n')
            app = base / "app"
            cargo_generate(source, app, "fresh-lock-probe", values)
            self.assertFalse((app / "Cargo.lock").exists(), "local template lock must never become a consumer baseline")

    def test_checked_source_and_application_closure_documentation(self):
        source = (ROOT / "template/examples/monty/cron.py").read_text()
        tree = ast.parse(source)
        self.assertEqual({node.name for node in tree.body if isinstance(node, ast.FunctionDef)}, {"cron_stop", "cron_tick"})
        self.assertIn('sc_compose_run("render",', source)
        self.assertIn('atm_run("agent",', source)
        doc = (ROOT / "docs/sc-cron/monty-generated-example.md").read_text()
        for token in ("jev_call", "MontyHostRegistry", "with_monty_hosts", "application-supplied", "no signing", "deadline"):
            self.assertIn(token, doc)
        bench = (ROOT / "crates/sc-cron/tests/sc_compose_bench.rs").read_text()
        production = bench.split("fn production_bridge_once()", 1)[1].split("async fn production_bridge_run", 1)[0]
        self.assertIn('sc_compose_run("render",', production)
        self.assertNotIn("render_template(", production)

    def test_fresh_temp_render_and_checkout_release_patch_contract(self):
        if shutil.which("cargo") is None:
            self.skipTest("cargo unavailable")
        generation = driver()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            values = root / "values.toml"
            values.write_text('[values]\ndb = "sqlite"\nmcp = true\nlicence = "MIT"\n')
            result = run_generation(["cargo", "generate", "--path", str(ROOT / "template"), "--name", "widget-app", "--destination", str(root), "--silent", "--allow-commands", "--template-values-file", str(values)])
            self.assertEqual(result.returncode, 0, result.stderr or result.stdout)
            output = root / "widget-app"
            self.assertTrue((output / "examples/monty/cron.py").is_file())
            daemon = tomllib.loads((output / "crates/daemon/Cargo.toml").read_text())
            self.assertEqual(daemon["package"]["name"], "widget-app-daemon")
            release = tomllib.loads((output / "Cargo.toml").read_text())
            self.assertNotIn("patch", release)
            self.assertTrue(all("path" not in spec for spec in release["workspace"]["dependencies"].values() if isinstance(spec, dict)))
            self.assertEqual(set(generation.patch_crate_names(ROOT)), INTERNAL)
            generation.apply_checkout_patch(output, ROOT)
            checkout = tomllib.loads((output / "Cargo.toml").read_text())
            self.assertEqual(set(checkout["patch"]["crates-io"]), INTERNAL)
            self.assertEqual(Path(checkout["patch"]["crates-io"]["sc-cron"]["path"]), ROOT / "crates/sc-cron")
            source = (output / "crates/daemon/src/observability.rs").read_text()
            self.assertIn("struct LoggerAdapter", source)
            self.assertIn("self.logger.try_log(event)", source)
            dependencies = checkout["workspace"]["dependencies"]
            self.assertEqual(dependencies["sc-observability"]["version"], "=1.5.0")
            self.assertEqual(dependencies["sc-observability-types"]["version"], "=1.5.0")
            self.assertNotIn("sc-observability-dev-adapter", dependencies)
            with patch.dict("os.environ", {"SC_RUNTIME_RELEASE": "0"}):
                self.assertTrue(generation.release_mode(True))
                self.assertFalse(generation.release_mode(False))

    def test_canonical_shim_needs_no_generation_mode(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text()
        self.assertNotIn("--development-observability", workflow)
        self.assertIn("SC_RUNTIME_RELEASE: ${{ github.ref_type == 'tag' && '1' || '' }}", workflow)
        self.assertIn('run: python3 scripts/new_project.py --patch-checkout "$RUNNER_TEMP/plain"', workflow)
        self.assertIn('new dest *args:', (ROOT / "Justfile").read_text())
        self.assertIn('scripts/new_project.py {{dest}} {{args}}', (ROOT / "Justfile").read_text())


if __name__ == "__main__":
    unittest.main()
