"""Release guard rejects local substitutions and incomplete Cargo evidence."""

import copy
import importlib.util
from pathlib import Path
import tempfile
import json
import tomllib
import unittest

SCRIPT = Path(__file__).resolve().parents[2] / "scripts/check_observability_release.py"
spec = importlib.util.spec_from_file_location("observability_guard", SCRIPT)
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


def registry_metadata():
    return {
        "packages": [{"name": guard.NAME, "version": "1.5.1", "id": "registry-id",
                      "source": guard.REGISTRY, "manifest_path": "/registry/source/Cargo.toml"}],
        "resolve": {"nodes": [{"id": "registry-id"}]},
    }


class ReleaseGuardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.metadata = registry_metadata()

    def check(self, manifest=None, metadata=None):
        guard.validate({} if manifest is None else manifest,
                       self.metadata if metadata is None else metadata, self.root)

    def test_clean_registry(self):
        self.check()

    def test_active_patch_rejected_even_when_metadata_is_registry(self):
        self.check_raises({"patch": {"crates-io": {guard.NAME: {"path": "dev/shim"}}}})
        self.check_raises({"patch": {"crates-io": {"alias": {"package": guard.NAME, "version": "1.5.1"}}}})

    def test_renamed_relative_shim_path_rejected(self):
        shim = self.root / "dev" / "renamed"
        shim.mkdir(parents=True)
        (shim / "Cargo.toml").write_text('[package]\nname = "sc-observability-types"\nversion = "1.5.1"\n')
        self.check_raises({"patch": {"crates-io": {"alias": {"path": "dev/../dev/renamed"}}}})

    def test_unreadable_path_patch_fails_closed(self):
        self.check_raises({"patch": {"crates-io": {"alias": {"path": "missing"}}}})

    def test_workspace_and_direct_path_dependencies_rejected_before_resolution(self):
        path_spec = {"version": "1.5.1", "path": "dev/observability-types-shim"}
        self.check_raises({"workspace": {"dependencies": {guard.NAME: path_spec}}})
        self.check_raises({"workspace": {"dependencies": {"alias": {**path_spec, "package": guard.NAME}}}})
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            with self.subTest(kind=kind):
                self.check_raises({kind: {guard.NAME: path_spec}})
                self.check_raises({"target": {"cfg(unix)": {kind: {guard.NAME: path_spec}}}})

    def test_renamed_workspace_dependency_inspects_path_manifest(self):
        shim = self.root / "renamed"
        shim.mkdir()
        (shim / "Cargo.toml").write_text('[package]\nname = "sc-observability-types"\nversion = "1.5.1"\n')
        self.check_raises({"workspace": {"dependencies": {"alias": {"path": "./renamed"}}}})

    def test_dev_adapter_declarations_and_resolved_package_rejected(self):
        adapter = guard.DEV_ADAPTER
        for entry in ({adapter: {"path": "dev/adapter"}},
                      {"alias": {"package": adapter, "path": "dev/adapter"}}):
            self.check_raises({"workspace": {"dependencies": entry}})
            self.check_raises({"dependencies": entry})
            self.check_raises({"patch": {"crates-io": entry}})
        metadata = copy.deepcopy(self.metadata)
        metadata["packages"].append({"name": adapter, "version": "0.0.0", "id": "adapter-id", "source": None, "manifest_path": str(self.root / "dev/adapter/Cargo.toml")})
        metadata["resolve"]["nodes"].append({"id": "adapter-id"})
        self.check_raises(metadata=metadata)

    def test_local_resolved_package_rejected(self):
        for source, path in [(None, str(self.root / "dev/shim/Cargo.toml")),
                             (guard.REGISTRY, str(self.root / "renamed/Cargo.toml")),
                             (None, "/elsewhere/Cargo.toml")]:
            with self.subTest(source=source, path=path):
                metadata = copy.deepcopy(self.metadata)
                metadata["packages"][0].update(source=source, manifest_path=path)
                self.check_raises(metadata=metadata)

    def test_missing_or_malformed_metadata_fails_closed(self):
        cases = [None, {}, {"packages": []}, {"packages": [], "resolve": {"nodes": []}},
                 {"packages": [None], "resolve": {"nodes": [{"id": "x"}]}},
                 {"packages": self.metadata["packages"], "resolve": {"nodes": [{"id": "other"}]}},
                 {"packages": self.metadata["packages"], "resolve": {"nodes": [None]}}]
        for case in cases:
            with self.subTest(case=case):
                with self.assertRaises(ValueError):
                    guard.validate({}, case, self.root)
        for field, value in [("manifest_path", "relative/Cargo.toml"), ("version", "1.5.0"), ("source", "git+https://example.com")]:
            with self.subTest(field=field):
                metadata = copy.deepcopy(self.metadata)
                metadata["packages"][0][field] = value
                self.check_raises(metadata=metadata)


    def test_consumer_input_and_installed_manifest_include_guard_and_benchmark(self):
        root = SCRIPT.parents[1]
        declared = json.loads((root / "release/sc-publish-consumer-input.json").read_text())["preflight_checks"]
        installed = tomllib.loads((root / "release/publish-preflight.toml").read_text())["checks"]
        self.assertEqual(declared, installed)
        self.assertEqual([check["id"] for check in installed],
                         ["observability-registry-only", "benchmark-pass"])
        self.assertEqual(installed[0]["run"], "python3 scripts/check_observability_release.py")

    def check_raises(self, manifest=None, metadata=None):
        with self.assertRaises(ValueError):
            self.check(manifest, metadata)


if __name__ == "__main__":
    unittest.main()
