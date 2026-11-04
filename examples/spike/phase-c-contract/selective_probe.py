#!/usr/bin/env python3
"""Qualify the exact pinned toolkit without publishing or rewriting workflows."""
import json
from pathlib import Path
import subprocess
import tempfile

REPO = Path(__file__).resolve().parents[3]
SCRIPT = Path('/tmp/c1-sc-publish-pinned/plugins/sc-publish/.github/scripts/release_artifacts.py')
with tempfile.TemporaryDirectory(prefix='phase-c-selective-') as directory:
    root = Path(directory)
    (root/'Cargo.toml').write_text('[workspace]\n[workspace.package]\nversion="0.2.0"\n')
    # Existing canonical generated manifest shape; only package matrix changes.
    manifest = 'schema_version=1\n[project]\nname="probe"\narchive_prefix="probe"\n'
    for name, version, publish, order in [
        ('sc-config', '0.1.1', False, 1),
        ('sc-transport', '0.1.1', False, 2),
        ('sc-command', '0.2.0', True, 3),
        ('sc-cron', '0.2.0', True, 4),
        ('sc-runtime', '0.2.0', True, 5),
        ('sc-cron-commands', '0.2.0', True, 6),
    ]:
        (root/name).mkdir()
        (root/name/'Cargo.toml').write_text(f'[package]\nname="{name}"\nversion="{version}"\n')
        manifest += f'\n[[crates]]\nartifact="{name}"\npackage="{name}"\ncargo_toml="{name}/Cargo.toml"\npublish={str(publish).lower()}\npublish_order={order}\nwait_after_publish_seconds=0\n'
    (root/'manifest.toml').write_text(manifest)
    for command, extra in [
        ('list-publish-plan', []),
        ('verify-version', ['--workspace-toml', 'Cargo.toml', '--version', '0.2.0']),
    ]:
        argv = ['python3', str(SCRIPT), command, '--manifest', 'manifest.toml', *extra]
        result = subprocess.run(argv, cwd=root, text=True, capture_output=True)
        print(json.dumps(dict(argv=argv, status=result.returncode, stdout=result.stdout, stderr=result.stderr)))
        result.check_returncode()
    print('SELECTIVE_RELEASE_PASS')
