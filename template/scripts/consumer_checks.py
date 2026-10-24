#!/usr/bin/env python3
"""Consumer-owned checks invoked by the generated Justfile.

The sc-observability pin lives in the workspace manifest. Consumers may bump
that version; this script reads it instead of embedding a version literal.
The durable check is the absence of sc-observability-otlp.
"""

import re
import subprocess
import sys
from pathlib import Path

SERVER_ONLY = (
    "axum",
    "rmcp",
    "sqlx",
    "sc-runtime",
    "sc-observability",
    "service",
    "store-sqlite",
    "daemon",
    "utoipa",
    "utoipa-axum",
    "schemars",
    "fd-lock",
)


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def cargo_tree(package: str) -> str:
    completed = subprocess.run(
        ["cargo", "tree", "-p", package, "-e", "normal", "--prefix", "none"],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        fail(completed.stderr.strip() or f"cargo tree -p {package} failed")
    return completed.stdout


def package_names(tree: str) -> list[str]:
    names = []
    for line in tree.splitlines():
        token = line.split(" ", 1)[0]
        if token:
            names.append(token)
    return names


def workspace_dep_version(text: str, name: str) -> str | None:
    in_section = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_section = stripped == "[workspace.dependencies]"
            continue
        if not in_section or stripped.split("=", 1)[0].strip() != name:
            continue
        matched = re.search(r'version\s*=\s*"([^"]+)"', stripped) or re.search(
            r'=\s*"([^"]+)"', stripped
        )
        if matched:
            return matched.group(1)
    return None


def observability() -> None:
    root = Path("Cargo.toml").read_text(encoding="utf-8")
    daemon = Path("crates/daemon/Cargo.toml").read_text(encoding="utf-8")
    service = Path("crates/service/Cargo.toml").read_text(encoding="utf-8")
    combined = root + daemon + service
    if "sc-observability-otlp" in combined:
        fail("sc-observability-otlp is present")
    if not workspace_dep_version(root, "sc-observability"):
        fail("workspace.dependencies must declare sc-observability")
    if not workspace_dep_version(root, "sc-observability-types"):
        fail("workspace.dependencies must declare sc-observability-types")
    if re.search(r"(?m)^sc-observability(?:\.workspace)?\s*=", daemon) is None:
        fail("crates/daemon must require sc-observability")
    if re.search(r"(?m)^sc-observability-types(?:\.workspace)?\s*=", service) is None:
        fail("crates/service must require sc-observability-types")


def build_cli() -> None:
    print(
        "cargo build --workspace is the exemption; it is not the shipped agent binary",
        file=sys.stderr,
    )
    names = package_names(cargo_tree("cli"))
    found = [name for name in names if name in SERVER_ONLY]
    if found:
        fail("server-only crates in the cli normal tree: " + ", ".join(found))


def build_daemon() -> None:
    names = package_names(cargo_tree("daemon"))
    found = [name for name in names if name in {"tempfile", "tempdir"}]
    if found:
        fail("tempfile or tempdir in the daemon normal tree: " + ", ".join(found))


def db_prepare() -> None:
    env = dict(**{key: value for key, value in __import__("os").environ.items()})
    env.setdefault("SQLITE_DATABASE_URL", "sqlite:target/db-prepare.sqlite")
    Path("target").mkdir(exist_ok=True)
    for command in (
        ["cargo", "sqlx", "database", "create"],
        ["cargo", "sqlx", "migrate", "run", "--source", "crates/store-sqlite/migrations"],
        ["cargo", "sqlx", "prepare", "--workspace", "--", "--all-targets"],
    ):
        completed = subprocess.run(command, env=env, check=False)
        if completed.returncode != 0:
            fail("db-prepare failed: " + " ".join(command))


def main() -> None:
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    if command == "observability":
        observability()
    elif command == "build-cli":
        build_cli()
    elif command == "build-daemon":
        build_daemon()
    elif command == "db-prepare":
        db_prepare()
    else:
        fail("usage: consumer_checks.py observability|build-cli|build-daemon|db-prepare")


if __name__ == "__main__":
    main()
