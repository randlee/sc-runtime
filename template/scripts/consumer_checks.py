#!/usr/bin/env python3
"""Consumer-owned checks invoked by the generated Justfile.

The sc-observability pin lives in the workspace manifest. Consumers may bump
that version; this script reads it instead of embedding a version literal.
The durable check is the absence of sc-observability-otlp.
"""

import os
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
    "store-postgres",
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


def run(command: list[str], env: dict[str, str], cwd: Path | None = None) -> None:
    completed = subprocess.run(command, env=env, cwd=cwd, check=False)
    if completed.returncode != 0:
        where = f" (cwd {cwd})" if cwd is not None else ""
        fail("db-prepare failed: " + " ".join(command) + where)


def prepare_crate(package: str, env: dict[str, str]) -> None:
    # Prepare inside the crate so `.sqlx` lands next to that crate's Cargo.toml.
    # `sqlx prepare --workspace` writes one cache at the workspace root instead.
    run(
        ["cargo", "sqlx", "prepare", "--", "--all-targets"],
        env,
        Path("crates") / package,
    )


def db_prepare() -> None:
    env = dict(os.environ)
    # Absolute path so `prepare` can run inside the crate directory. A relative
    # `sqlite:target/...` URL would point at a different file from that cwd.
    sqlite_file = (Path("target") / "db-prepare.sqlite").resolve()
    sqlite_file.parent.mkdir(exist_ok=True)
    env.setdefault("SQLITE_DATABASE_URL", f"sqlite:{sqlite_file}")
    run(["cargo", "sqlx", "database", "create"], env)
    run(
        [
            "cargo",
            "sqlx",
            "migrate",
            "run",
            "--source",
            "crates/store-sqlite/migrations",
        ],
        env,
    )
    prepare_crate("store-sqlite", env)

    postgres = Path("crates/store-postgres")
    if not postgres.is_dir():
        return
    url = env.get("POSTGRES_DATABASE_URL", "").strip()
    if not url:
        print(
            "db-prepare: POSTGRES_DATABASE_URL is unset; leaving crates/store-postgres/.sqlx unchanged",
            file=sys.stderr,
        )
        return
    pg_env = dict(env)
    pg_env["POSTGRES_DATABASE_URL"] = url
    run(["cargo", "sqlx", "database", "create"], pg_env, postgres)
    run(["cargo", "sqlx", "migrate", "run"], pg_env, postgres)
    prepare_crate("store-postgres", pg_env)


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
