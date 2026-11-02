"""store-boundaries compares only the store crates a layout rendered."""

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TEMPLATE_CRATES = ROOT / "template" / "crates"
STORES = ("store-sqlite", "store-postgres", "store-mysql")

_FUNCTIONS = {
    "store-sqlite": ("known_table", "widget_from_payload"),
    "store-postgres": ("known_table", "widget_from_payload"),
    "store-mysql": ("known_table", "widget_from_payload"),
}


def scanner():
    path = Path(__file__).with_name("store_boundaries.py")
    spec = importlib.util.spec_from_file_location("store_boundaries", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def link_stores(dest: Path, names: tuple[str, ...]) -> Path:
    crates = dest / "crates"
    crates.mkdir(parents=True)
    for name in names:
        (crates / name).symlink_to(TEMPLATE_CRATES / name, target_is_directory=True)
    return crates


def write_lib(crates: Path, crate: str, bodies: dict[str, str]) -> None:
    lib = crates / crate / "src"
    lib.mkdir(parents=True)
    text = "".join(f"fn {name}() {{ {body} }}\n" for name, body in bodies.items())
    (lib / "lib.rs").write_text(text, encoding="utf-8")


def matching_bodies(crate: str, body: str = "let ok = 1;") -> dict[str, str]:
    return {name: body for name in _FUNCTIONS[crate]}


class StoreBoundaryLayoutTests(unittest.TestCase):
    def setUp(self):
        self.checks = scanner()
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def test_template_workspace_still_compares_every_store(self):
        self.checks.store_boundaries(TEMPLATE_CRATES)

    def test_none_layout_requires_no_store_crate(self):
        crates = self.root / "crates"
        crates.mkdir()
        (crates / "api-types").mkdir()
        self.assertFalse((crates / "store-sqlite").exists())
        self.checks.store_boundaries(crates)

    def test_sqlite_mysql_omits_store_postgres(self):
        crates = link_stores(self.root, ("store-sqlite", "store-mysql"))
        self.assertFalse((crates / "store-postgres").exists())
        self.checks.store_boundaries(crates)

    def test_one_store_layouts_omit_the_other_stores(self):
        for name in STORES:
            with self.subTest(name=name):
                crates = link_stores(self.root / name, (name,))
                self.checks.store_boundaries(crates)

    def test_sqlite_postgres_omits_store_mysql(self):
        crates = link_stores(self.root, ("store-sqlite", "store-postgres"))
        self.assertFalse((crates / "store-mysql").exists())
        self.checks.store_boundaries(crates)

    def test_rendered_pair_still_fails_when_it_drifts(self):
        crates = self.root / "crates"
        write_lib(crates, "store-sqlite", matching_bodies("store-sqlite"))
        mysql = matching_bodies("store-mysql")
        mysql["known_table"] = "let drifted = 2;"
        write_lib(crates, "store-mysql", mysql)
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            with self.assertRaises(SystemExit):
                self.checks.store_boundaries(crates)
        self.assertIn(
            "store-boundaries: fn known_table in store-mysql drifted from store-sqlite",
            stderr.getvalue(),
        )

    def test_drift_uses_the_first_rendered_crate_when_sqlite_is_absent(self):
        crates = self.root / "crates"
        write_lib(crates, "store-postgres", matching_bodies("store-postgres"))
        mysql = matching_bodies("store-mysql")
        mysql["known_table"] = "let drifted = 2;"
        write_lib(crates, "store-mysql", mysql)
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            with self.assertRaises(SystemExit):
                self.checks.store_boundaries(crates)
        self.assertIn(
            "fn known_table in store-mysql drifted from store-postgres",
            stderr.getvalue(),
        )
        self.assertNotIn("missing", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
