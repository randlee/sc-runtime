#!/usr/bin/env python3
"""Consumer-owned checks invoked by the generated Justfile."""

import re
import subprocess
import sys
from pathlib import Path

SERVER_ONLY = (
    "axum",
    "rmcp",
    "sqlx",
    "sc-runtime",
    "service",
    "store-sqlite",
    "daemon",
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


def observability() -> None:
    text = Path("crates/daemon/Cargo.toml").read_text(encoding="utf-8")
    if "sc-observability-otlp" in text:
        fail("sc-observability-otlp is present in crates/daemon/Cargo.toml")
    if re.search(r'(?m)^sc-observability\s*=\s*"1\.5\.0"', text) is None:
        fail("crates/daemon must require sc-observability 1.5.0")


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
