#!/usr/bin/env python3
"""Reject development observability patches before release publication."""

import json
import importlib.util
import os
import re
import tempfile
import urllib.error
import urllib.request
from urllib.parse import urlparse
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
# Repo-only preflight reuses the validated session/Windows Job owner. Resolve
# from this file, never the caller's cwd; marker state is set only in children.
_owned_spec = importlib.util.spec_from_file_location(
    "sc_runtime_owned_commands", ROOT / "tests/integration/generation/owned_process.py")
_owned = importlib.util.module_from_spec(_owned_spec)
_owned_spec.loader.exec_module(_owned)
NAME = "sc-observability-types"
DEV_ADAPTER = "sc-observability-dev-adapter"
LOCAL_FORBIDDEN = {NAME, "sc-observability", DEV_ADAPTER}
REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"


UPSTREAM_VERSION = "1.5.1"
UPSTREAM_NAMES = ("sc-observability-types", "sc-observability")
CLEANUP = "https://github.com/randlee/sc-runtime/issues/97"


def registry_version(name):
    """Fetch exact crates.io release metadata afresh; only HTTP 404 means absent."""
    url = f"https://crates.io/api/v1/crates/{name}/{UPSTREAM_VERSION}"
    request = urllib.request.Request(url, headers={"User-Agent": "sc-runtime-release-readiness/1",
                                                    "Cache-Control": "no-cache, no-store", "Pragma": "no-cache"})
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            final = urlparse(response.geturl())
            if final.scheme != "https" or final.hostname != "crates.io" or final.path != urlparse(url).path:
                raise ValueError(f"{name}: unexpected registry response source")
            data = json.loads(response.read(1024 * 1024 + 1))
    except urllib.error.HTTPError as exc:
        if exc.code == 404 and exc.geturl() == url:
            return None
        raise ValueError(f"{name}: registry HTTP {exc.code}; availability indeterminate") from exc
    except (urllib.error.URLError, TimeoutError, OSError, json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise ValueError(f"{name}: registry request failed; availability indeterminate: {exc}") from exc
    version = data.get("version") if isinstance(data, dict) else None
    if not isinstance(version, dict) or version.get("crate") != name or version.get("num") != UPSTREAM_VERSION:
        raise ValueError(f"{name}: malformed or mismatched registry version metadata")
    checksum = version.get("checksum")
    if version.get("yanked") is not False or not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum):
        raise ValueError(f"{name}: yanked or missing/invalid registry checksum")
    return checksum


def run_probe_cargo(args, root, env, context, timeout=180):
    try:
        result = _owned.run(args, cwd=root, env=env, timeout=timeout, stdin=subprocess.DEVNULL,
                            capture_output=True, text=True)
    except subprocess.TimeoutExpired as exc:
        raise ValueError(f"{context}: Cargo timed out after {timeout}s; owned tree terminated and reaped") from exc
    except (OSError, RuntimeError, TimeoutError, subprocess.SubprocessError) as exc:
        raise ValueError(f"{context}: owned Cargo execution or cleanup failed: {exc}") from exc
    if result.returncode:
        raise ValueError(f"{context}: Cargo failed ({result.returncode}): {result.stderr.strip()}")
    return result.stdout


def validate_probe(metadata, lock, checksums):
    """Require one resolved registry identity and the fresh index checksum for each package."""
    if not isinstance(metadata, dict) or not isinstance(metadata.get("packages"), list):
        raise ValueError("capability probe metadata malformed")
    resolve = metadata.get("resolve")
    if not isinstance(resolve, dict) or not isinstance(resolve.get("nodes"), list):
        raise ValueError("capability probe resolve graph malformed")
    if not isinstance(lock, dict) or not isinstance(lock.get("package"), list):
        raise ValueError("capability probe lock malformed")
    nodes = resolve["nodes"]
    ids = {node.get("id") for node in nodes if isinstance(node, dict)}
    for name in UPSTREAM_NAMES:
        packages = [p for p in metadata["packages"] if isinstance(p, dict) and p.get("name") == name]
        locked = [p for p in lock["package"] if isinstance(p, dict) and p.get("name") == name]
        if (len(packages) != 1 or len(locked) != 1 or packages[0].get("id") not in ids
                or packages[0].get("source") != REGISTRY or packages[0].get("version") != UPSTREAM_VERSION
                or locked[0].get("source") != REGISTRY or locked[0].get("version") != UPSTREAM_VERSION
                or locked[0].get("checksum") != checksums[name]):
            raise ValueError(f"capability probe {name}: registry identity/version/checksum mismatch")


def probe_upstream(checksums):
    with tempfile.TemporaryDirectory(prefix="sc-observability-capability-") as directory:
        root = Path(directory)
        (root / "src").mkdir()
        (root / "Cargo.toml").write_text(
            '[package]\nname = "sc-observability-capability-probe"\nversion = "0.0.0"\nedition = "2021"\n'
            '[workspace]\n[dependencies]\nsc-observability = "=1.5.1"\nsc-observability-types = "=1.5.1"\n')
        (root / "src/lib.rs").write_text("""
use std::sync::Arc;
use sc_observability_types::{CanonicalLog, SharedCanonicalLog, NoopCanonicalLog, ServiceName, LogEvent, v2::EventError};
fn signatures(log: &dyn CanonicalLog, event: LogEvent, name: ServiceName) -> Result<(), EventError> {
    let _: &ServiceName = log.service_name();
    let _: NoopCanonicalLog = NoopCanonicalLog::new(name);
    let _: NoopCanonicalLog = NoopCanonicalLog::named_or_default("noop");
    log.emit(event)
}
fn direct(logger: Arc<sc_observability::v2::Logger>) -> SharedCanonicalLog { logger }
""")
        env = {key: value for key, value in os.environ.items() if not key.startswith("CARGO_")}
        env["CARGO_HOME"] = str(root / "cargo-home")
        env["CARGO_TERM_COLOR"] = "never"
        metadata = json.loads(run_probe_cargo(["cargo", "metadata", "--format-version", "1"], root, env,
                                               "upstream capability metadata"))
        lock = tomllib.loads((root / "Cargo.lock").read_text())
        validate_probe(metadata, lock, checksums)
        run_probe_cargo(["cargo", "check", "--locked"], root, env, "upstream capability compilation")


def check_upstream_readiness():
    checksums = {name: registry_version(name) for name in UPSTREAM_NAMES}
    for name, checksum in checksums.items():
        print(f"crates.io {name} {UPSTREAM_VERSION}: " + (f"published checksum={checksum}" if checksum else "confirmed absent (HTTP 404)"))
    if all(value is None for value in checksums.values()):
        return
    if any(value is None for value in checksums.values()):
        raise ValueError("upstream 1.5.1 publication is partial; readiness indeterminate")
    try:
        probe_upstream(checksums)
    except (ValueError, OSError, tomllib.TOMLDecodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"upstream 1.5.1 capability indeterminate or incompatible: {exc}; see {CLEANUP}") from exc
    raise ValueError(f"upstream 1.5.1 capability confirmed; complete shim cleanup before release: {CLEANUP}")


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
        check_upstream_readiness()
        metadata = json.loads(run_probe_cargo(
            ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
            ROOT, os.environ.copy(), "workspace release metadata"))
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
