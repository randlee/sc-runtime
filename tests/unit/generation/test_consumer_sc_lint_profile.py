"""Exercise generated policy and released sc-lint through public consumer recipes."""
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch

from tests.integration.generation.generate_owned import cargo_generate, run_generation

ROOT = Path(__file__).resolve().parents[3]
ANALYZERS = ('sc-boundary', 'sc-portability', 'sc-runtime')
MEMBERS = ('fmt', 'clippy', *ANALYZERS)


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def assert_receipts(test, output):
    if 'lint.ci.consumer: pass (5 steps)' in output:
        return
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
    def setUp(self):
        self.state = Path(self.enterContext(tempfile.TemporaryDirectory(
            prefix='consumer-install-', dir=self.base)))
        self.env = self.environment(self.state)

    def environment(self, state):
        """Give each regression an independent installation and writable caches."""
        state = Path(state)
        state.mkdir(parents=True, exist_ok=True)
        original_home = Path.home()
        env = dict(os.environ)
        env.pop('SC_LINT_BIN', None)
        env.pop('SC_LINT_RELEASE_BASE_URL', None)
        env.update({
            'HOME': str(state / 'home'), 'USERPROFILE': str(state / 'home'),
            'XDG_DATA_HOME': str(state / 'data'), 'XDG_CONFIG_HOME': str(state / 'config'),
            'XDG_CACHE_HOME': str(state / 'cache'), 'LOCALAPPDATA': str(state / 'local'),
            'APPDATA': str(state / 'roaming'), 'UV_CACHE_DIR': str(state / 'uv-cache'),
            'PIP_CACHE_DIR': str(state / 'pip-cache'),
            'SC_LINT_INSTALL_DIR': str(state / 'install' / 'bin'),
            'CARGO_HOME': str(state / 'cargo'), 'CARGO_TARGET_DIR': str(state / 'target'),
            'RUSTUP_HOME': os.environ.get('RUSTUP_HOME', str(original_home / '.rustup')),
            'CARGO_BUILD_JOBS': os.environ.get('CARGO_BUILD_JOBS', '2'),
        })
        for name in ('HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME',
                     'LOCALAPPDATA', 'APPDATA', 'UV_CACHE_DIR', 'PIP_CACHE_DIR', 'CARGO_HOME'):
            Path(env[name]).mkdir(parents=True, exist_ok=True)
        # A deliberate incompatible PATH installation makes setup prove a fresh
        # managed install even on hosts already running 0.8.6. It is never used
        # for analyzer acceptance and must remain byte-for-byte unchanged.
        ambient = state / 'ambient'
        ambient.mkdir()
        script = ambient / 'incompatible.py'
        script.write_text("import sys\nprint('incompatible test launcher', file=sys.stderr)\nsys.exit(78)\n")
        launcher = ambient / ('sc-lint.cmd' if os.name == 'nt' else 'sc-lint')
        if os.name == 'nt':
            launcher.write_text(f'@"{sys.executable}" "{script}" %*\n')
        else:
            launcher.write_text(f'#!{sys.executable}\n'+script.read_text())
            launcher.chmod(0o755)
        (ambient / 'consumer-owned-sentinel').write_text('keep this installation unchanged\n')
        env['PATH'] = str(ambient) + os.pathsep + env.get('PATH', '')
        return env

    def execute(self, root, argv, *, env=None):
        return run_generation(argv, cwd=root, env=self.env if env is None else env,
                              timeout=600)


    def command(self, root, *argv, expected=0):
        result = self.execute(root, argv)
        output = result.stdout + result.stderr
        print(f'consumer proof: {" ".join(argv)}\n{output}', flush=True)
        self.assertEqual(result.returncode, expected, output)
        return output

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
            cargo_generate(ROOT, destination, answers['project_name'], values, env=self.env)
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
        result = self.execute(root, [sys.executable, str(ROOT / 'template/boundaries/render.py'), str(root)])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('refuses to overwrite consumer policy', result.stderr)
        self.assertEqual(policy.read_bytes(), before)

    def test_public_profiles_and_preservation(self):
        root = self.generate('sqlite-mcp.json')
        tracked = {p: p.read_bytes() for p in root.rglob('*') if p.is_file() and '.git' not in p.parts and 'target' not in p.parts}
        self.assertFalse(Path(self.env['SC_LINT_INSTALL_DIR']).exists())
        ambient = self.state / 'ambient'
        ambient_before = {p: p.read_bytes() for p in ambient.iterdir()}
        self.command(root, 'just', 'setup')
        for path, before in tracked.items():
            self.assertEqual(path.read_bytes(), before, str(path))
        bootstrap = module(root / '.sc-lint/bootstrap.py', 'generated_consumer_bootstrap')
        with patch.dict(os.environ, self.env, clear=True):
            managed = bootstrap.managed_path()
            binary, override = bootstrap.resolve_binary(managed)
        self.assertEqual(binary, managed)
        self.assertFalse(override)
        self.assertTrue(binary.is_file())
        self.assertTrue(binary.is_relative_to(self.state / 'install'))
        version = json.loads(self.command(root, str(binary), 'version', '--json'))
        self.assertEqual(version['data']['version'], '0.8.6')
        for profile in ((), ('default',), ('full',)):
            assert_receipts(self, self.command(root, 'just', 'lint', *profile))
        receipt = json.loads(self.command(
            root, str(binary), '--json', 'lint', '--consumer', '--config',
            'sc-lint.toml', 'ci', '--profile', 'full'))
        self.assertEqual(receipt['data']['status'], 'pass')
        self.assertEqual(
            [step['name'] for step in receipt['data']['steps']], list(MEMBERS))
        for step in receipt['data']['steps']:
            self.assertEqual(step['status'], 'pass', step)
        for name in ANALYZERS:
            step = next(step for step in receipt['data']['steps'] if step['name'] == name)
            self.assertEqual(step['command'],
                f'sc-lint lint {name} --root . --config sc-lint.toml')
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
            result = self.execute(root, ['just', 'lint', 'full'])
            result.stdout += result.stderr
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
        result = self.execute(root, [str(binary), '--json', 'lint', 'sc-boundary', '--root', '.', '--config', 'sc-lint.toml'])
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('package_dependency_not_allowed', result.stdout + result.stderr)
        print('consumer proof: removing an allowed edge failed the actual boundary analyzer', flush=True)
        policy.write_text(before)
        store_manifest = root / 'crates/store-sqlite/Cargo.toml'
        store_before = store_manifest.read_text()
        types_package = tomllib.loads((root / 'crates/api-types/Cargo.toml').read_text())['package']['name']
        store_manifest.write_text(store_before.replace('[dependencies]',
            '[dependencies]\napi-types = { path = "../api-types", package = "' + types_package + '", version = "0.1.1" }'))
        forbidden = self.execute(root, [str(binary), '--json', 'lint', 'sc-boundary', '--root', '.', '--config', 'sc-lint.toml'])
        self.assertNotEqual(forbidden.returncode, 0, forbidden.stdout + forbidden.stderr)
        self.assertIn('package_dependency_not_allowed', forbidden.stdout + forbidden.stderr)
        store_manifest.write_text(store_before)
        print('consumer proof: adding a forbidden dependency failed the actual boundary analyzer', flush=True)
        policy.write_text(before + '\n# Consumer-owned policy customization.\n')
        justfile = root / 'Justfile'
        justfile.write_text(justfile.read_text() + '\nextra-db-prepare:\n    @echo SDK-owned preparation\n')
        custom_config = config.read_text()
        self.assertIn('full = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]', custom_config)
        config.write_text(custom_config.replace(
            'full = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]',
            'full = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]\n'
            'sdk = ["fmt", "clippy", "sc-boundary", "sc-portability", "sc-runtime"]'))
        preserved = {p: p.read_bytes() for p in (config, policy, root / 'Justfile')}
        operations = (
            ('just', 'setup'),
            ('just', 'upgrade'),
            (str(binary), 'init', '--just'),
            (str(binary), 'init', '--just'),
        )
        for operation in operations:
            self.command(root, *operation)
            assert_receipts(self, self.command(root, 'just', 'lint', 'sdk'))
            for path, contents in preserved.items():
                self.assertEqual(path.read_bytes(), contents, f'{operation}: {path}')
            profiles = tomllib.loads(config.read_text())['tool']['sc-lint']['lint_profiles']
            self.assertEqual(tuple(profiles['sdk']), MEMBERS)
            self.assertIn('Consumer-owned policy customization', policy.read_text())
            listed = self.command(root, 'just', '--list')
            self.assertIn('extra-db-prepare', listed)
            self.assertIn('SDK-owned preparation', self.command(root, 'just', 'extra-db-prepare'))
        self.assertTrue((root / 'boundaries').is_dir())
        sdk_receipt = json.loads(self.command(
            root, str(binary), '--json', 'lint', '--consumer', '--config',
            'sc-lint.toml', 'ci', '--profile', 'sdk'))
        self.assertEqual(sdk_receipt['data']['status'], 'pass')
        self.assertEqual([step['name'] for step in sdk_receipt['data']['steps']], list(MEMBERS))
        for name in ANALYZERS:
            step = next(step for step in sdk_receipt['data']['steps'] if step['name'] == name)
            self.assertEqual(step['status'], 'pass', step)
            self.assertEqual(step['command'], f'sc-lint lint {name} --root . --config sc-lint.toml')
        for path, contents in ambient_before.items():
            self.assertEqual(path.read_bytes(), contents, str(path))
        other = self.environment(self.state / 'second-instance')
        other_root = self.generate('none.json')
        first_bytes = binary.read_bytes()
        for operation in ('setup', 'upgrade'):
            installed = self.execute(other_root, ['just', operation], env=other)
            self.assertEqual(installed.returncode, 0, installed.stdout + installed.stderr)
        other_binary = Path(other['SC_LINT_INSTALL_DIR']) / binary.name
        self.assertTrue(other_binary.is_file())
        self.assertNotEqual(other_binary.parent, binary.parent)
        self.assertEqual(binary.read_bytes(), first_bytes)
        for path, contents in ambient_before.items():
            self.assertEqual(path.read_bytes(), contents, str(path))


if __name__ == '__main__':
    unittest.main()
