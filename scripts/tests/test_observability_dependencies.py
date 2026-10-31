"""Shared types-only dependency check: actual workspace graph and negative fixtures."""

import importlib.util
from pathlib import Path
import unittest

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
        edge = {"version": "0.1.1", "path": "../sc-command", "default-features": False}
        good = {"dependencies": {"sc-command": edge}}
        CHECK.check_cron_manifest(good)
        variants = [
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

    def test_only_checkout_dev_self_may_omit_path_version(self):
        import copy
        good = {"dependencies": {"sc-command": {"version": "0.1.1", "path": "../sc-command", "default-features": False}},
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

    def test_missing_root_fails_closed(self):
        with self.assertRaisesRegex(ValueError, "expected one package identity"):
            CHECK.check_graph(graph("other", "sc-observability-types"), "sc-runtime")


if __name__ == "__main__":
    unittest.main()
