"""Package prefix and the single rendered adaptor directory."""

import importlib.util
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]

ROLES = {
    "api-types": "acme-types",
    "schema": "acme-schema",
    "store-sqlite": "acme-store-sqlite",
    "store-postgres": "acme-store-postgres",
    "store-mysql": "acme-store-mysql",
    "store-adaptor": "acme-store-adaptor",
    "service": "acme-service",
    "daemon": "acme-daemon",
    "cli": "acme-cli",
}


def prefix():
    path = ROOT / "template" / "hooks" / "prefix_packages.py"
    previous = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        spec = importlib.util.spec_from_file_location("prefix_packages", path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module
    finally:
        sys.dont_write_bytecode = previous
        cache = path.parent / "__pycache__"
        if cache.is_dir():
            import shutil

            shutil.rmtree(cache)


class PrefixPackageTests(unittest.TestCase):
    def test_acme_package_names(self):
        hook = prefix()
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp)
            for role in ROLES:
                crate = dest / "crates" / role
                crate.mkdir(parents=True)
                (crate / "src").mkdir()
                (crate / "src" / "lib.rs").write_text("\n", encoding="utf-8")
                (crate / "Cargo.toml").write_text(
                    textwrap.dedent(
                        f"""\
                        [package]
                        name = "{role}"
                        version = "0.1.0"

                        [dependencies]
                        schema = {{ path = "../schema" }}
                        """
                    ),
                    encoding="utf-8",
                )
            hook.prefix_tree("acme", dest)
            for role, package in ROLES.items():
                text = (dest / "crates" / role / "Cargo.toml").read_text(encoding="utf-8")
                self.assertIn(f'name = "{package}"', text, role)
                if role != "schema":
                    self.assertIn('package = "acme-schema"', text, role)

    def test_two_store_render_is_one_adaptor_directory(self):
        hook = prefix()
        template = ROOT / "template" / "crates" / "store-adaptor"
        for db in ("sqlite-postgres", "sqlite-mysql"):
            with self.subTest(db=db):
                with tempfile.TemporaryDirectory() as tmp:
                    dest = Path(tmp)
                    crate = dest / "crates" / "store-adaptor"
                    (crate / "src" / "engines").mkdir(parents=True)
                    for name in (
                        "Cargo.toml",
                        "Cargo.sqlite-postgres.toml",
                        "Cargo.sqlite-mysql.toml",
                        "src/lib.rs",
                        "src/shared.rs",
                        "src/membership.rs",
                    ):
                        source = template / name
                        target = crate / name
                        target.parent.mkdir(parents=True, exist_ok=True)
                        target.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")
                    for engine in ("sqlite-postgres.rs", "sqlite-mysql.rs"):
                        source = template / "src" / "engines" / engine
                        (crate / "src" / "engines" / engine).write_text(
                            source.read_text(encoding="utf-8"),
                            encoding="utf-8",
                        )
                    sidecar = dest / "crates" / f"store-adaptor-{db}" / "src"
                    sidecar.mkdir(parents=True)
                    (sidecar / "lib.rs").write_text("sidecar\n", encoding="utf-8")
                    hook.prepare_adaptor(dest, db)
                    hook.prefix_tree("acme", dest)
                    packages = [
                        path
                        for path in (dest / "crates").iterdir()
                        if path.is_dir() and (path / "Cargo.toml").is_file()
                    ]
                    self.assertEqual([path.name for path in packages], ["store-adaptor"])
                    rendered = crate / "src" / "lib.rs"
                    self.assertEqual(rendered.read_text(encoding="utf-8"), hook._RENDERED_LIB)
                    engine = (crate / "src" / "engine.rs").read_text(encoding="utf-8")
                    engine_name = db.removeprefix("sqlite-")
                    self.assertIn(f'REMOTE_ENGINE: &str = "{engine_name}"', engine)
                    self.assertFalse((crate / "src" / "engines").exists())
                    self.assertFalse((crate / "Cargo.sqlite-postgres.toml").exists())
                    self.assertFalse((crate / "Cargo.sqlite-mysql.toml").exists())
                    self.assertFalse((dest / "crates" / f"store-adaptor-{db}").exists())
                    manifest = (crate / "Cargo.toml").read_text(encoding="utf-8")
                    self.assertIn('name = "acme-store-adaptor"', manifest)
                    self.assertIn("[lib]", manifest)
                    self.assertIn('name = "store_adaptor"', manifest)


class AdaptorSourceTests(unittest.TestCase):
    def test_membership_decoder_is_one_file(self):
        functions = []
        shapes = []
        for path in (ROOT / "template").rglob("*.rs"):
            text = path.read_text(encoding="utf-8")
            if "fn membership_from_toml" in text:
                functions.append(path)
            if "struct MembershipFile" in text:
                shapes.append(path)
        self.assertEqual(
            functions,
            [ROOT / "template/crates/schema/tests/common/membership_toml.rs"],
        )
        self.assertEqual(shapes, functions)

    def test_two_store_ignores_do_not_keep_an_adaptor_sidecar(self):
        import tomllib

        cargo = tomllib.loads(
            (ROOT / "template" / "cargo-generate.toml").read_text(encoding="utf-8")
        )
        for key in ('db == "sqlite-postgres"', 'db == "sqlite-mysql"'):
            ignored = cargo["conditional"][key]["ignore"]
            self.assertIn("crates/store-adaptor-sqlite-postgres", ignored)
            self.assertIn("crates/store-adaptor-sqlite-mysql", ignored)
            self.assertNotIn("crates/store-adaptor", ignored)
            self.assertNotIn("crates/store-adaptor/src", ignored)
            self.assertNotIn("crates/store-adaptor/Cargo.toml", ignored)

    def test_workspace_check_covers_every_db_value(self):
        import json

        workflow = (ROOT / ".github" / "workflows" / "template.yml").read_text(encoding="utf-8")
        self.assertIn("SQLX_OFFLINE=true cargo check --workspace", workflow)
        values = set()
        for path in (ROOT / "wizard" / "fixtures").glob("*.json"):
            values.add(json.loads(path.read_text(encoding="utf-8"))["db"])
        self.assertEqual(
            values,
            {"none", "sqlite", "postgres", "mysql", "sqlite-postgres", "sqlite-mysql"},
        )


if __name__ == "__main__":
    unittest.main()
