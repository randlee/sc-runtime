#!/usr/bin/env python3
"""Enforce the public-SDK observability boundary on resolved normal and build package identities."""

import json
import importlib.util
import os
from pathlib import Path
import subprocess
import tomllib
import tempfile

ROOT = Path(__file__).resolve().parent.parent
# Repo-only preflight reuses the validated session/Windows Job owner. Resolve
# from this file, never the caller's cwd; marker state is set only in children.
_owned_spec = importlib.util.spec_from_file_location(
    "sc_runtime_owned_commands", ROOT / "tests/integration/generation/owned_process.py")
_owned = importlib.util.module_from_spec(_owned_spec)
_owned_spec.loader.exec_module(_owned)
OWNERS = ("sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron", "sc-cron-commands")
NO_TYPES = frozenset({"sc-config", "sc-transport"})
FORBIDDEN = frozenset({
    "sc-observability", "sc-observability-otlp", "sc-observability-dev-adapter",
})


def check_graph(metadata, owner):
    """Reject forbidden packages reachable through any resolved normal or build edge.

    Edge aliases never determine identity: deps[].pkg selects packages[].name.
    Target-specific edges are included; dev-only edges are excluded.
    """
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    pending = [pid for pid, package in packages.items() if package["name"] == owner]
    if len(pending) != 1:
        raise ValueError(f"expected one package identity for {owner}")
    visited = set()
    while pending:
        package_id = pending.pop()
        if package_id in visited:
            continue
        visited.add(package_id)
        package = packages[package_id]
        if package["name"] in FORBIDDEN or (owner in NO_TYPES and package["name"] == "sc-observability-types"):
            raise ValueError(f"{owner} normal/build dependency reaches {package['name']} ({package_id})")
        for edge in nodes[package_id]["deps"]:
            if any(kind["kind"] in (None, "build") for kind in edge["dep_kinds"]):
                pending.append(edge["pkg"])

    return visited


def check_command_manifest(manifest):
    if manifest.get("dependencies", {}).get("sc-observability-types") != "=1.5.0":
        raise ValueError("command must pin registry sc-observability-types to =1.5.0")


def check_cron_manifest(manifest):
    """Enforce the single, defaults-off temporary command edge."""
    expected = {"version": "0.2.0", "path": "../sc-command", "default-features": False}
    deps = manifest.get("dependencies", {})
    if deps.get("sc-command") != expected:
        raise ValueError("cron requires the exact defaults-off sc-command normal dependency")
    def reject_extra(entries):
        for alias, spec in entries.items():
            name = spec.get("package", alias) if isinstance(spec, dict) else alias
            if name == "sc-command" or alias == "sc-command":
                raise ValueError("extra cron sc-command normal/build edge")
    reject_extra({k: v for k, v in deps.items() if k != "sc-command"})
    reject_extra(manifest.get("build-dependencies", {}))
    for tables in manifest.get("target", {}).values():
        for kind in ("dependencies", "build-dependencies"):
            reject_extra(tables.get(kind, {}))
    # SCB-MANIFEST-002 cannot represent Cargo's versionless checkout-only
    # dev self edge. Its one manifest/rule exclusion is replaced here with
    # exact equivalent alignment checks for every other path dependency.
    def check_path_versions(entries, global_dev=False):
        for alias, spec in entries.items():
            if not isinstance(spec, dict) or "path" not in spec:
                continue
            if spec.get("path") == "." or spec.get("package", alias) == "sc-cron":
                if global_dev and alias == "sc-cron" and spec == {"path": ".", "features": ["test-utils"]}:
                    continue
                raise ValueError("cron workspace self edge must be exactly checkout-only dev test-utils")
            expected_version = "0.2.0"
            if spec.get("version") != expected_version:
                raise ValueError(f"cron path dependency {alias} must match frozen package version {expected_version}")
    for kind in ("dependencies", "build-dependencies", "dev-dependencies"):
        check_path_versions(manifest.get(kind, {}), global_dev=kind == "dev-dependencies")
    for tables in manifest.get("target", {}).values():
        for kind in ("dependencies", "build-dependencies", "dev-dependencies"):
            check_path_versions(tables.get(kind, {}))
    for feature, entries in manifest.get("features", {}).items():
        if any(entry.startswith(("sc-command/", "sc-command?/", "dep:sc-command")) for entry in entries):
            raise ValueError(f"cron feature {feature} activates sc-command")


def run_metadata(command, cwd, context, timeout=120):
    """Bound Cargo and all owned descendants using the shared validated helper."""
    try:
        result = _owned.run(command, cwd=cwd, timeout=timeout, stdin=subprocess.DEVNULL,
                            capture_output=True, text=True)
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f"{context}: Cargo metadata timed out after {timeout}s; owned tree terminated and reaped") from exc
    except (OSError, RuntimeError, TimeoutError, subprocess.SubprocessError) as exc:
        raise RuntimeError(f"{context}: owned Cargo metadata execution or cleanup failed: {exc}") from exc
    if result.returncode:
        raise RuntimeError(f"{context}: Cargo metadata failed ({result.returncode}): {result.stderr.strip()}")
    return json.loads(result.stdout)


def resolved_metadata():
    # All supported features include cron's optional Monty production graph.
    # Cargo metadata resolves packages without compiling the higher-MSRV feature.
    return run_metadata(["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
                        ROOT, "workspace observability graph")


def check_isolated_features():
    """Resolve cron and command adapter without workspace feature unification."""
    probes = (
        ("sc-cron", ((), ("monty",))),
        ("sc-cron-commands", ((), ("schema",), ("scheduler",),
                              ("schema", "scheduler"), ("monty",), ("schema", "monty"))),
    )
    with tempfile.TemporaryDirectory(prefix="sc-runtime-cron-boundary-") as directory:
        root = Path(directory)
        (root / "src").mkdir()
        (root / "src/lib.rs").write_text("")
        for owner, configurations in probes:
            for features in configurations:
                (root / "Cargo.toml").write_text(
                    '[package]\nname = "cron-boundary-probe"\nversion = "0.0.0"\nedition = "2024"\n'
                    f'[workspace]\n[dependencies.{owner}]\n'
                    f'path = {json.dumps(str(ROOT / "crates" / owner))}\n'
                    f'default-features = false\nfeatures = {json.dumps(features)}\n'
                )
                label = "Monty" if features == ("monty",) else ",".join(features) or "default"
                context = f"isolated {owner} {label} graph"
                metadata = run_metadata(["cargo", "metadata", "--offline", "--format-version", "1"], root, context)
                check_isolated_graph(metadata, owner, features)


def check_isolated_graph(metadata, owner="sc-cron", features=()):
    visited = check_graph(metadata, owner)
    packages = {package["id"]: package for package in metadata["packages"]}
    adapter = owner == "sc-cron-commands"
    scheduler = "scheduler" in features or "monty" in features
    for node in metadata["resolve"]["nodes"]:
        if node["id"] not in visited:
            continue
        name = packages[node["id"]]["name"]
        forbidden = name in {"axum", "rmcp"} or (adapter and name == "sc-runtime")
        if adapter and not scheduler:
            forbidden |= name in {"sc-cron", "rusqlite", "tokio", "monty"}
        if forbidden:
            raise ValueError(f"isolated {owner} unexpectedly links {name}")
        if name == "sc-command" and {"server", "test-util"}.intersection(node.get("features", [])):
            raise ValueError(f"isolated {owner} activates command server/test-util")


def main():
    check_command_manifest(tomllib.loads((ROOT / "crates/sc-command/Cargo.toml").read_text()))
    check_cron_manifest(tomllib.loads((ROOT / "crates/sc-cron/Cargo.toml").read_text()))
    metadata = resolved_metadata()
    for owner in OWNERS:
        check_graph(metadata, owner)
    check_isolated_features()
    print("observability normal/build dependency boundary: PASS (six SDKs)")


if __name__ == "__main__":
    main()
