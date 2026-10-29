"""Liquid in Rust sources stays under the line budget."""

import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def conditional_lines(text: str) -> list[str]:
    inside = False
    chosen = []
    for line in text.splitlines():
        if "{%" in line or inside:
            chosen.append(line)
        if "{% if" in line or "{% else" in line:
            inside = True
        if "{% endif" in line:
            inside = False
    return chosen


class LiquidTests(unittest.TestCase):
    def test_rust_conditionals_are_small_and_do_not_vary_bodies(self):
        lines = []
        for path in sorted((ROOT / "template").rglob("*.rs")):
            lines.extend(conditional_lines(path.read_text(encoding="utf-8")))
        self.assertLess(len(lines), 10)
        bodies = []
        for line in lines:
            text = line
            while "{%" in text and "%}" in text:
                start = text.index("{%")
                end = text.index("%}", start) + 2
                text = text[:start] + text[end:]
            if text.strip():
                bodies.append(text.strip())
        self.assertEqual(
            bodies,
            ["pub mod mcp;", ".mcp(daemon::mcp::service)"],
        )

    def test_whole_file_differences_are_ignored(self):
        cargo = tomllib.loads((ROOT / "template" / "cargo-generate.toml").read_text(encoding="utf-8"))
        ignored = cargo["conditional"]["mcp == false"]["ignore"]
        self.assertIn("crates/daemon/src/mcp.rs", ignored)
        self.assertIn("crates/cli/tests/widget_mcp.rs", ignored)
        self.assertEqual(
            cargo["template"]["exclude"],
            ["AGENTS.md.j2", "CLAUDE.md.j2"],
        )
        self.assertEqual(
            cargo["template"]["ignore"],
            ["config/cron.schema.json"],
        )


if __name__ == "__main__":
    unittest.main()
