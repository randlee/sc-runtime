#!/usr/bin/env python3
"""Consumer-owned checks invoked by the generated Justfile.

The sc-observability pin lives in the workspace manifest. Consumers may bump
that version; this script reads it instead of embedding a version literal.
The durable check is the absence of sc-observability-otlp.

`db-prepare` walks `crates/store-*` directories that contain `sqlx.toml`.
The database URL variable comes from that file. A missing crate is skipped.
An unset URL is skipped unless `DB_PREPARE_STRICT=1` or `--require` is set,
except `SQLITE_DATABASE_URL`, which defaults to a file under `target/` so a
file-backed engine can prepare without a server.

"""

import os
import re
import subprocess
import sys
from pathlib import Path

# Store crates are derived from crates/store-* in build_cli. Do not list them
# here; a later engine would otherwise need another manual edit.
SERVER_ONLY = (
    "axum",
    "rmcp",
    "sqlx",
    "sc-runtime",
    "sc-observability",
    "service",
    "daemon",
    "utoipa",
    "utoipa-axum",
    "schemars",
    "fd-lock",
)

_URL_VAR = re.compile(r'(?m)^\s*database-url-var\s*=\s*"([^"]+)"\s*$')


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def manifest_package_name(crate: Path) -> str:
    """`[package] name` from a crate manifest."""
    section = ""
    for line in (crate / "Cargo.toml").read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            section = stripped
            continue
        if section != "[package]":
            continue
        if stripped.startswith("name") and "=" in stripped:
            return stripped.split("=", 1)[1].strip().strip('"')
    fail(f"{crate / 'Cargo.toml'} has no package name")
    return ""


def cli_package() -> str:
    return manifest_package_name(Path("crates/cli"))


def daemon_package() -> str:
    return manifest_package_name(Path("crates/daemon"))


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


def server_only_names(crates: Path = Path("crates")) -> set[str]:
    """Fixed server crates plus the package name of every server directory."""
    names = set(SERVER_ONLY)
    if not crates.is_dir():
        return names
    for path in crates.iterdir():
        if not path.is_dir() or not (path / "Cargo.toml").is_file():
            continue
        if path.name in {"daemon", "service"} or path.name.startswith("store-"):
            names.add(manifest_package_name(path))
    return names


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
    names = package_names(cargo_tree(cli_package()))
    found = [name for name in names if name in server_only_names()]
    if found:
        fail("server-only crates in the cli normal tree: " + ", ".join(found))


def build_daemon() -> None:
    names = package_names(cargo_tree(daemon_package()))
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


def store_directories(crates: Path) -> list[Path]:
    """`crates/store-*` directories that carry their own `sqlx.toml`."""
    if not crates.is_dir():
        return []
    found = [
        path
        for path in crates.iterdir()
        if path.is_dir() and path.name.startswith("store-") and (path / "sqlx.toml").is_file()
    ]
    return sorted(found, key=lambda path: path.name)


def database_url_var(sqlx_toml: Path) -> str:
    text = sqlx_toml.read_text(encoding="utf-8")
    matched = _URL_VAR.search(text)
    if matched is None:
        fail(f"{sqlx_toml} must set database-url-var")
    assert matched is not None
    return matched.group(1)


def prepare_is_strict(argv: list[str]) -> bool:
    if "--require" in argv:
        return True
    return os.environ.get("DB_PREPARE_STRICT", "").strip() == "1"


def sqlite_fallback(var: str, sqlite_file: Path) -> str | None:
    if var == "SQLITE_DATABASE_URL":
        return f"sqlite:{sqlite_file}"
    return None


def resolve_prepare_url(var: str, env: dict[str, str], sqlite_file: Path) -> str:
    url = env.get(var, "").strip()
    if url:
        return url
    return sqlite_fallback(var, sqlite_file) or ""


def missing_url_message(crate: Path, var: str) -> str:
    return (
        f"db-prepare: {var} is unset; leaving {crate / '.sqlx'} unchanged. "
        f"Set {var} and rerun `scripts/consumer_checks.py db-prepare`, "
        "or set DB_PREPARE_STRICT=1 (or pass --require) to fail this skip."
    )


def prepare_store(
    crate: Path, env: dict[str, str], sqlite_file: Path, strict: bool
) -> None:
    var = database_url_var(crate / "sqlx.toml")
    if var == "SQLITE_DATABASE_URL":
        sqlite_file.parent.mkdir(exist_ok=True)
    url = resolve_prepare_url(var, env, sqlite_file)
    if not url:
        message = missing_url_message(crate, var)
        if strict:
            fail(message)
        print(message, file=sys.stderr)
        return
    crate_env = dict(env)
    crate_env[var] = url
    # Create, migrate, and prepare inside the crate. An absolute SQLite path
    # still points at the same file when cargo's cwd is the crate directory.
    run(["cargo", "sqlx", "database", "create"], crate_env, crate)
    run(["cargo", "sqlx", "migrate", "run"], crate_env, crate)
    prepare_crate(crate.name, crate_env)


def db_prepare(argv: list[str] | None = None) -> None:
    argv = sys.argv if argv is None else argv
    strict = prepare_is_strict(argv)
    env = dict(os.environ)
    sqlite_file = (Path("target") / "db-prepare.sqlite").resolve()
    crates = store_directories(Path("crates"))
    if not crates:
        print("db-prepare: no crates/store-* with sqlx.toml", file=sys.stderr)
        return
    for crate in crates:
        prepare_store(crate, env, sqlite_file, strict)


def main() -> None:
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    if command == "observability":
        observability()
    elif command == "build-cli":
        build_cli()
    elif command == "build-daemon":
        build_daemon()
    elif command == "db-prepare":
        db_prepare(sys.argv)
    else:
        fail(
            "usage: consumer_checks.py "
            "observability|build-cli|build-daemon|db-prepare [--require]"
        )


if __name__ == "__main__":
    main()
