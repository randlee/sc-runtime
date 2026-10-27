#!/usr/bin/env python3
"""Prefix generated package names with the project name.

The template workspace keeps role names (`store-sqlite`, `api-types`, …) so
`cargo test -p store-sqlite --manifest-path template/Cargo.toml` still
selects those crates. This hook runs after Liquid, inside the generated
project. It does not change Rust crate names: `[lib] name` stays the role,
and dependency keys stay the role. The package name becomes `<project>-<role>`.
"""

from __future__ import annotations

import re
import shutil
import sys
from pathlib import Path

# Old package name -> (generated role, rust lib name).
ROLES: dict[str, tuple[str, str]] = {
    "api-types": ("types", "api_types"),
    "schema": ("schema", "schema"),
    "store-sqlite": ("store-sqlite", "store_sqlite"),
    "store-postgres": ("store-postgres", "store_postgres"),
    "store-mysql": ("store-mysql", "store_mysql"),
    "store-adaptor": ("store-adaptor", "store_adaptor"),
    "store-adaptor-sqlite-postgres": ("store-adaptor", "store_adaptor"),
    "store-adaptor-sqlite-mysql": ("store-adaptor", "store_adaptor"),
    "service": ("service", "service"),
    "daemon": ("daemon", "daemon"),
    "cli": ("cli", "cli"),
}

_NAME = re.compile(r'^(\s*name\s*=\s*)"([^"]+)"(\s*)$')
_PACKAGE = re.compile(r'package\s*=\s*"([^"]+)"')
_DEP_HEADER = re.compile(r"^\[(dev-dependencies|build-dependencies|dependencies)\]$")


def mapping(project: str) -> dict[str, str]:
    return {old: f"{project}-{role}" for old, (role, _lib) in ROLES.items()}


def rewrite(text: str, project: str, crate_dir: Path) -> str:
    """Return `text` with this crate's package name and dependency packages prefixed."""
    names = mapping(project)
    lines = text.splitlines()
    section = ""
    renamed_from: str | None = None
    out: list[str] = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped
            out.append(line)
            continue
        if section == "[package]":
            matched = _NAME.match(line)
            if matched and matched.group(2) in names:
                renamed_from = matched.group(2)
                out.append(f'{matched.group(1)}"{names[renamed_from]}"{matched.group(3)}')
                continue
        if _DEP_HEADER.match(section) and "=" in line and not stripped.startswith("#"):
            out.append(_rewrite_dep(line, names))
            continue
        out.append(line)
    rendered = "\n".join(out)
    if text.endswith("\n"):
        rendered += "\n"
    if renamed_from is None:
        return rendered
    return _ensure_lib_name(rendered, ROLES[renamed_from][1], crate_dir)


def _rewrite_dep(line: str, names: dict[str, str]) -> str:
    key, rest = line.split("=", 1)
    dep_key = key.strip()
    body = rest.lstrip()
    indent = line[: len(line) - len(line.lstrip())]
    if not body.startswith("{"):
        return line

    def replace_package(match: re.Match[str]) -> str:
        old = match.group(1)
        if old not in names:
            return match.group(0)
        return f'package = "{names[old]}"'

    updated, count = _PACKAGE.subn(replace_package, body, count=1)
    if count == 0 and dep_key in names:
        updated = body.replace("{", '{ package = "%s",' % names[dep_key], 1)
    if updated == body:
        return line
    # Keep a single space around `=`.
    return f"{indent}{dep_key} = {updated.lstrip()}"


def _ensure_lib_name(text: str, lib_name: str, crate_dir: Path) -> str:
    if "[lib]" in text or not (crate_dir / "src" / "lib.rs").is_file():
        return text
    lines = text.splitlines()
    insert_at = 0
    in_package = False
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            continue
        if in_package and stripped.startswith("[") and stripped.endswith("]"):
            insert_at = index
            break
    else:
        insert_at = len(lines)
    block = ["[lib]", f'name = "{lib_name}"', ""]
    lines[insert_at:insert_at] = block
    rendered = "\n".join(lines)
    if text.endswith("\n"):
        rendered += "\n"
    return rendered


def prefix_tree(project: str, dest: Path) -> None:
    for path in sorted(dest.rglob("Cargo.toml")):
        if "target" in path.parts:
            continue
        original = path.read_text(encoding="utf-8")
        updated = rewrite(original, project, path.parent)
        if updated != original:
            path.write_text(updated, encoding="utf-8")


_LAYOUT_MANIFESTS = {
    "sqlite-postgres": "Cargo.sqlite-postgres.toml",
    "sqlite-mysql": "Cargo.sqlite-mysql.toml",
}

_RENDERED_LIB = """#[path = "../../schema/tests/common/membership_toml.rs"]
mod membership_toml;

include!("shared.rs");
include!("membership.rs");
include!("engine.rs");
include!("layout_tests.rs");
"""


def prepare_adaptor(dest: Path, db: str) -> None:
    """Install one `crates/store-adaptor` for a two-store layout.

    One-store and no-database layouts ignore that directory, so this returns
    without looking for it. A two-store layout replaces the checkout manifest
    with the engine manifest, writes `engine.rs` from the selected engine
    file, and deletes the other engine file. The rendered `lib.rs` is the
    same text for Postgres and MySQL.
    """
    manifest_name = _LAYOUT_MANIFESTS.get(db)
    if manifest_name is None:
        return
    crate = dest / "crates" / "store-adaptor"
    if not crate.is_dir():
        print("prefix_packages: missing crates/store-adaptor", file=sys.stderr)
        raise SystemExit(1)
    source = crate / manifest_name
    engine_src = crate / "src" / "engines" / f"{db}.rs"
    if not source.is_file() or not engine_src.is_file():
        print("prefix_packages: adaptor layout files are missing", file=sys.stderr)
        raise SystemExit(1)
    (crate / "Cargo.toml").write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
    for name in _LAYOUT_MANIFESTS.values():
        (crate / name).unlink(missing_ok=True)
    (crate / "src" / "engine.rs").write_text(
        engine_src.read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    engines = crate / "src" / "engines"
    if engines.is_dir():
        shutil.rmtree(engines)
    (crate / "src" / "lib.rs").write_text(_RENDERED_LIB, encoding="utf-8")
    for sidecar in ("store-adaptor-sqlite-postgres", "store-adaptor-sqlite-mysql"):
        path = dest / "crates" / sidecar
        if path.exists():
            shutil.rmtree(path)


def main(argv: list[str]) -> None:
    if len(argv) != 4:
        print("usage: prefix_packages.py PROJECT DEST DB", file=sys.stderr)
        raise SystemExit(2)
    project, dest, db = argv[1], Path(argv[2]), argv[3]
    if not project or not dest.is_dir():
        print("prefix_packages: project and destination are required", file=sys.stderr)
        raise SystemExit(1)
    prepare_adaptor(dest, db)
    prefix_tree(project, dest)


if __name__ == "__main__":
    main(sys.argv)
