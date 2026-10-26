"""Unit tests for the ADR-RUN-0011 point 1 scanner."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import adr_run_0011_grep as scanner  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]


class ScanTests(unittest.TestCase):
    def test_sleep_and_block_on_forms_fail(self) -> None:
        text = "\n".join(
            [
                "fn hot() {",
                "    std::thread::sleep(std::time::Duration::from_millis(1));",
                "    std::thread::sleep_until(std::time::Instant::now());",
                "    tokio::time::sleep(std::time::Duration::from_millis(1));",
                "    tokio::time::sleep_until(tokio::time::Instant::now());",
                "    Runtime::block_on(work());",
                "    Handle::block_on(work());",
                "    futures::executor::block_on(work());",
                "    runtime.block_on(work());",
                "}",
            ]
        )
        failures = scanner.scan(text)
        self.assertEqual(len(failures), 8)

    def test_yield_now_and_timeout_are_not_matches(self) -> None:
        text = "\n".join(
            [
                "fn hot() {",
                "    tokio::task::yield_now().await;",
                "    let _ = tokio::time::timeout(std::time::Duration::from_millis(1), work()).await;",
                "    let _ = tokio::time::timeout_at(deadline, handler).await;",
                "}",
            ]
        )
        self.assertEqual(scanner.scan(text), [])

    def test_cfg_test_is_skipped(self) -> None:
        text = "\n".join(
            [
                "#[cfg(test)]",
                "mod tests {",
                "    fn probe() {",
                "        runtime.block_on(work());",
                "        tokio::time::sleep(std::time::Duration::from_millis(1));",
                "    }",
                "}",
                "fn hot() { tokio::task::yield_now(); }",
            ]
        )
        self.assertEqual(scanner.scan(text), [])

    def test_same_line_allow_is_skipped(self) -> None:
        text = "runtime.block_on(work()); // allow: NFR-RUN-0002 fixture thread\n"
        self.assertEqual(scanner.scan(text), [])

    def test_block_on_without_allow_fails(self) -> None:
        failures = scanner.scan("fn hot() { runtime.block_on(work()); }\n")
        self.assertEqual(len(failures), 1)
        self.assertIn(".block_on(", failures[0])

    def test_allow_without_reason_still_fails(self) -> None:
        failures = scanner.scan("runtime.block_on(work()); // allow:\n")
        self.assertEqual(len(failures), 1)

    def test_scan_roots_reports_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "src" / "hot.rs"
            source.parent.mkdir()
            source.write_text("fn hot() { runtime.block_on(work()); }\n", encoding="utf-8")
            found = scanner.scan_roots([root])
        self.assertEqual(len(found), 1)
        self.assertIn("hot.rs", found[0])
        self.assertIn(".block_on(", found[0])

    def test_fixture_block_on_has_same_line_allow(self) -> None:
        text = (ROOT / "crates/sc-runtime/src/testing.rs").read_text(encoding="utf-8")
        self.assertEqual(scanner.scan(text), [])

    def test_owned_trees_are_clean(self) -> None:
        self.assertEqual(
            scanner.scan_roots([ROOT / "crates", ROOT / "template"]),
            [],
        )


if __name__ == "__main__":
    unittest.main()
