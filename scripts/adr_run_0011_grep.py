#!/usr/bin/env python3
"""ADR-RUN-0011 point 1 scanner.

Fails when production Rust under ``crates/`` or ``template/`` calls a banned
sleep or ``block_on`` form outside ``#[cfg(test)]`` and outside a same-line
``allow:`` comment. ``yield_now`` is not a match. ``just adr-run-0011`` runs
this file with the platform ``python`` command.
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
CFG_TEST = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
ALLOW = re.compile(r"\ballow\b\s*:\s*\S")


def scan(text: str) -> list[str]:
    """Return ``line:snippet`` failures in ``text``."""
    failures: list[str] = []
    index = 0
    length = len(text)
    line = 1
    line_start = 0
    depth = 0
    test_stack: list[int] = []
    pending_test = False

    def finish_line(end: int) -> None:
        nonlocal line, line_start
        snippet = text[line_start:end]
        if CALL.search(snippet) and not test_stack and not ALLOW.search(snippet):
            failures.append(f"{line}:{snippet.strip()}")
        line += 1
        line_start = end + 1

    while index < length:
        if CFG_TEST.match(text, index):
            pending_test = True
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
            pending_test = False
        elif char == "\n":
            finish_line(index)
        index += 1
    if line_start < length:
        finish_line(length)
    return failures


def scan_roots(roots: list[pathlib.Path]) -> list[str]:
    """Scan ``*.rs`` files under ``roots`` and return ``path:line:snippet``."""
    found: list[str] = []
    for root in roots:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.rs")):
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
            "ADR-RUN-0011 point 1: banned call outside #[cfg(test)] and a same-line allow",
            file=sys.stderr,
        )
        for failure in found:
            print(failure, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
