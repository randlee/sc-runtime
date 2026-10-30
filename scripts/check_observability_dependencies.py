#!/usr/bin/env python3
"""Enforce the runtime/cron types-only boundary on resolved normal package identities."""

import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
OWNERS = ("sc-runtime", "sc-cron")
FORBIDDEN = frozenset({
    "sc-observability", "sc-observability-otlp", "sc-observability-dev-adapter",
})


def check_graph(metadata, owner):
    """Reject forbidden packages reachable through any resolved normal edge.

    Edge aliases never determine identity: deps[].pkg selects packages[].name.
    Target-specific normal edges are included; dev/build-only edges are excluded.
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
        if package["name"] in FORBIDDEN:
            raise ValueError(f"{owner} normal dependency reaches {package['name']} ({package_id})")
        for edge in nodes[package_id]["deps"]:
            if any(kind["kind"] is None for kind in edge["dep_kinds"]):
                pending.append(edge["pkg"])


def resolved_metadata():
    # All supported features include cron's optional Monty production graph.
    # Cargo metadata resolves packages without compiling the higher-MSRV feature.
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    )
    return json.loads(result.stdout)


def main():
    metadata = resolved_metadata()
    for owner in OWNERS:
        check_graph(metadata, owner)
    print("observability normal-dependency boundary: PASS (sc-runtime, sc-cron)")


if __name__ == "__main__":
    main()
