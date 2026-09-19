#!/usr/bin/env python3
"""Validate REQ / NFR / ADR entries against req-adr-format.md.

Usage: validate_req_adr.py [repo_root]   (exit 1 on any error)
"""
import re
import sys
from pathlib import Path

REQ_DEF = re.compile(r"^- `((?:REQ|NFR)-[A-Z0-9]+-\d{3}[A-Z]*)` \*\*(.+?)\*\*", re.M)
ADR_DEF = re.compile(r"^### (ADR-(?:[A-Z0-9]+-)?\d{3}) (.+)$", re.M)
ANY_REF = re.compile(r"\b((?:REQ|NFR)-[A-Z0-9]+-\d{3}[A-Z]*|ADR-(?:[A-Z0-9]+-)?\d{3})\b")
RANGE = re.compile(r"\b((?:REQ|NFR)-[A-Z0-9]+-)(\d{3})\.\.(\d{3})\b")
ADR_PARTS = ["| Status |", "| Date |", "| Relates to |", "**Context.**",
             "**Decision.**", "**Consequences.**", "**Rejected.**", "**Enforced by.**"]
STATUS = re.compile(r"\| Status \| (proposed|accepted|superseded by ADR-[A-Z0-9-]+)")


def blocks(text, pattern):
    hits = list(pattern.finditer(text))
    for i, m in enumerate(hits):
        end = hits[i + 1].start() if i + 1 < len(hits) else len(text)
        nxt = re.search(r"^## ", text[m.end():end], re.M)
        yield m, text[m.start():m.end() + nxt.start() if nxt else end]


def main():
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    docs = root / "docs"
    errors, defined, refs = [], {}, []
    files = [docs / "requirements.md", docs / "architecture.md"]
    crates = sorted(p.name for p in (root / "crates").glob("*") if p.is_dir())
    for c in crates:
        files += [docs / c / "requirements.md", docs / c / "architecture.md"]
    for f in files:
        if not f.is_file():
            errors.append(f"{f}: required file is missing")
            continue
        text = f.read_text()
        rel = f.relative_to(root)
        for m, body in blocks(text, REQ_DEF):
            rid = m.group(1)
            if f.name != "requirements.md":
                errors.append(f"{rel}: {rid} defined outside a requirements file")
            for part in ("**Why:**", "**Verified by:**"):
                if part not in body:
                    errors.append(f"{rel}: {rid} has no {part}")
            if rid in defined:
                errors.append(f"{rel}: {rid} already defined in {defined[rid]}")
            defined[rid] = rel
        if f.name == "architecture.md" and "\n## ADR" not in text:
            errors.append(f"{rel}: no '## ADR' heading")
        for m, body in blocks(text, ADR_DEF):
            aid = m.group(1)
            for part in ADR_PARTS:
                if part not in body:
                    errors.append(f"{rel}: {aid} has no {part}")
            if not STATUS.search(body):
                errors.append(f"{rel}: {aid} has no valid Status")
            if aid in defined:
                errors.append(f"{rel}: {aid} already defined in {defined[aid]}")
            defined[aid] = rel
        for m in RANGE.finditer(text):
            refs += [(rel, f"{m.group(1)}{n:03d}") for n in range(int(m.group(2)), int(m.group(3)) + 1)]
        refs += [(rel, r) for r in ANY_REF.findall(text)]
    external = set(sys.argv[2:])
    for rel, r in sorted(set(refs)):
        if r not in defined and r not in external:
            errors.append(f"{rel}: reference to undefined id {r}")
    for e in errors:
        print(f"error: {e}")
    print(f"{len(defined)} ids defined in {len(files)} files, {len(errors)} errors")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
