"""Generated Monty feature and checkout/release inventory contracts."""
import ast
import hashlib
import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
INTERNAL = {"sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron"}


def driver():
    spec = importlib.util.spec_from_file_location("monty_generation", ROOT / "scripts/new_project.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class MontyExampleTests(unittest.TestCase):
    def test_manifest_inventory_and_default_off_features(self):
        for name in ("Cargo.toml", "workspace.template.toml"):
            dependencies = tomllib.loads((ROOT / "template" / name).read_text())["workspace"]["dependencies"]
            self.assertEqual({name for name in dependencies if name in INTERNAL}, INTERNAL)
            for name in INTERNAL:
                self.assertEqual(dependencies[name]["version"], "0.1.1")
            for name in ("sc-observability", "sc-observability-types"):
                self.assertEqual(dependencies[name]["version"], "1.5.1")
        text = (ROOT / "template/crates/daemon/Cargo.toml").read_text()
        for db in ("none", "sqlite"):
            # The only Liquid block in this manifest chooses the existing defaults.
            rendered = text.replace('{% if db == "none" %}\ndefault = []\n{% else %}\ndefault = ["general-db"]\n{% endif %}', 'default = []' if db == "none" else 'default = ["general-db"]')
            rendered = rendered.replace("{% if mcp %}\n", "").replace("{% endif %}\n", "")
            manifest = tomllib.loads(rendered)
            self.assertEqual(manifest["features"]["cron"], ["dep:sc-cron"])
            self.assertEqual(manifest["features"]["monty"], ["cron", "sc-cron/monty"])
            self.assertNotIn("cron", manifest["features"]["default"])
            self.assertNotIn("monty", manifest["features"]["default"])
            self.assertEqual(manifest["dependencies"]["sc-cron"], {"workspace": True, "optional": True})
        for path in (ROOT / "template/crates").glob("*/Cargo.toml"):
            if path.parent.name != "daemon":
                self.assertNotIn("sc-cron", path.read_text(), str(path))

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
            result = subprocess.run(["cargo", "generate", "--path", str(ROOT / "template"), "--name", "widget-app", "--destination", str(root), "--silent", "--allow-commands", "--template-values-file", str(values)], capture_output=True, text=True, check=False)
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
            snapshot = {str(path.relative_to(output)): hashlib.sha256(path.read_bytes()).hexdigest() for path in output.rglob("*.rs")}
            generation.apply_development_observability(output, ROOT)
            development = tomllib.loads((output / "Cargo.toml").read_text())["workspace"]["dependencies"]
            self.assertEqual(development["sc-observability"]["version"], "=1.5.0")
            self.assertEqual(development["sc-observability-types"]["version"], "1.5.1")
            self.assertIn("path", development["sc-observability-types"])
            self.assertEqual(development["sc-observability-dev-adapter"]["version"], "0.0.0")
            # Development selection touches the manifest only, never rendered Rust.
            self.assertEqual(snapshot, {str(path.relative_to(output)): hashlib.sha256(path.read_bytes()).hexdigest() for path in output.rglob("*.rs")})
            with patch.dict("os.environ", {"SC_RUNTIME_RELEASE": "0"}):
                self.assertTrue(generation.release_mode(True))
                self.assertFalse(generation.release_mode(False))

    def test_development_ci_is_explicit_and_recipe_forwards_flags(self):
        workflow = (ROOT / ".github/workflows/template.yml").read_text()
        self.assertIn("${{ github.ref_type != 'tag' && '--development-observability' || '' }}", workflow)
        self.assertIn("SC_RUNTIME_RELEASE: ${{ github.ref_type == 'tag' && '1' || '' }}", workflow)
        self.assertIn('run: python3 scripts/new_project.py --patch-checkout "$RUNNER_TEMP/plain" --development-observability', workflow)
        self.assertIn('new dest *args:', (ROOT / "justfile").read_text())
        self.assertIn('scripts/new_project.py {{dest}} {{args}}', (ROOT / "justfile").read_text())


    def test_plain_checkout_cli_applies_explicit_development_selection(self):
        generation = driver()
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp)
            (output / "Cargo.toml").write_text((ROOT / "template/workspace.template.toml").read_text())
            with patch.object(generation, "repo_root", return_value=ROOT):
                generation.main(["--patch-checkout", str(output), "--development-observability"])
            manifest = tomllib.loads((output / "Cargo.toml").read_text())
            self.assertEqual(set(manifest["patch"]["crates-io"]), INTERNAL)
            self.assertEqual(manifest["workspace"]["dependencies"]["sc-observability"]["version"], "=1.5.0")
            self.assertEqual(manifest["workspace"]["dependencies"]["sc-observability-types"]["version"], "1.5.1")

    def test_development_release_conflict_fails_before_output(self):
        generation = driver()
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "absent"
            for explicit, environment in [(True, "0"), (False, "1")]:
                with self.subTest(explicit=explicit, environment=environment), patch.dict("os.environ", {"SC_RUNTIME_RELEASE": environment}):
                    with self.assertRaises(SystemExit):
                        generation.generate(dest, Path(tmp) / "missing-answers.json", explicit, True)
                    self.assertFalse(dest.exists())



if __name__ == "__main__":
    unittest.main()
