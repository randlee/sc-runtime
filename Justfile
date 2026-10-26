# Managed by sc-lint; regenerate with `sc-lint init --just`.
set windows-shell := ["cmd.exe", "/c"]

python := if os_family() == "windows" { "python" } else { "python3" }

# The `full` profile runs boundary, line-counts, fmt, clippy
# (`-D warnings` and `-D clippy::await_holding_lock`), sc-portability,
# sc-runtime, function-length, identity-literals, compatibility, and the
# Windows cross-check.
#
# Workspace form kept for later sprints, when every member cross-compiles:
#   cargo xwin check --workspace --target x86_64-pc-windows-msvc
#   cargo xwin clippy --workspace --all-targets --target x86_64-pc-windows-msvc -- -D warnings -D clippy::await_holding_lock
# This sprint's profile runs those two commands package-scoped to sc-config,
# sc-transport, sc-command, and sc-runtime.
default:
    {{python}} .sc-lint/bootstrap.py lint full --config sc-lint.toml

setup:
    {{python}} .sc-lint/bootstrap.py setup --config sc-lint.toml

lint *profile:
    {{python}} .sc-lint/bootstrap.py lint --config sc-lint.toml {{profile}}
    just adr-run-0011
    just supply-chain

# ADR-RUN-0011 point 1 until sc-lint#640. A match outside #[cfg(test)] and
# outside a same-line `// allow: <reason>` comment fails. yield_now is not a
# match. A bare sleep(...) after a use import is not a match.
adr-run-0011:
    #!/usr/bin/env python3
    import pathlib
    import re
    import sys

    CALL = re.compile(
        r"std::thread::sleep_until|std::thread::sleep(?!_)|"
        r"tokio::time::sleep_until|tokio::time::sleep(?!_)|"
        r"Runtime::block_on|Handle::block_on|futures::executor::block_on"
    )
    CFG_TEST = re.compile(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
    ALLOW = re.compile(r"\ballow\b\s*:\s*\S")

    def scan(text):
        failures = []
        i = 0
        n = len(text)
        line = 1
        line_start = 0
        depth = 0
        test_stack = []
        pending_test = False

        def finish_line(end):
            nonlocal line, line_start
            snippet = text[line_start:end]
            if CALL.search(snippet) and not test_stack and not ALLOW.search(snippet):
                failures.append(f"{line}:{snippet.strip()}")
            line += 1
            line_start = end + 1

        while i < n:
            if CFG_TEST.match(text, i):
                pending_test = True
            if text.startswith("//", i):
                end = text.find("\n", i)
                if end == -1:
                    finish_line(n)
                    break
                finish_line(end)
                i = end + 1
                continue
            if text.startswith("/*", i):
                end = text.find("*/", i + 2)
                if end == -1:
                    end = n
                else:
                    end += 2
                chunk = text[i:end]
                if CALL.search(chunk) and not test_stack and not ALLOW.search(chunk):
                    failures.append(f"{line}:{chunk.strip()}")
                newlines = chunk.count("\n")
                line += newlines
                if newlines:
                    line_start = i + chunk.rfind("\n") + 1
                i = end
                continue
            char = text[i]
            if char == '"':
                i += 1
                while i < n and text[i] != '"':
                    if text[i] == "\\":
                        i += 2
                        continue
                    if text[i] == "\n":
                        finish_line(i)
                    i += 1
                i = min(i + 1, n)
                continue
            if char == "'":
                i += 1
                while i < n and text[i] != "'":
                    if text[i] == "\\":
                        i += 2
                        continue
                    if text[i] == "\n":
                        finish_line(i)
                    i += 1
                i = min(i + 1, n)
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
                finish_line(i)
            i += 1
        if line_start < n:
            finish_line(n)
        return failures

    roots = [pathlib.Path("crates"), pathlib.Path("template")]
    found = []
    for root in roots:
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.rs")):
            for failure in scan(path.read_text(encoding="utf-8")):
                found.append(f"{path}:{failure}")
    if found:
        print("ADR-RUN-0011 point 1: banned call outside #[cfg(test)] and a same-line allow", file=sys.stderr)
        for failure in found:
            print(failure, file=sys.stderr)
        sys.exit(1)

test *layer:
    {{python}} .sc-lint/bootstrap.py test --config sc-lint.toml {{layer}}

upgrade:
    {{python}} .sc-lint/bootstrap.py upgrade --config sc-lint.toml

# Repo-specific (not regenerated by sc-lint init).

# License, advisory, and duplicate checks. `just lint` always runs this.
supply-chain:
    cargo deny check
    cargo audit

test-generation:
    {{python}} scripts/run_generation_tests.py

new dest *args:
    {{python}} scripts/new_project.py {{dest}} {{args}}
