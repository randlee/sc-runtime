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

`store-boundaries` rejects cross-engine SQL tokens and blocking calls under
`crates/store-*`, and fails when the shared codec and redaction functions
drift across the store crates this layout rendered. A layout that omits a
store crate does not require it: `none` has no `store-sqlite`, and
`sqlite-mysql` has no `store-postgres`. Beads run-brx tracks hoisting those
functions into the schema crate.
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


# NFR-RUN-0002. Same pattern as the store sprint validation commands.
_BLOCKING_CALL = re.compile(
    r"std::thread::sleep(?:_until)?|tokio::time::sleep(?:_until)?"
    r"|Runtime::block_on|Handle::block_on|futures::executor::block_on"
)
# REQ-RUN-0321. MySQL must not contain the other engines' SQL.
_MYSQL_SQL = re.compile(r"RETURNING |ON CONFLICT|AUTOINCREMENT|PRAGMA ")
# Postgres and SQLite must not contain MySQL UUID() or backtick identifiers.
# Rustdoc backticks are comments and are stripped before this runs.
_OTHER_ENGINE_SQL = re.compile(r"UUID\(\)|`")

# Driver-agnostic copies. Beads run-brx hoists these into the schema crate.
# store-sqlite's record_from_columns omits the 1 MiB check; run-brx aligns it,
# so that copy is not in this set. Compare only crates the layout rendered.
_SHARED_FNS = (
    ("known_table", ("store-sqlite", "store-postgres", "store-mysql")),
    ("widget_from_payload", ("store-sqlite", "store-postgres", "store-mysql")),
    ("redact_credentials", ("store-postgres", "store-mysql")),
    ("redact_authority", ("store-postgres", "store-mysql")),
    ("record_from_columns", ("store-postgres", "store-mysql")),
)


def _raw_string_end(text: str, index: int) -> int | None:
    """End index of a Rust raw string starting at `index`, or None."""
    cursor = index
    if text[cursor] in "bc":
        cursor += 1
    if cursor >= len(text) or text[cursor] != "r":
        return None
    cursor += 1
    hashes = 0
    while cursor < len(text) and text[cursor] == "#":
        hashes += 1
        cursor += 1
    if cursor >= len(text) or text[cursor] != '"':
        return None
    cursor += 1
    closer = '"' + ("#" * hashes)
    end = text.find(closer, cursor)
    if end < 0:
        return len(text)
    return end + len(closer)


def _consume_quoted(text: str, index: int, quote: str) -> int:
    """Index just past a normal quoted string that starts at `index`."""
    cursor = index + 1
    while cursor < len(text):
        if text[cursor] == "\\":
            cursor += 2
            continue
        if text[cursor] == quote:
            return cursor + 1
        cursor += 1
    return len(text)


def _skip_token(text: str, index: int, *, sql: bool) -> int | None:
    """If `index` starts a comment or string, return the index after it."""
    if index >= len(text):
        return None
    if text.startswith("/*", index):
        end = text.find("*/", index + 2)
        return len(text) if end < 0 else end + 2
    if not sql and text.startswith("//", index):
        end = text.find("\n", index)
        return len(text) if end < 0 else end
    if sql and text.startswith("--", index):
        end = text.find("\n", index)
        return len(text) if end < 0 else end
    raw = _raw_string_end(text, index)
    if raw is not None:
        return raw
    if text[index] == '"':
        return _consume_quoted(text, index, '"')
    if text[index] == "'":
        return _consume_quoted(text, index, "'")
    return None


def strip_comments(text: str, *, sql: bool) -> str:
    """Replace comments with spaces, preserving newlines and string contents."""
    out: list[str] = []
    index = 0
    while index < len(text):
        if text.startswith("/*", index) or (not sql and text.startswith("//", index)) or (
            sql and text.startswith("--", index)
        ):
            end = _skip_token(text, index, sql=sql)
            assert end is not None
            out.append(re.sub(r"[^\n]", " ", text[index:end]))
            index = end
            continue
        skipped = _skip_token(text, index, sql=sql)
        if skipped is not None and skipped != index:
            out.append(text[index:skipped])
            index = skipped
            continue
        out.append(text[index])
        index += 1
    return "".join(out)


def _matching_brace(text: str, open_brace: int) -> int:
    """Index just past the block that opens at `open_brace`."""
    depth = 0
    index = open_brace
    while index < len(text):
        skipped = _skip_token(text, index, sql=False)
        if skipped is not None and skipped != index:
            index = skipped
            continue
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                return index + 1
        index += 1
    fail("unbalanced brace while reading a store function")
    return len(text)


def rust_function(text: str, name: str) -> str:
    """The `fn name(...) { ... }` item, including its body."""
    needle = f"fn {name}("
    starts: list[int] = []
    index = 0
    while True:
        found = text.find(needle, index)
        if found < 0:
            break
        if found == 0 or not (text[found - 1].isalnum() or text[found - 1] == "_"):
            starts.append(found)
        index = found + len(needle)
    if len(starts) != 1:
        fail(f"fn {name} is defined {len(starts)} times")
    cursor = starts[0]
    while cursor < len(text) and text[cursor] != "{":
        skipped = _skip_token(text, cursor, sql=False)
        if skipped is not None and skipped != cursor:
            cursor = skipped
            continue
        cursor += 1
    if cursor >= len(text) or text[cursor] != "{":
        fail(f"fn {name} has no body")
    return text[starts[0] : _matching_brace(text, cursor)]


def canonical_rust(text: str) -> str:
    return re.sub(r"\s+", "", strip_comments(text, sql=False))


def store_code_files(crate: Path) -> list[Path]:
    found: list[Path] = []
    if not crate.is_dir():
        return found
    for path in crate.rglob("*"):
        if not path.is_file() or "target" in path.parts:
            continue
        if path.suffix in {".rs", ".sql"}:
            found.append(path)
    return sorted(found)


def _sql_rule(crate_name: str) -> re.Pattern[str] | None:
    if crate_name == "store-mysql":
        return _MYSQL_SQL
    if crate_name in {"store-postgres", "store-sqlite"}:
        return _OTHER_ENGINE_SQL
    return None


def scan_store_tokens(crates: Path) -> None:
    if not crates.is_dir():
        fail(f"store-boundaries: missing {crates}")
    crates_found = sorted(
        path for path in crates.iterdir() if path.is_dir() and path.name.startswith("store-")
    )
    for crate in crates_found:
        rule = _sql_rule(crate.name)
        for path in store_code_files(crate):
            text = path.read_text(encoding="utf-8")
            visible = strip_comments(text, sql=path.suffix == ".sql")
            patterns = [_BLOCKING_CALL]
            if rule is not None:
                patterns.append(rule)
            for pattern in patterns:
                matched = pattern.search(visible)
                if matched is None:
                    continue
                line = visible.count("\n", 0, matched.start()) + 1
                fail(
                    f"store-boundaries: {path}:{line} matches {pattern.pattern!r} ({matched.group(0)!r})"
                )


def _present_store_crates(crates: Path) -> set[str]:
    if not crates.is_dir():
        return set()
    return {
        path.name
        for path in crates.iterdir()
        if path.is_dir()
        and path.name.startswith("store-")
        and (path / "src" / "lib.rs").is_file()
    }


def scan_shared_functions(crates: Path) -> None:
    present = _present_store_crates(crates)
    if not present:
        return
    for name, crate_names in _SHARED_FNS:
        active = [crate_name for crate_name in crate_names if crate_name in present]
        if len(active) < 2:
            continue
        bodies: list[str] = []
        for crate_name in active:
            path = crates / crate_name / "src" / "lib.rs"
            bodies.append(canonical_rust(rust_function(path.read_text(encoding="utf-8"), name)))
        first = bodies[0]
        for crate_name, body in zip(active, bodies, strict=True):
            if body != first:
                fail(
                    f"store-boundaries: fn {name} in {crate_name} drifted from {active[0]}"
                )


def store_boundaries(crates: Path | None = None) -> None:
    root = Path("crates") if crates is None else crates
    scan_store_tokens(root)
    scan_shared_functions(root)


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
    elif command == "store-boundaries":
        store_boundaries()
    else:
        fail(
            "usage: consumer_checks.py "
            "observability|build-cli|build-daemon|db-prepare|store-boundaries [--require]"
        )


if __name__ == "__main__":
    main()
