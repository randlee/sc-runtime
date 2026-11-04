"""Exercise generated policy and released sc-lint through public consumer recipes."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[3]
ANALYZERS = ('sc-boundary', 'sc-portability', 'sc-runtime')
MEMBERS = ('fmt', 'clippy', *ANALYZERS)


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def assert_receipts(test, output):
    for name in MEMBERS:
        test.assertIn(f'lint.{name}: running ', output)
        test.assertIn(f'lint.{name}: ok', output)
    for name in ANALYZERS:
        test.assertIn(f'lint {name} --root . --config sc-lint.toml', output)


class ConsumerProfileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix='consumer-sc-lint-')
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.base = Path(cls.temporary.name)
        cls.driver = module(ROOT / 'scripts/new_project.py', 'consumer_profile_driver')
        cls.env = dict(os.environ, CARGO_BUILD_JOBS=os.environ.get('CARGO_BUILD_JOBS', '2'))

    def command(self, root, *argv, expected=0):
        result = subprocess.run(argv, cwd=root, env=self.env, text=True, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, check=False)
        print(f'consumer proof: {" ".join(argv)}\n{result.stdout}', flush=True)
        self.assertEqual(result.returncode, expected, result.stdout)
        return result.stdout

    def generate(self, fixture, mcp=None):
        suffix = '' if mcp is None else f'-mcp-{mcp}'
        destination = self.base / (fixture.removesuffix('.json') + suffix)
        if destination.exists():
            return destination
        answers = json.loads((ROOT / 'wizard/fixtures' / fixture).read_text())
        if mcp is not None:
            answers['mcp'] = mcp
        schema = self.driver.load_schema(ROOT)
        answers = self.driver.validate(schema, answers)
        values = self.driver.write_values(answers)
        try:
            self.driver.cargo_generate(ROOT, destination, answers['project_name'], values)
            self.driver.apply_checkout_patch(destination, ROOT)
        finally:
            values.unlink()
        return destination

    def test_selected_policy_inventory(self):
        for fixture, mcp in ((fixture, mcp) for fixture in ('sqlite-mcp.json', 'sqlite-postgres.json', 'sqlite-mysql.json', 'none.json') for mcp in (False, True)):
            with self.subTest(fixture=fixture, mcp=mcp):
                root = self.generate(fixture, mcp)
                manifests = [tomllib.loads(p.read_text()) for p in (root / 'crates').glob('*/Cargo.toml')]
                packages = {data['package']['name'] for data in manifests}
                policies = [tomllib.loads(p.read_text()) for p in (root / 'boundaries').glob('*/*.toml')]
                self.assertEqual({p['owner_package'] for p in policies}, packages)
                for policy in policies:
                    self.assertTrue(set(policy['dependencies']['allowed_dependencies']) <= packages)
                    self.assertTrue(set(policy['dependencies']['allowed_dependents']) <= packages)
                roles = {p.parent.name: tomllib.loads(p.read_text())['package']['name']
                         for p in (root / 'crates').glob('*/Cargo.toml')}
                service = next(p for p in policies if p['owner_package'] == roles['service'])
                expected = {roles['api-types']}
                if 'schema' in roles:
                    expected.add(roles['schema'])
                    store = 'store-adaptor' if 'store-adaptor' in roles else 'store-sqlite'
                    expected.add(roles[store])
                self.assertEqual(set(service['dependencies']['allowed_dependencies']), expected)
                self.assertEqual((root / 'crates/daemon/src/mcp.rs').exists(), mcp)
                for prohibited in ('clippy.toml', 'rustfmt.toml', 'deny.toml'):
                    self.assertFalse(list(root.rglob(prohibited)))
                self.assertFalse((root / 'boundaries/render.py').exists())

    def test_policy_render_refuses_existing_consumer_bytes(self):
        root = self.generate('sqlite-no-mcp.json')
        policy = next((root / 'boundaries').glob('*service/*.toml'))
        policy.write_text(policy.read_text() + "\n# Application-owned policy edit.\n")
        before = policy.read_bytes()
        result = subprocess.run(['python3', str(ROOT / 'template/boundaries/render.py'), str(root)],
                                text=True, capture_output=True, check=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('refuses to overwrite consumer policy', result.stderr)
        self.assertEqual(policy.read_bytes(), before)

    def test_public_profiles_and_preservation(self):
        root = self.generate('sqlite-mcp.json')
        tracked = {p: p.read_bytes() for p in root.rglob('*') if p.is_file() and '.git' not in p.parts and 'target' not in p.parts}
        self.command(root, 'just', 'setup')
        for path, before in tracked.items():
            self.assertEqual(path.read_bytes(), before, str(path))
        bootstrap = module(root / '.sc-lint/bootstrap.py', 'generated_consumer_bootstrap')
        binary, _ = bootstrap.resolve_binary(bootstrap.managed_path())
        self.assertIsNotNone(binary)
        version = json.loads(self.command(root, str(binary), 'version', '--json'))
        self.assertEqual(version['data']['version'], '0.8.4')
        for profile in ((), ('default',), ('full',)):
            assert_receipts(self, self.command(root, 'just', 'lint', *profile))
        listed = self.command(root, 'just', '--list')
        for recipe in ('setup', 'lint', 'test', 'upgrade', 'db-prepare'):
            self.assertIn(recipe, listed)
        config = root / 'sc-lint.toml'
        original = config.read_text()
        for mutation in ('member', 'command'):
            if mutation == 'member':
                changed = original.replace(', "sc-runtime"]', ']')
            else:
                changed = original.replace('name = "sc-runtime"', 'name = "missing-runtime"')
            config.write_text(changed)
            result = subprocess.run(['just', 'lint', 'full'], cwd=root, env=self.env, text=True,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
            # A missing member may run a shorter profile successfully, but must
            # fail the execution proof. A missing declaration fails dispatch.
            with self.assertRaises(AssertionError):
                assert_receipts(self, result.stdout)
            if mutation == 'command':
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertIn('PROFILE.UNKNOWN_MEMBER', result.stdout)
            config.write_text(original)
        policy = next((root / 'boundaries').glob('*service/*.toml'))
        before = policy.read_text()
        parsed = tomllib.loads(before)
        self.assertTrue(parsed['dependencies']['allowed_dependencies'])
        policy.write_text(before.replace('allowed_dependencies = '+json.dumps(parsed['dependencies']['allowed_dependencies']), 'allowed_dependencies = []'))
        result = subprocess.run([str(binary), '--json', 'lint', 'sc-boundary', '--root', '.', '--config', 'sc-lint.toml'],
                                cwd=root, env=self.env, text=True, capture_output=True, check=False)
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('package_dependency_not_allowed', result.stdout + result.stderr)
        print('consumer proof: removing an allowed edge failed the actual boundary analyzer', flush=True)
        policy.write_text(before)
        store_manifest = root / 'crates/store-sqlite/Cargo.toml'
        store_before = store_manifest.read_text()
        types_package = tomllib.loads((root / 'crates/api-types/Cargo.toml').read_text())['package']['name']
        store_manifest.write_text(store_before.replace('[dependencies]',
            '[dependencies]\napi-types = { path = "../api-types", package = "' + types_package + '", version = "0.1.1" }'))
        forbidden = subprocess.run([str(binary), '--json', 'lint', 'sc-boundary', '--root', '.', '--config', 'sc-lint.toml'],
                                   cwd=root, env=self.env, text=True, capture_output=True, check=False)
        self.assertNotEqual(forbidden.returncode, 0, forbidden.stdout + forbidden.stderr)
        self.assertIn('package_dependency_not_allowed', forbidden.stdout + forbidden.stderr)
        store_manifest.write_text(store_before)
        print('consumer proof: adding a forbidden dependency failed the actual boundary analyzer', flush=True)
        policy.write_text(before + '\n# Consumer-owned policy customization.\n')
        preserved = {p: p.read_bytes() for p in (config, policy, root / 'Justfile')}
        self.command(root, 'just', 'setup')
        self.command(root, 'just', 'upgrade')
        init = self.command(root, str(binary), 'init', '--just', expected=3)
        self.assertIn('CLI.SC_LINT_INTEGRATION_CONFLICT', init)
        print('repeat-init exit 3: temporarily waived by sc-runtime#117; bytes preserved', flush=True)
        for path, contents in preserved.items():
            self.assertEqual(path.read_bytes(), contents, str(path))


if __name__ == '__main__':
    unittest.main()
