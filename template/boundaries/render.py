#!/usr/bin/env python3
"""Generate fresh consumer inventory once, after selected packages are prefixed.

This generator is removed by the generation hook. Setup and upgrade never
reconstruct the inventory: its contents belong to the generated application.
"""
import json
from pathlib import Path
import sys
import tomllib

FACADES = {
    'api-types': 'Widget', 'schema': 'EngineStore', 'service': 'Stores',
    'daemon': 'instance_root', 'cli': 'Command', 'store-sqlite': 'Store',
    'store-postgres': 'Store', 'store-mysql': 'Store', 'store-adaptor': 'TableFlags',
}


def render(root):
    root = Path(root).resolve()
    manifests = {}
    for path in sorted((root / 'crates').glob('*/Cargo.toml')):
        with path.open('rb') as handle:
            manifests[path.parent.name] = tomllib.load(handle)
    packages = {role: data['package']['name'] for role, data in manifests.items()}
    edges = {role: set() for role in manifests}
    for role, data in manifests.items():
        # Include test edges too: CLI's integration tests use daemon/service.
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for name, spec in data.get(section, {}).items():
                if isinstance(spec, dict) and 'path' in spec:
                    target = (root / 'crates' / role / spec['path']).resolve().name
                    if target in packages:
                        edges[role].add(target)
    directory = root / 'boundaries'
    if list(directory.rglob('*.toml')):
        raise FileExistsError('fresh boundary generation refuses to overwrite consumer policy')
    (directory / 'planning.toml').write_text('[planning]\ncurrent_sprint = "A.1"\n', encoding='utf-8')
    for role, data in manifests.items():
        package = packages[role]
        crate = data.get('lib', {}).get('name', package.replace('-', '_'))
        dependencies = sorted(packages[target] for target in edges[role])
        dependents = sorted(packages[source] for source in edges if role in edges[source])
        text = f'''# Initial selected application inventory; consumer-owned after generation.
boundary_id = "BOUNDARY-{package}"
owner_package = "{package}"
owner_crate_path = "{crate}"
name = "{package}"
[public]
facade = "{crate}::{FACADES[role]}"
[implementation]
type = "marker"
module = "{crate}"
visibility = "public"
constructor = "none"
[composition]
roots = ["{FACADES[role]}"]
[dependencies]
allowed_dependencies = {json.dumps(dependencies)}
allowed_dependents = {json.dumps(dependents)}
forbidden_edges = []
[references]
scope = "outside_owner_crate"
forbidden = []
[testing]
allowed_test_double_paths = []
forbidden_test_bypasses = []
[enforcement]
lint_rules = []
review_gates = []
[status]
state = "concrete_landed"
'''
        destination = directory / package / f'{package}.toml'
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(text, encoding='utf-8')


if __name__ == '__main__':
    render(sys.argv[1] if len(sys.argv) == 2 else '.')
