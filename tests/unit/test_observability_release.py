"""Release guard rejects local substitutions and incomplete Cargo evidence."""

import copy
import importlib.util
from pathlib import Path
import tempfile
import json
import tomllib
import unittest
import subprocess
from unittest import mock
import urllib.error
import io

SCRIPT = Path(__file__).resolve().parents[2] / "scripts/check_observability_release.py"
spec = importlib.util.spec_from_file_location("observability_guard", SCRIPT)
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)


def registry_metadata():
    return {
        "packages": [{"name": guard.NAME, "version": "1.5.0", "id": "registry-id",
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

    def test_replace_and_git_overrides_rejected(self):
        self.check_raises({"replace": {"sc-observability-types:1.5.0": {"git": "https://example.com/types"}}})
        self.check_raises({"dependencies": {"alias": {"package": guard.NAME, "git": "https://example.com/types"}}})
        self.check_raises({"patch": {"crates-io": {"sc-observability": "1.5.0"}}})

    def test_sdk_path_is_allowed(self):
        sdk = self.root / "sc-command"
        sdk.mkdir()
        (sdk / "Cargo.toml").write_text('[package]\nname = "sc-command"\nversion = "0.1.1"\n')
        self.check({"dependencies": {"sc-command": {"version": "0.1.1", "path": "sc-command"}}})

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

    def test_multiple_types_identities_rejected(self):
        metadata = copy.deepcopy(self.metadata)
        second = copy.deepcopy(metadata["packages"][0])
        second["id"] = "another-id"
        metadata["packages"].append(second)
        metadata["resolve"]["nodes"].append({"id": "another-id"})
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
        for field, value in [("manifest_path", "relative/Cargo.toml"), ("version", "1.5.1"), ("source", "git+https://example.com")]:
            with self.subTest(field=field):
                metadata = copy.deepcopy(self.metadata)
                metadata["packages"][0][field] = value
                self.check_raises(metadata=metadata)


    def test_probe_timeout_uses_validated_tree_owner(self):
        with mock.patch.object(guard._owned, "run", side_effect=subprocess.TimeoutExpired("cargo", 1)) as run:
            with self.assertRaisesRegex(ValueError, "upstream.*timed out.*terminated and reaped"):
                guard.run_probe_cargo(["cargo"], self.root, {}, "upstream", timeout=1)
            self.assertEqual(run.call_args.kwargs["timeout"], 1)
            self.assertEqual(run.call_args.kwargs["env"], {})

    def test_registry_absence_partial_and_probe_outcomes(self):
        checksum = "a" * 64
        with mock.patch.object(guard, "registry_version", side_effect=[None, None]), \
             mock.patch.object(guard, "probe_upstream") as probe:
            guard.check_upstream_readiness()
            probe.assert_not_called()
        with mock.patch.object(guard, "registry_version", side_effect=[None, checksum]):
            with self.assertRaisesRegex(ValueError, "partial"):
                guard.check_upstream_readiness()
        with mock.patch.object(guard, "registry_version", side_effect=[checksum, checksum]), \
             mock.patch.object(guard, "probe_upstream", side_effect=ValueError("compile failed")):
            with self.assertRaisesRegex(ValueError, "incompatible.*compile failed"):
                guard.check_upstream_readiness()
        with mock.patch.object(guard, "registry_version", side_effect=[checksum, checksum]), \
             mock.patch.object(guard, "probe_upstream"):
            with self.assertRaisesRegex(ValueError, "cleanup.*issues/97"):
                guard.check_upstream_readiness()

    def test_registry_http_and_metadata_fail_closed(self):
        url = "https://crates.io/api/v1/crates/sc-observability-types/1.5.1"
        class Response:
            def __init__(self, data, source=url):
                self.data, self.source = data, source
            def __enter__(self): return self
            def __exit__(self, *args): pass
            def geturl(self): return self.source
            def read(self, limit): return json.dumps(self.data).encode()
        good = {"version": {"crate": guard.NAME, "num": "1.5.1", "yanked": False, "checksum": "a" * 64}}
        with mock.patch.object(guard.urllib.request, "urlopen", return_value=Response(good)) as fetch:
            self.assertEqual(guard.registry_version(guard.NAME), "a" * 64)
            request = fetch.call_args.args[0]
            self.assertEqual(request.get_header("Cache-control"), "no-cache, no-store")
            self.assertEqual(fetch.call_args.kwargs["timeout"], 15)
        for data, source in (({}, url), ({"version": {**good["version"], "yanked": True}}, url),
                             ({"version": {**good["version"], "checksum": "bad"}}, url),
                             (good, "https://evil.example/x")):
            with self.subTest(data=data, source=source), mock.patch.object(guard.urllib.request, "urlopen", return_value=Response(data, source)):
                with self.assertRaises(ValueError): guard.registry_version(guard.NAME)
        for code in (404, 500):
            error = urllib.error.HTTPError(url, code, "error", {}, io.BytesIO())
            with mock.patch.object(guard.urllib.request, "urlopen", side_effect=error):
                if code == 404: self.assertIsNone(guard.registry_version(guard.NAME))
                else:
                    with self.assertRaises(ValueError): guard.registry_version(guard.NAME)
        with mock.patch.object(guard.urllib.request, "urlopen", side_effect=urllib.error.URLError("offline")):
            with self.assertRaises(ValueError): guard.registry_version(guard.NAME)

    def test_probe_builds_exact_isolated_workspace_and_checks_metadata(self):
        checksums = {name: "a" * 64 for name in guard.UPSTREAM_NAMES}
        packages = [{"name": name, "version": "1.5.1", "source": guard.REGISTRY, "id": name} for name in guard.UPSTREAM_NAMES]
        metadata = {"packages": packages, "resolve": {"nodes": [{"id": name} for name in guard.UPSTREAM_NAMES]}}
        def run(args, root, env, context):
            manifest = tomllib.loads((root / "Cargo.toml").read_text())
            self.assertEqual(manifest["dependencies"], {name: "=1.5.1" for name in guard.UPSTREAM_NAMES})
            self.assertIn("workspace", manifest)
            self.assertEqual(env["CARGO_HOME"], str(root / "cargo-home"))
            source = (root / "src/lib.rs").read_text()
            self.assertIn("Arc<sc_observability::v2::Logger>", source)
            self.assertIn("-> SharedCanonicalLog", source)
            if args[1] == "metadata":
                lock = "".join('[[package]]\nname = "' + name + '"\nversion = "1.5.1"\nsource = "' + guard.REGISTRY + '"\nchecksum = "' + checksums[name] + '"\n' for name in guard.UPSTREAM_NAMES)
                (root / "Cargo.lock").write_text(lock)
                return json.dumps(metadata)
            self.assertEqual(args, ["cargo", "check", "--locked"])
            return ""
        with mock.patch.object(guard, "run_probe_cargo", side_effect=run) as cargo:
            guard.probe_upstream(checksums)
            self.assertEqual(cargo.call_count, 2)

    def test_probe_metadata_and_checksum(self):
        checksums = {name: "a" * 64 for name in guard.UPSTREAM_NAMES}
        packages = [{"name": name, "version": "1.5.1", "source": guard.REGISTRY, "id": name} for name in guard.UPSTREAM_NAMES]
        metadata = {"packages": packages, "resolve": {"nodes": [{"id": name} for name in guard.UPSTREAM_NAMES]}}
        lock = {"package": [{"name": name, "version": "1.5.1", "source": guard.REGISTRY, "checksum": checksums[name]} for name in guard.UPSTREAM_NAMES]}
        guard.validate_probe(metadata, lock, checksums)
        for change in ("source", "version", "checksum", "duplicate"):
            bad = copy.deepcopy(lock)
            if change == "duplicate": bad["package"].append(copy.deepcopy(bad["package"][0]))
            else: bad["package"][0][change] = "invalid"
            with self.subTest(change=change), self.assertRaises(ValueError):
                guard.validate_probe(metadata, bad, checksums)
        bad_metadata = copy.deepcopy(metadata)
        bad_metadata["packages"][0]["source"] = None
        with self.assertRaises(ValueError): guard.validate_probe(bad_metadata, lock, checksums)

    def test_consumer_input_and_installed_manifest_include_guard_and_benchmark(self):
        root = SCRIPT.parents[1]
        declared = json.loads((root / "release/sc-publish-consumer-input.json").read_text())["preflight_checks"]
        installed = tomllib.loads((root / "release/publish-preflight.toml").read_text())["checks"]
        self.assertEqual(declared, installed)
        self.assertEqual(
            [check["id"] for check in installed],
            ["observability-registry-only", "release-contract-sources", "benchmark-pass"],
        )
        self.assertEqual(installed[0]["run"], "python3 scripts/check_observability_release.py")

    def check_raises(self, manifest=None, metadata=None):
        with self.assertRaises(ValueError):
            self.check(manifest, metadata)


if __name__ == "__main__":
    unittest.main()
