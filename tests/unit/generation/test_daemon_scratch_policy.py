"""Ancestry checks for the approved private Python scratch dependency."""

import copy
import importlib.util
import os
import sys
import tempfile
import tomllib
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tests/integration/generation"))
from owned_process import run
SPEC = importlib.util.spec_from_file_location(
    "dependency_policy", ROOT / "template/scripts/dependency_policy.py"
)
POLICY = importlib.util.module_from_spec(SPEC)
previous_bytecode_setting = sys.dont_write_bytecode
try:
    sys.dont_write_bytecode = True
    SPEC.loader.exec_module(POLICY)
finally:
    sys.dont_write_bytecode = previous_bytecode_setting
DAEMON_MANIFEST = Path("/fixture/crates/daemon/Cargo.toml")


def graph(edges):
    names = {"daemon": "example-daemon", "cron": "sc-cron", "old-cron": "sc-cron"}
    identities = {"daemon", *(parent for parent, _, _, _ in edges),
                  *(child for _, child, _, _ in edges)}
    packages = []
    nodes = []
    for identity in sorted(identities):
        dependencies = []
        resolved = []
        for parent, child, binding, kind in edges:
            if parent != identity:
                continue
            dependencies.append({"name": names.get(child, child), "kind": kind,
                                 "rename": binding if binding != names.get(child, child).replace("-", "_") else None})
            resolved.append({"name": binding, "pkg": child,
                             "dep_kinds": [{"kind": kind, "target": None}]})
        packages.append({"id": identity, "name": names.get(identity, identity),
                         "version": "0.1.0" if identity == "old-cron" else "0.2.0",
                         "manifest_path": str(DAEMON_MANIFEST) if identity == "daemon" else f"/fixture/{identity}/Cargo.toml",
                         "dependencies": dependencies})
        nodes.append({"id": identity, "deps": resolved})
    return {"packages": packages, "resolve": {"nodes": nodes}}


def normal_tree(metadata):
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    lines = []
    expanded = set()

    def visit(identity, depth):
        package = packages[identity]
        display = f"{package['name']} v{package['version']} ({Path(package['manifest_path']).parent})"
        lines.append(f"{depth}|{display}" + (" (*)" if identity in expanded else ""))
        if identity in expanded:
            return
        expanded.add(identity)
        for edge in nodes[identity]["deps"]:
            if any(kind["kind"] is None for kind in edge["dep_kinds"]):
                visit(edge["pkg"], depth + 1)

    visit("daemon", 0)
    return "\n".join(lines)


SDK = ("daemon", "cron", "sc_cron", None)
SCRATCH = ("cron", "tempfile", "tempfile", None)


def repeated_dependency_workspace(root):
    """Repeat a registry package using the repository's already-locked version."""
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    version = next(package["version"] for package in lock["package"]
                   if package["name"] == "tracing")
    (root / "Cargo.toml").write_text(
        '[workspace]\nmembers=["crates/daemon", "crates/helper"]\nresolver="2"\n'
        f'[workspace.dependencies]\ntracing="={version}"\n', encoding="utf-8"
    )
    for name, dependencies in (
        ("daemon", 'helper={path="../helper"}\ntracing.workspace=true\n'),
        ("helper", 'tracing.workspace=true\n'),
    ):
        crate = root / "crates" / name
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text(
            f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n'
            f'[dependencies]\n{dependencies}', encoding="utf-8"
        )
        (crate / "src/lib.rs").write_text("", encoding="utf-8")


class DaemonScratchPolicy(unittest.TestCase):
    def errors(self, edges):
        data = graph(edges)
        return POLICY.daemon_scratch_errors(data, DAEMON_MANIFEST, normal_tree(data))

    def test_machine_tree_ignores_forced_cargo_color(self):
        with tempfile.TemporaryDirectory(prefix="daemon-normal-color-") as temporary:
            root = Path(temporary)
            repeated_dependency_workspace(root)
            env = dict(os.environ, CARGO_TERM_COLOR="always", CARGO_NET_OFFLINE="true")
            run(
                ["cargo", "metadata", "--format-version", "1"], cwd=root,
                env=env, timeout=30, check=True, capture_output=True, text=True,
            )
            colored = run(
                ["cargo", "tree", "-p", "daemon", "-e", "normal", "--locked",
                 "--prefix", "depth", "--format", "|{p}"], cwd=root,
                env=env, timeout=30, check=True, capture_output=True, text=True,
            )
            self.assertIn("\x1b[", colored.stdout)
            result = run(
                [sys.executable, str(ROOT / "template/scripts/consumer_checks.py"),
                 "build-daemon"], cwd=root, env=env, timeout=30,
                capture_output=True, text=True,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_unselected_cron_without_temporary_packages_passes(self):
        self.assertEqual(self.errors([("daemon", "service", "service", None)]), [])

    def test_selected_sdk_direct_scratch_passes(self):
        self.assertEqual(self.errors([SDK, SCRATCH]), [])

    def test_direct_daemon_tempfile_fails(self):
        self.assertTrue(self.errors([("daemon", "tempfile", "tempfile", None)]))

    def test_same_tempfile_identity_with_second_direct_path_fails(self):
        self.assertTrue(self.errors([SDK, SCRATCH, ("daemon", "tempfile", "tempfile", None)]))

    def test_same_identity_with_other_transitive_path_fails(self):
        self.assertTrue(self.errors([SDK, SCRATCH, ("daemon", "other", "other", None),
                                     ("other", "tempfile", "tempfile", None)]))

    def test_cron_subtree_is_not_blanket_allowed(self):
        self.assertTrue(self.errors([SDK, ("cron", "other", "other", None),
                                     ("other", "tempfile", "tempfile", None)]))

    def test_alt_path_below_cron_fails_despite_qualified_direct_edge(self):
        self.assertTrue(self.errors([SDK, SCRATCH, ("cron", "other", "other", None),
                                     ("other", "tempfile", "tempfile", None)]))

    def test_unqualified_namesake_cannot_authorize_tempfile(self):
        self.assertTrue(self.errors([("daemon", "cron", "helper", None), SCRATCH]))

    def test_second_sc_cron_identity_is_not_the_qualified_sdk(self):
        self.assertTrue(self.errors([SDK, SCRATCH, ("daemon", "old-cron", "old_cron", None),
                                     ("old-cron", "tempfile", "tempfile", None)]))

    def test_tempdir_is_always_forbidden(self):
        for edges in ([ ("daemon", "tempdir", "tempdir", None) ],
                      [SDK, ("cron", "tempdir", "tempdir", None)]):
            with self.subTest(edges=edges):
                self.assertTrue(self.errors(edges))

    def test_dev_and_build_edges_do_not_enter_the_normal_graph(self):
        for kind in ("dev", "build"):
            with self.subTest(kind=kind):
                self.assertEqual(self.errors([("daemon", "tempfile", "tempfile", kind)]), [])

    def test_dev_sdk_edge_does_not_qualify_a_normal_alternate_path(self):
        self.assertTrue(self.errors([("daemon", "cron", "sc_cron", "dev"),
                                     ("daemon", "other", "other", None),
                                     ("other", "cron", "sc_cron", None), SCRATCH]))

    def test_build_edge_does_not_hide_a_parallel_normal_edge(self):
        data = graph([SDK, SCRATCH, ("daemon", "tempfile", "tempfile", "build")])
        node = next(node for node in data["resolve"]["nodes"] if node["id"] == "daemon")
        edge = next(edge for edge in node["deps"] if edge["pkg"] == "tempfile")
        edge["dep_kinds"].append({"kind": None, "target": None})
        self.assertTrue(POLICY.daemon_scratch_errors(data, DAEMON_MANIFEST, normal_tree(data)))

    def test_unreachable_other_workspace_tree_is_not_a_daemon_violation(self):
        self.assertEqual(self.errors([SDK, SCRATCH, ("other", "tempdir", "tempdir", None)]), [])

    def test_metadata_dev_feature_unification_is_not_the_normal_build_graph(self):
        data = graph([SDK, SCRATCH, ("daemon", "runtime", "runtime", None)])
        tree = normal_tree(data)
        extra = graph([("runtime", "tempfile", "tempfile", None)])
        node = next(node for node in data["resolve"]["nodes"] if node["id"] == "runtime")
        node["deps"] = next(node for node in extra["resolve"]["nodes"] if node["id"] == "runtime")["deps"]
        self.assertEqual(POLICY.daemon_scratch_errors(data, DAEMON_MANIFEST, tree), [])

    def test_ambiguous_resolved_identity_fails_closed(self):
        data = graph([("daemon", "first", "first", None), ("daemon", "second", "second", None)])
        for package in data["packages"]:
            if package["id"] in {"first", "second"}:
                package.update(name="same", version="1.0.0")
        with self.assertRaises(ValueError):
            POLICY.daemon_scratch_errors(data, DAEMON_MANIFEST,
                                        "0|example-daemon v0.2.0\n1|same v1.0.0")

    def test_missing_resolve_identity_fails_closed(self):
        data = graph([SDK, SCRATCH])
        tree = normal_tree(data)
        data["resolve"]["nodes"] = [node for node in data["resolve"]["nodes"] if node["id"] != "cron"]
        with self.assertRaises(KeyError):
            POLICY.daemon_scratch_errors(data, DAEMON_MANIFEST, tree)

    def test_ambiguous_daemon_manifest_fails_closed(self):
        data = graph([SDK, SCRATCH])
        extra = copy.deepcopy(data["packages"][0])
        extra.update(id="duplicate", manifest_path=str(DAEMON_MANIFEST))
        data["packages"].append(extra)
        with self.assertRaises(ValueError):
            POLICY.daemon_scratch_errors(data, DAEMON_MANIFEST, normal_tree(data))


if __name__ == "__main__":
    unittest.main()
