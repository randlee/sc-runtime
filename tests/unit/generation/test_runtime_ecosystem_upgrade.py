"""Frozen runtime package identities and executable generic projection proof."""
from __future__ import annotations
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[3]
FROZEN = {"rmcp": "3.8.0", "utoipa": "6.0.0", "utoipa-axum": "0.3.0", "schemars": "1.2.3"}


def load_file(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


OWNED = load_file("ecosystem_owned", ROOT / "tests/integration/generation/owned_process.py")


def verify_identities(metadata, roots):
    """Follow resolved normal edges from actual roots; reject a second package identity."""
    packages = {item["id"]: item for item in metadata["packages"]}
    nodes = {item["id"]: item for item in metadata["resolve"]["nodes"]}
    pending = []
    for root in roots:
        candidates = [pid for pid, package in packages.items() if package["name"] == root]
        if len(candidates) != 1:
            raise ValueError(f"expected one actual root identity for {root}: {candidates}")
        pending.extend(candidates)
    visited = set()
    while pending:
        current = pending.pop()
        if current in visited:
            continue
        visited.add(current)
        for edge in nodes[current]["deps"]:
            if any(kind["kind"] is None for kind in edge["dep_kinds"]):
                pending.append(edge["pkg"])
    identities = {}
    for name, expected in FROZEN.items():
        found = {(packages[pid]["name"], packages[pid]["version"], packages[pid]["source"])
                 for pid in visited if packages[pid]["name"] == name}
        if len(found) != 1 or next(iter(found))[1] != expected:
            raise ValueError(f"{name}: expected exactly one {expected} normal identity, found {found}")
        identities[name] = sorted(found)
    return identities


def metadata(root):
    process = OWNED.run(["cargo", "metadata", "--locked", "--format-version", "1", "--all-features"],
        cwd=root, timeout=120, check=True, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return json.loads(process.stdout)


class RuntimeIdentityTests(unittest.TestCase):
    def test_actual_locked_workspace_uses_frozen_identities(self):
        verify_identities(metadata(ROOT), ["sc-runtime"])

    def test_second_reachable_identity_fails_even_in_same_major(self):
        document = {"packages": [{"id":"root", "name":"sc-runtime", "version":"0.2.0", "source":None}],
            "resolve":{"nodes":[{"id":"root", "deps":[]}]}}
        for name, version in FROZEN.items():
            document["packages"].append({"id":name,"name":name,"version":version,"source":"registry+one"})
            document["resolve"]["nodes"].append({"id":name,"deps":[]})
            document["resolve"]["nodes"][0]["deps"].append({"name":"alias_"+name,"pkg":name,"dep_kinds":[{"kind":None}]})
        verify_identities(document, ["sc-runtime"])
        document["packages"].append({"id":"second","name":"rmcp","version":"3.8.0","source":"git+different"})
        document["resolve"]["nodes"].append({"id":"second","deps":[]})
        edge = {"name":"hidden_alias","pkg":"second","dep_kinds":[{"kind":"dev"}]}
        document["resolve"]["nodes"][0]["deps"].append(edge)
        verify_identities(document, ["sc-runtime"])  # Unrelated/dev-only identities cannot waive or invent normal conflicts.
        edge["dep_kinds"] = [{"kind":None}]
        with self.assertRaisesRegex(ValueError, "exactly one"):
            verify_identities(document, ["sc-runtime"])
        document["packages"][-1]["version"] = "3.9.0"
        with self.assertRaisesRegex(ValueError, "exactly one"):
            verify_identities(document, ["sc-runtime"])

    def test_affected_versions_and_template_requirements_are_atomic(self):
        for name in ["sc-command", "sc-runtime", "sc-cron", "sc-cron-commands"]:
            manifest = tomllib.loads((ROOT / "crates" / name / "Cargo.toml").read_text())
            self.assertEqual(manifest["package"]["version"], "0.2.0")
            for template in ["Cargo.toml", "workspace.template.toml"]:
                spec = tomllib.loads((ROOT / "template" / template).read_text())
                self.assertEqual(spec["workspace"]["dependencies"][name]["version"], "0.2.0")
        root = tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(root["workspace"]["package"]["version"], "0.1.1")

    def test_former_utoipa5_router_is_rejected_at_public_composition_seam(self):
        fixture = ROOT / "crates/sc-runtime/tests/fixtures/utoipa5-router/Cargo.toml"
        process = OWNED.run(["cargo", "check", "--manifest-path", str(fixture), "--locked"],
            cwd=ROOT, timeout=180, capture_output=True, text=True, stdin=subprocess.DEVNULL)
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("error[E0308]: mismatched types", process.stderr)
        self.assertIn("different versions of crate `utoipa_axum`", process.stderr)
        self.assertEqual(process.stderr.count("error[E"), 1, process.stderr)

    def test_sqlite_manifest_ranges_retain_compatible_native_family(self):
        store = tomllib.loads((ROOT / "template/crates/store-sqlite/Cargo.toml").read_text())
        self.assertTrue(store["dependencies"]["sqlx"]["version"].startswith("0.9"))
        native = tomllib.loads((ROOT / "crates/sc-runtime-db-sqlite/Cargo.toml").read_text())
        cron = tomllib.loads((ROOT / "crates/sc-cron/Cargo.toml").read_text())
        self.assertTrue(native["dependencies"]["rusqlite"]["version"].startswith("0.39"))
        self.assertTrue(cron["dependencies"]["rusqlite"]["version"].startswith("0.39"))


class GenericProjectionTests(unittest.TestCase):
    def test_real_generated_projections_use_canonical_double_and_schema(self):
        driver = load_file("ecosystem_new_project", ROOT / "scripts/new_project.py")
        with tempfile.TemporaryDirectory(prefix="runtime-projections-") as directory:
            base = Path(directory)
            values = base / "values.toml"
            values.write_text('[values]\ndb="none"\nmcp=true\ncron=true\ncron_commands=true\nlicence="MIT"\n')
            app = base / "app"
            driver.cargo_generate(ROOT, app, "cron-projection-probe", values)
            driver.apply_checkout_patch(app, ROOT)
            cli = "cron-projection-probe-cli"
            tree = OWNED.run(["cargo", "tree", "-p", cli, "--no-default-features", "-e", "normal", "--prefix", "none"],
                cwd=app, timeout=120, check=True, capture_output=True, text=True, stdin=subprocess.DEVNULL)
            names = {line.split()[0] for line in tree.stdout.splitlines() if line.split()}
            forbidden = {"axum", "rmcp", "sqlx", "sc-runtime", "sc-cron", "rusqlite", "monty", "utoipa", "utoipa-axum", "schemars"}
            self.assertFalse(names & forbidden, tree.stdout)
            verify_identities(metadata(app), ["sc-runtime", "cron-projection-probe-daemon"])
            support = app / "tests/support"
            support.mkdir(parents=True)
            shutil.copyfile(ROOT / "tests/contracts/phase-c/executor_double.rs", support / "executor_double.rs")
            target = app / "crates/cli/tests/c3_cron_projections.rs"
            shutil.copyfile(ROOT / "tests/contracts/phase-c/projection_tests.rs", target)
            result = OWNED.run(["cargo", "test", "-p", "cron-projection-probe-cli", "--test", "c3_cron_projections"],
                cwd=app, timeout=900, capture_output=True, text=True, stdin=subprocess.DEVNULL)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("test result: ok.", result.stdout)


if __name__ == "__main__":
    if sys.argv[1:2] == ["--metadata"]:
        parser = argparse.ArgumentParser()
        parser.add_argument("--metadata", type=Path, required=True)
        parser.add_argument("--root", action="append", required=True)
        arguments = parser.parse_args()
        print(json.dumps(verify_identities(json.loads(arguments.metadata.read_text()), arguments.root), indent=2))
    else:
        unittest.main()
