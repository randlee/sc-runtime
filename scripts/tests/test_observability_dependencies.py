"""Shared types-only dependency check: actual workspace graph and negative fixtures."""

import importlib.util
from pathlib import Path
import unittest
from unittest import mock
import subprocess
import json
import os
import shutil
import tempfile

SPEC = importlib.util.spec_from_file_location(
    "observability_dependencies", Path(__file__).resolve().parents[1] / "check_observability_dependencies.py",
)
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


def graph(owner, package, transitive=False, kind=None):
    def edge(package_id):
        return {"name": "innocent_alias", "pkg": package_id,
                "dep_kinds": [{"kind": kind, "target": "cfg(unix)"}]}
    return {
        "packages": [{"id": "owner", "name": owner}, {"id": "bridge", "name": "bridge"},
                     {"id": "hidden", "name": package}],
        "resolve": {"nodes": [
            {"id": "owner", "deps": [edge("bridge" if transitive else "hidden")]},
            {"id": "bridge", "deps": [edge("hidden")]}, {"id": "hidden", "deps": []},
        ]},
    }


class ObservabilityDependencies(unittest.TestCase):
    def test_isolated_cron_logging_consumer(self):
        fixture = CHECK.ROOT / "crates/sc-cron/tests/fixtures/logging_consumer"
        with tempfile.TemporaryDirectory(prefix="cron-logging-consumer-") as directory:
            root = Path(directory)
            shutil.copytree(fixture / "tests", root / "tests")
            manifest = (fixture / "Cargo.toml").read_text().replace(
                '"../../.."', json.dumps(str(CHECK.ROOT / "crates/sc-cron")))
            (root / "Cargo.toml").write_text(manifest)
            metadata = CHECK.run_metadata(
                ["cargo", "metadata", "--offline", "--format-version", "1"],
                root, "isolated logging consumer")
            CHECK.check_isolated_graph(metadata)
            consumer = next(p for p in metadata["packages"] if p["name"] == "cron-logging-consumer")
            self.assertNotIn("sc-command", [d["name"] for d in consumer["dependencies"]])
            env = os.environ.copy()
            env.setdefault("CARGO_TARGET_DIR", str(CHECK.ROOT / "target/cron-logging-consumer"))
            result = CHECK._owned.run(
                ["cargo", "test", "--offline"], cwd=root, env=env, timeout=300,
                stdin=subprocess.DEVNULL, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_actual_workspace_normal_and_build_graph_for_all_libraries(self):
        metadata = CHECK.resolved_metadata()
        for owner in CHECK.OWNERS:
            with self.subTest(owner=owner):
                CHECK.check_graph(metadata, owner)

    def test_rejects_actual_package_identity_through_alias_and_transitive_edges(self):
        for owner in CHECK.OWNERS:
            for forbidden in CHECK.FORBIDDEN:
                for transitive in (False, True):
                    with self.subTest(owner=owner, package=forbidden, transitive=transitive):
                        with self.assertRaisesRegex(ValueError, forbidden):
                            CHECK.check_graph(graph(owner, forbidden, transitive), owner)

    def test_types_allowed_only_for_contract_sdks(self):
        for owner in CHECK.OWNERS:
            if owner in CHECK.NO_TYPES:
                with self.assertRaisesRegex(ValueError, "sc-observability-types"):
                    CHECK.check_graph(graph(owner, "sc-observability-types", transitive=True), owner)
            else:
                CHECK.check_graph(graph(owner, "sc-observability-types", transitive=True), owner)

    def test_build_edges_rejected_and_dev_edges_excluded(self):
        for owner in CHECK.OWNERS:
            for forbidden in CHECK.FORBIDDEN:
                with self.subTest(owner=owner, forbidden=forbidden):
                    with self.assertRaisesRegex(ValueError, forbidden):
                        CHECK.check_graph(graph(owner, forbidden, True, "build"), owner)
                    CHECK.check_graph(graph(owner, forbidden, True, "dev"), owner)

    def test_command_pin_is_exact(self):
        CHECK.check_command_manifest({"dependencies": {"sc-observability-types": "=1.5.0"}})
        for spec in ("1.5.0", "=1.5.1", {"version": "=1.5.0", "path": "local"}):
            with self.subTest(spec=spec), self.assertRaises(ValueError):
                CHECK.check_command_manifest({"dependencies": {"sc-observability-types": spec}})

    def test_cron_contract_edge_is_exact(self):
        import copy
        edge = {"version": "0.2.0", "path": "../sc-command", "default-features": False}
        good = {"dependencies": {"sc-command": edge}}
        CHECK.check_cron_manifest(good)
        variants = [
            {"dependencies": {"sc-command": {**edge, "version": "0.1.1"}}},
            {"dependencies": {"sc-command": {**edge, "features": ["server"]}}},
            {"dependencies": {"sc-command": {**edge, "default-features": True}}},
            {"dependencies": {"alias": {**edge, "package": "sc-command"}}},
            {**copy.deepcopy(good), "build-dependencies": {"alias": {"package": "sc-command"}}},
            {**copy.deepcopy(good), "target": {"cfg(unix)": {"dependencies": {"alias": {"package": "sc-command"}}}}},
            {**copy.deepcopy(good), "features": {"bad": ["sc-command/server"]}},
        ]
        for variant in variants:
            with self.subTest(variant=variant), self.assertRaises(ValueError):
                CHECK.check_cron_manifest(variant)

    def test_cron_dev_command_follows_affected_package_version(self):
        edge = {"version": "0.2.0", "path": "../sc-command", "default-features": False}
        good = {"dependencies": {"sc-command": edge},
                "dev-dependencies": {"sc-command": {**edge, "features": ["test-util"]}}}
        CHECK.check_cron_manifest(good)
        good["dev-dependencies"]["sc-command"]["version"] = "0.1.1"
        with self.assertRaisesRegex(ValueError, "frozen package version"):
            CHECK.check_cron_manifest(good)

    def test_only_checkout_dev_self_may_omit_path_version(self):
        import copy
        good = {"dependencies": {"sc-command": {"version": "0.2.0", "path": "../sc-command", "default-features": False}},
                "dev-dependencies": {"sc-cron": {"path": ".", "features": ["test-utils"]}}}
        CHECK.check_cron_manifest(good)
        for spec in ({"path": "."}, {"path": ".", "features": ["test-utils", "monty"]},
                     {"path": ".", "features": ["test-utils"], "version": "0.1.0"},
                     {"path": ".", "features": ["test-utils"], "version": "0.1.1"}):
            bad = copy.deepcopy(good)
            bad["dev-dependencies"]["sc-cron"] = spec
            with self.subTest(spec=spec), self.assertRaisesRegex(ValueError, "workspace"):
                CHECK.check_cron_manifest(bad)
        bad = copy.deepcopy(good)
        bad["target"] = {"cfg(unix)": {"dev-dependencies": {"sc-cron": {"path": ".", "features": ["test-utils"]}}}}
        with self.assertRaisesRegex(ValueError, "workspace"):
            CHECK.check_cron_manifest(bad)

    def test_mixed_edge_still_rejects_normal_reachability(self):
        metadata = graph("sc-runtime", "sc-observability")
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"].append(
            {"kind": "dev", "target": None},
        )
        with self.assertRaisesRegex(ValueError, "sc-observability"):
            CHECK.check_graph(metadata, "sc-runtime")

    def test_isolated_cron_rejects_server_feature_and_server_packages(self):
        metadata = graph("sc-cron", "sc-command")
        CHECK.check_isolated_graph(metadata)
        for feature in ("server", "test-util"):
            metadata["resolve"]["nodes"][2]["features"] = [feature]
            with self.subTest(feature=feature), self.assertRaisesRegex(ValueError, "activates"):
                CHECK.check_isolated_graph(metadata)
        for name in ("axum", "rmcp"):
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, name):
                CHECK.check_isolated_graph(graph("sc-cron", name, transitive=True))

    def test_adapter_is_enforced_by_main_owner_loop(self):
        for kind in (None, "build", "dev"):
            metadata = graph("sc-cron-commands", "sc-observability-otlp", True, kind)
            for owner in CHECK.OWNERS:
                if owner != "sc-cron-commands":
                    metadata["packages"].append({"id": owner, "name": owner})
                    metadata["resolve"]["nodes"].append({"id": owner, "deps": []})
            with self.subTest(kind=kind), mock.patch.object(CHECK, "resolved_metadata", return_value=metadata), \
                 mock.patch.object(CHECK, "check_isolated_features"), mock.patch("builtins.print"):
                if kind == "dev":
                    CHECK.main()
                else:
                    with self.assertRaisesRegex(ValueError, "sc-cron-commands.*sc-observability-otlp"):
                        CHECK.main()

    def test_isolated_adapter_preserves_optional_scheduler_and_lean_defaults(self):
        owner = "sc-cron-commands"
        for name in ("sc-cron", "rusqlite", "tokio", "monty"):
            for features in ((), ("schema",)):
                with self.subTest(name=name, features=features), self.assertRaisesRegex(ValueError, name):
                    CHECK.check_isolated_graph(graph(owner, name, True), owner, features)
            CHECK.check_isolated_graph(graph(owner, name, True), owner, ("scheduler",))
        for features in ((), ("scheduler",), ("schema", "monty")):
            for name in ("axum", "rmcp", "sc-runtime"):
                with self.subTest(name=name, features=features), self.assertRaisesRegex(ValueError, name):
                    CHECK.check_isolated_graph(graph(owner, name, True), owner, features)
            metadata = graph(owner, "sc-command")
            metadata["resolve"]["nodes"][2]["features"] = ["server"]
            with self.assertRaisesRegex(ValueError, "activates command"):
                CHECK.check_isolated_graph(metadata, owner, features)

    def test_isolated_feature_runner_covers_adapter_configurations(self):
        seen = []
        def resolve(_command, root, _context):
            import tomllib
            manifest = tomllib.loads((root / "Cargo.toml").read_text())
            owner, spec = next(iter(manifest["dependencies"].items()))
            seen.append((owner, tuple(spec["features"])))
            return graph(owner, "sc-command")
        with mock.patch.object(CHECK, "run_metadata", side_effect=resolve):
            CHECK.check_isolated_features()
        adapter = {features for owner, features in seen if owner == "sc-cron-commands"}
        self.assertEqual(adapter, {(), ("schema",), ("scheduler",), ("schema", "scheduler"),
                                   ("monty",), ("schema", "monty")})
        self.assertIn(("sc-cron", ()), seen)
        self.assertIn(("sc-cron", ("monty",)), seen)

    def test_metadata_timeout_uses_validated_tree_owner(self):
        with mock.patch.object(CHECK._owned, "run", side_effect=subprocess.TimeoutExpired("cargo", 1)) as run:
            with self.assertRaisesRegex(RuntimeError, "workspace.*timed out.*terminated and reaped"):
                CHECK.run_metadata(["cargo"], ".", "workspace", timeout=1)
            self.assertEqual(run.call_args.kwargs["timeout"], 1)
            self.assertEqual(run.call_args.kwargs["stdin"], subprocess.DEVNULL)

    def test_default_and_monty_timeouts_report_probe(self):
        for fail_on, label in ((1, "default"), (2, "Monty")):
            def run(*args):
                run.calls += 1
                if run.calls == fail_on:
                    raise RuntimeError(args[2] + ": timed out")
                return graph("sc-cron", "sc-command")
            run.calls = 0
            with self.subTest(label=label), mock.patch.object(CHECK, "run_metadata", side_effect=run):
                with self.assertRaisesRegex(RuntimeError, label + ".*timed out"):
                    CHECK.check_isolated_features()

    def test_missing_root_fails_closed(self):
        with self.assertRaisesRegex(ValueError, "expected one package identity"):
            CHECK.check_graph(graph("other", "sc-observability-types"), "sc-runtime")


if __name__ == "__main__":
    unittest.main()
