#!/usr/bin/env python3
"""Enforce the five-SDK observability boundary on resolved normal and build package identities."""

import json
from pathlib import Path
import subprocess
import tomllib
import tempfile

ROOT = Path(__file__).resolve().parent.parent
OWNERS = ("sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron")
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
    expected = {"version": "0.1.1", "path": "../sc-command", "default-features": False}
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
    for feature, entries in manifest.get("features", {}).items():
        if any(entry.startswith(("sc-command/", "sc-command?/", "dep:sc-command")) for entry in entries):
            raise ValueError(f"cron feature {feature} activates sc-command")


def resolved_metadata():
    # All supported features include cron's optional Monty production graph.
    # Cargo metadata resolves packages without compiling the higher-MSRV feature.
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    return json.loads(result.stdout)


def check_isolated_features():
    """Resolve cron outside the workspace so runtime cannot unify server features."""
    with tempfile.TemporaryDirectory(prefix="sc-runtime-cron-boundary-") as directory:
        root = Path(directory)
        (root / "src").mkdir()
        (root / "src/lib.rs").write_text("")
        for features in ([], ["monty"]):
            (root / "Cargo.toml").write_text(
                '[package]\nname = "cron-boundary-probe"\nversion = "0.0.0"\nedition = "2024"\n'
                '[workspace]\n[dependencies.sc-cron]\n'
                f'path = {json.dumps(str(ROOT / "crates/sc-cron"))}\n'
                f'default-features = false\nfeatures = {json.dumps(features)}\n'
            )
            result = subprocess.run(
                ["cargo", "metadata", "--offline", "--format-version", "1"],
                cwd=root, check=True, capture_output=True, text=True,
            )
            check_isolated_graph(json.loads(result.stdout))


def check_isolated_graph(metadata):
    visited = check_graph(metadata, "sc-cron")
    packages = {package["id"]: package for package in metadata["packages"]}
    for node in metadata["resolve"]["nodes"]:
        if node["id"] not in visited:
            continue
        name = packages[node["id"]]["name"]
        if name in {"axum", "rmcp"}:
            raise ValueError(f"isolated cron unexpectedly links {name}")
        if name == "sc-command" and {"server", "test-util"}.intersection(node.get("features", [])):
            raise ValueError("isolated cron activates command server/test-util")


def main():
    check_command_manifest(tomllib.loads((ROOT / "crates/sc-command/Cargo.toml").read_text()))
    check_cron_manifest(tomllib.loads((ROOT / "crates/sc-cron/Cargo.toml").read_text()))
    metadata = resolved_metadata()
    for owner in OWNERS:
        check_graph(metadata, owner)
    check_isolated_features()
    print("observability normal/build dependency boundary: PASS (five SDKs)")


if __name__ == "__main__":
    main()
