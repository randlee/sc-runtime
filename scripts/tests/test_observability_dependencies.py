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
    def test_actual_workspace_normal_graph_for_both_libraries(self):
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

    def test_authorized_types_facade_is_allowed(self):
        for owner in CHECK.OWNERS:
            CHECK.check_graph(graph(owner, "sc-observability-types", transitive=True), owner)

    def test_build_and_dev_only_edges_do_not_enter_normal_graph(self):
        for kind in ("dev", "build"):
            for owner in CHECK.OWNERS:
                CHECK.check_graph(graph(owner, "sc-observability", True, kind), owner)

    def test_mixed_edge_still_rejects_normal_reachability(self):
        metadata = graph("sc-runtime", "sc-observability")
        metadata["resolve"]["nodes"][0]["deps"][0]["dep_kinds"].append(
            {"kind": "dev", "target": None},
        )
        with self.assertRaisesRegex(ValueError, "sc-observability"):
            CHECK.check_graph(metadata, "sc-runtime")

    def test_missing_root_fails_closed(self):
        with self.assertRaisesRegex(ValueError, "expected one package identity"):
            CHECK.check_graph(graph("other", "sc-observability-types"), "sc-runtime")


if __name__ == "__main__":
    unittest.main()
