#!/usr/bin/env python3
"""ADR-RUN-0011 point 1 scanner.

Fails when production Rust under ``crates/`` or ``template/`` calls a banned
sleep or ``block_on`` form outside a ``cfg`` predicate that requires ``test``
and outside a same-line ``allow:`` comment. ``#[cfg(test)]`` and
``#[cfg(all(test, feature = "test-util"))]`` are that exemption. A ``mod name;``
under such a cfg exempts the included module file, which has no cfg of its own.
``yield_now`` is not a match. ``just adr-run-0011`` runs this file with the
platform ``python`` command.
"""

from __future__ import annotations

import pathlib
import re
import sys

CALL = re.compile(
    r"std::thread::sleep_until|std::thread::sleep(?!_)|"
    r"tokio::time::sleep_until|tokio::time::sleep(?!_)|"
    r"Runtime::block_on|Handle::block_on|futures::executor::block_on|"
    r"\.block_on\s*\("
)
ALLOW = re.compile(r"\ballow\b\s*:\s*\S")
_CFG_OPEN = re.compile(r"#\s*\[\s*cfg\s*\(")
_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_MOD = re.compile(r"(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)")


def _skip_ws(text: str, index: int) -> int:
    length = len(text)
    while index < length and text[index].isspace():
        index += 1
    return index


def _skip_ws_comments(text: str, index: int) -> int:
    length = len(text)
    while index < length:
        if text[index].isspace():
            index += 1
            continue
        if text.startswith("//", index):
            end = text.find("\n", index)
            index = length if end == -1 else end + 1
            continue
        if text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = length if end == -1 else end + 2
            continue
        break
    return index


def _group_open(text: str, index: int, name: str) -> int | None:
    if not text.startswith(name, index):
        return None
    end = index + len(name)
    if end < len(text) and (text[end].isalnum() or text[end] == "_"):
        return None
    cursor = _skip_ws(text, end)
    if cursor < len(text) and text[cursor] == "(":
        return cursor + 1
    return None


def _parse_predicate(text: str, index: int) -> tuple[tuple[str, object], int] | None:
    """Parse one cfg predicate starting at ``index``."""
    index = _skip_ws(text, index)
    opened_name: str | None = None
    opened_at: int | None = None
    for name in ("all", "any", "not"):
        opened = _group_open(text, index, name)
        if opened is not None:
            opened_name = name
            opened_at = opened
            break
    if opened_name is not None and opened_at is not None:
        args: list[tuple[str, object]] = []
        index = opened_at
        while True:
            index = _skip_ws(text, index)
            if index < len(text) and text[index] == ")":
                return (opened_name, args), index + 1
            parsed = _parse_predicate(text, index)
            if parsed is None:
                return None
            node, index = parsed
            args.append(node)
            index = _skip_ws(text, index)
            if index < len(text) and text[index] == ",":
                index += 1
                continue
            if index < len(text) and text[index] == ")":
                return (opened_name, args), index + 1
            return None
    match = _IDENT.match(text, index)
    if match is None:
        return None
    name = match.group(0)
    index = match.end()
    after = _skip_ws(text, index)
    if after < len(text) and text[after] == "=":
        after = _skip_ws(text, after + 1)
        if after >= len(text) or text[after] != '"':
            return None
        cursor = after + 1
        length = len(text)
        while cursor < length and text[cursor] != '"':
            if text[cursor] == "\\":
                cursor += 2
                continue
            cursor += 1
        if cursor >= length:
            return None
        return ("meta", name), cursor + 1
    return ("ident", name), index


def _without_test(node: tuple[str, object]) -> str:
    """Whether ``node`` is true, false, or unknown when ``test`` is off.

    ``false`` means the predicate cannot hold in a production build.
    """
    kind, payload = node
    if kind == "ident":
        return "false" if payload == "test" else "maybe"
    if kind == "meta":
        return "maybe"
    if kind == "not":
        args = payload if isinstance(payload, list) else []
        if len(args) != 1:
            return "maybe"
        return {"true": "false", "false": "true"}.get(_without_test(args[0]), "maybe")
    if kind in {"all", "any"}:
        args = payload if isinstance(payload, list) else []
        values = [_without_test(arg) for arg in args]
        if kind == "all":
            if "false" in values:
                return "false"
            if "maybe" in values:
                return "maybe"
            return "true"
        if "true" in values:
            return "true"
        if "maybe" in values:
            return "maybe"
        return "false"
    return "maybe"


def _cfg_requires_test_at(text: str, index: int) -> bool:
    """True when a ``#[cfg(...)]`` at ``index`` is false unless ``test`` is set."""
    match = _CFG_OPEN.match(text, index)
    if match is None:
        return False
    parsed = _parse_predicate(text, match.end())
    if parsed is None:
        return False
    node, end = parsed
    end = _skip_ws(text, end)
    if end >= len(text) or text[end] != ")":
        return False
    end = _skip_ws(text, end + 1)
    if end >= len(text) or text[end] != "]":
        return False
    return _without_test(node) == "false"


def _skip_attribute(text: str, index: int) -> int | None:
    if not (text.startswith("#[", index) or text.startswith("#![", index)):
        return None
    start = text.find("[", index)
    if start == -1:
        return None
    depth = 0
    cursor = start
    length = len(text)
    while cursor < length:
        char = text[cursor]
        if char == '"':
            cursor += 1
            while cursor < length and text[cursor] != '"':
                if text[cursor] == "\\":
                    cursor += 2
                    continue
                cursor += 1
            cursor = min(cursor + 1, length)
            continue
        if char == "[":
            depth += 1
        elif char == "]":
            depth -= 1
            if depth == 0:
                return cursor + 1
        cursor += 1
    return None


def _declared_module(statement: str) -> str | None:
    """Return ``name`` when ``statement`` is a ``mod name`` item with no body."""
    index = 0
    length = len(statement)
    while index < length:
        index = _skip_ws_comments(statement, index)
        if index >= length:
            return None
        skipped = _skip_attribute(statement, index)
        if skipped is not None:
            index = skipped
            continue
        match = _MOD.match(statement, index)
        if match is None:
            return None
        rest = _skip_ws_comments(statement, match.end())
        if rest == length:
            return match.group(1)
        return None
    return None


def _at_word(text: str, index: int, size: int) -> bool:
    if index > 0 and (text[index - 1].isalnum() or text[index - 1] == "_"):
        return False
    end = index + size
    if end < len(text) and (text[end].isalnum() or text[end] == "_"):
        return False
    return True


def _module_includes(text: str) -> list[str]:
    """Return every ``mod name;`` in ``text``, skipping comments and strings."""
    names: list[str] = []
    index = 0
    length = len(text)
    while index < length:
        if text.startswith("//", index):
            end = text.find("\n", index)
            index = length if end == -1 else end + 1
            continue
        if text.startswith("/*", index):
            end = text.find("*/", index + 2)
            index = length if end == -1 else end + 2
            continue
        char = text[index]
        if char in {'"', "'"}:
            index += 1
            while index < length and text[index] != char:
                if text[index] == "\\":
                    index += 2
                    continue
                index += 1
            index = min(index + 1, length)
            continue
        if text.startswith("mod", index) and _at_word(text, index, 3):
            cursor = _skip_ws_comments(text, index + 3)
            match = _IDENT.match(text, cursor)
            if match is not None:
                after = _skip_ws_comments(text, match.end())
                if after < length and text[after] == ";":
                    names.append(match.group(0))
                    index = after + 1
                    continue
        index += 1
    return names


def _analyze(text: str) -> tuple[list[str], list[str]]:
    """Return failures and ``mod name;`` items whose cfg requires ``test``."""
    failures: list[str] = []
    test_modules: list[str] = []
    index = 0
    length = len(text)
    line = 1
    line_start = 0
    depth = 0
    test_stack: list[int] = []
    pending_test = False
    cfg_origin = 0

    def finish_line(end: int) -> None:
        nonlocal line, line_start
        snippet = text[line_start:end]
        if CALL.search(snippet) and not test_stack and not ALLOW.search(snippet):
            failures.append(f"{line}:{snippet.strip()}")
        line += 1
        line_start = end + 1

    while index < length:
        if not pending_test and _cfg_requires_test_at(text, index):
            pending_test = True
            cfg_origin = index
        if text.startswith("//", index):
            end = text.find("\n", index)
            if end == -1:
                finish_line(length)
                break
            finish_line(end)
            index = end + 1
            continue
        if text.startswith("/*", index):
            end = text.find("*/", index + 2)
            if end == -1:
                end = length
            else:
                end += 2
            chunk = text[index:end]
            if CALL.search(chunk) and not test_stack and not ALLOW.search(chunk):
                failures.append(f"{line}:{chunk.strip()}")
            newlines = chunk.count("\n")
            line += newlines
            if newlines:
                line_start = index + chunk.rfind("\n") + 1
            index = end
            continue
        char = text[index]
        if char == '"':
            index += 1
            while index < length and text[index] != '"':
                if text[index] == "\\":
                    index += 2
                    continue
                if text[index] == "\n":
                    finish_line(index)
                index += 1
            index = min(index + 1, length)
            continue
        if char == "'":
            # Lifetimes and labels (`'static`, `'a`, `'outer:`) are not char
            # literals. Treating them as quotes hides a later `#[cfg(test)]`.
            nxt = index + 1
            if nxt < length and (text[nxt].isalpha() or text[nxt] == "_"):
                end = nxt + 1
                while end < length and (text[end].isalnum() or text[end] == "_"):
                    end += 1
                if end >= length or text[end] != "'":
                    index = end
                    continue
            index += 1
            while index < length and text[index] != "'":
                if text[index] == "\\":
                    index += 2
                    continue
                if text[index] == "\n":
                    finish_line(index)
                index += 1
            index = min(index + 1, length)
            continue
        if char == "{":
            depth += 1
            if pending_test:
                test_stack.append(depth)
                pending_test = False
        elif char == "}":
            if test_stack and test_stack[-1] == depth:
                test_stack.pop()
            depth = max(0, depth - 1)
        elif char == ";" and pending_test:
            declared = _declared_module(text[cfg_origin:index])
            if declared is not None:
                test_modules.append(declared)
            pending_test = False
        elif char == "\n":
            finish_line(index)
        index += 1
    if line_start < length:
        finish_line(length)
    return failures, test_modules


def scan(text: str) -> list[str]:
    """Return ``line:snippet`` failures in ``text``."""
    failures, _modules = _analyze(text)
    return failures


def _module_file(owner: pathlib.Path, name: str) -> pathlib.Path | None:
    """Resolve ``mod name;`` from ``owner`` to ``name.rs`` or ``name/mod.rs``."""
    if owner.name in {"lib.rs", "main.rs", "mod.rs"}:
        directory = owner.parent
    else:
        directory = owner.with_suffix("")
    for candidate in (directory / f"{name}.rs", directory / name / "mod.rs"):
        if candidate.is_file():
            return candidate.resolve()
    return None


def _exempt_module_files(files: list[pathlib.Path]) -> set[pathlib.Path]:
    """Module files compiled only when ``test`` is set, plus their children.

    ``#[cfg(all(test, feature = "test-util"))] mod tests;`` puts the banned-call
    scan of ``tests.rs`` in the same exemption as an inline ``#[cfg(test)]``
    module. The included file does not repeat that cfg.
    """
    exempt: set[pathlib.Path] = set()
    queue: list[pathlib.Path] = []
    for path in files:
        _failures, modules = _analyze(path.read_text(encoding="utf-8"))
        for name in modules:
            child = _module_file(path, name)
            if child is not None and child not in exempt:
                exempt.add(child)
                queue.append(child)
    while queue:
        parent = queue.pop()
        if not parent.is_file():
            continue
        for name in _module_includes(parent.read_text(encoding="utf-8")):
            child = _module_file(parent, name)
            if child is not None and child not in exempt:
                exempt.add(child)
                queue.append(child)
    return exempt


def scan_roots(roots: list[pathlib.Path]) -> list[str]:
    """Scan ``*.rs`` files under ``roots`` and return ``path:line:snippet``."""
    files: list[pathlib.Path] = []
    for root in roots:
        if not root.is_dir():
            continue
        files.extend(sorted(root.rglob("*.rs")))
    exempt = _exempt_module_files(files)
    found: list[str] = []
    for path in files:
        if path.resolve() in exempt:
            continue
        for failure in scan(path.read_text(encoding="utf-8")):
            found.append(f"{path}:{failure}")
    return found


def main(argv: list[str] | None = None) -> int:
    """Scan the default trees, or the paths in ``argv``."""
    args = list(sys.argv[1:] if argv is None else argv)
    roots = [pathlib.Path(arg) for arg in args] or [
        pathlib.Path("crates"),
        pathlib.Path("template"),
    ]
    found = scan_roots(roots)
    if found:
        print(
            "ADR-RUN-0011 point 1: banned call outside a cfg that requires test "
            "and a same-line allow",
            file=sys.stderr,
        )
        for failure in found:
            print(failure, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
