"""Store-boundary scan for this repository's tests.

This is not a generated consumer check. ADR-RUN-0005 keeps crate-boundary
rules out of `template/`. `tests/unit/generation` runs this module against
the template store crates and against layout subsets.
"""

import re
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


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
# redact_credentials, redact_authority, and record_from_columns may diverge
# by URL dialect. They are not identity-checked. known_table and
# widget_from_payload stay shared.
_SHARED_FNS = (
    ("known_table", ("store-sqlite", "store-postgres", "store-mysql")),
    ("widget_from_payload", ("store-sqlite", "store-postgres", "store-mysql")),
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


