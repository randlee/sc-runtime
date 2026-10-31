#!/usr/bin/env python3
"""Reject development observability patches before release publication."""

import json
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
NAME = "sc-observability-types"
DEV_ADAPTER = "sc-observability-dev-adapter"
LOCAL_FORBIDDEN = {NAME, "sc-observability", DEV_ADAPTER}
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"


def validate_manifest(manifest, root):
    """Reject local observability dependencies and active observability patches."""
    root = Path(root).resolve()
    if not isinstance(manifest, dict):
        raise ValueError("manifest must be an object")

    def check_entries(entries, label, patch=False):
        if not isinstance(entries, dict):
            raise ValueError(f"invalid {label} table")
        for alias, spec in entries.items():
            if not isinstance(spec, (str, dict)):
                raise ValueError(f"invalid {label} entry: {alias}")
            package = spec.get("package", alias) if isinstance(spec, dict) else alias
            if not isinstance(package, str):
                raise ValueError(f"invalid {label} package: {alias}")
            is_target = alias in LOCAL_FORBIDDEN or package in LOCAL_FORBIDDEN
            if patch and is_target:
                raise ValueError(f"active observability patch: {alias}")
            if isinstance(spec, dict) and is_target and "git" in spec:
                raise ValueError(f"git observability dependency: {label}.{alias}")
            if isinstance(spec, dict) and "path" in spec:
                path = spec["path"]
                if not isinstance(path, str) or not path:
                    raise ValueError(f"invalid {label} path: {alias}")
                if is_target:
                    raise ValueError(f"local observability dependency: {label}.{alias}")
                path_manifest = (root / path / "Cargo.toml").resolve()
                try:
                    path_data = tomllib.loads(path_manifest.read_text(encoding="utf-8"))
                    path_name = path_data["package"]["name"]
                except (OSError, KeyError, TypeError, tomllib.TOMLDecodeError) as exc:
                    raise ValueError(f"cannot identify {label} path {alias}: {exc}") from exc
                if path_name in LOCAL_FORBIDDEN:
                    raise ValueError(f"local observability dependency: {path_manifest}")

    patches = manifest.get("patch", {})
    if not isinstance(patches, dict):
        raise ValueError("invalid patch table")
    for registry, entries in patches.items():
        check_entries(entries, f"patch.{registry}", patch=True)
    replacements = manifest.get("replace", {})
    if not isinstance(replacements, dict):
        raise ValueError("invalid replace table")
    for identity, spec in replacements.items():
        package = identity.rsplit(":", 1)[0]
        if package in LOCAL_FORBIDDEN:
            raise ValueError(f"active observability replacement: {identity}")
        check_entries({identity: spec}, "replace", patch=True)
    workspace = manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        raise ValueError("invalid workspace table")
    check_entries(workspace.get("dependencies", {}), "workspace.dependencies")
    for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
        check_entries(manifest.get(kind, {}), kind)
    targets = manifest.get("target", {})
    if not isinstance(targets, dict):
        raise ValueError("invalid target table")
    for target, tables in targets.items():
        if not isinstance(tables, dict):
            raise ValueError(f"invalid target {target}")
        for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
            check_entries(tables.get(kind, {}), f"target.{target}.{kind}")


def validate(manifest, metadata, root):
    """Return only if the root patch and resolved package are release-safe."""
    validate_manifest(manifest, root)
    root = Path(root).resolve()
    if not isinstance(metadata, dict):
        raise ValueError("Cargo metadata must be an object")
    packages = metadata.get("packages")
    resolve = metadata.get("resolve")
    if not isinstance(packages, list) or not isinstance(resolve, dict):
        raise ValueError("missing Cargo packages or resolve graph")
    nodes = resolve.get("nodes")
    if not isinstance(nodes, list) or not nodes:
        raise ValueError("missing Cargo resolve nodes")
    ids = {node.get("id") for node in nodes if isinstance(node, dict) and isinstance(node.get("id"), str)}
    if len(ids) != len(nodes):
        raise ValueError("malformed Cargo resolve nodes")
    found = 0
    for package in packages:
        if not isinstance(package, dict) or not isinstance(package.get("name"), str):
            raise ValueError("malformed Cargo package")
        if package["name"] not in LOCAL_FORBIDDEN:
            continue
        package_id = package.get("id")
        manifest_path = package.get("manifest_path")
        version = package.get("version")
        if not isinstance(package_id, str) or package_id not in ids or not isinstance(version, str):
            raise ValueError("observability package missing from resolve graph")
        if not isinstance(manifest_path, str) or not Path(manifest_path).is_absolute():
            raise ValueError("observability package has no absolute source path")
        path = Path(manifest_path).resolve()
        if package.get("source") != REGISTRY or path.is_relative_to(root):
            raise ValueError(f"non-registry observability package: {path}")
        if package["name"] == DEV_ADAPTER:
            raise ValueError("development observability adapter is present in resolved graph")
        if version != "1.5.0":
            raise ValueError(f"unexpected registry types version: {version}")
        if package["name"] == NAME:
            found += 1
    if found != 1:
        raise ValueError("expected exactly one resolved crates.io sc-observability-types 1.5.0")


def main():
    try:
        manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        # Reject declared local substitutes before Cargo, even if currently unused.
        validate_manifest(manifest, ROOT)
        result = subprocess.run(
            ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        )
        metadata = json.loads(result.stdout)
        validate(manifest, metadata, ROOT)
        # Workspace manifests can contain unused target or build overrides too.
        members = set(metadata.get("workspace_members", []))
        for package in metadata["packages"]:
            if package.get("id") in members:
                path = Path(package["manifest_path"])
                validate_manifest(tomllib.loads(path.read_text(encoding="utf-8")), path.parent)
    except (OSError, ValueError, subprocess.CalledProcessError, tomllib.TOMLDecodeError) as exc:
        print(f"observability release guard: FAIL: {exc}", file=sys.stderr)
        return 1
    print("observability release guard: OK (registry types 1.5.0 resolved)")
    return 0



if __name__ == "__main__":
    sys.exit(main())
