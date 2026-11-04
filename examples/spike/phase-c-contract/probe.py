#!/usr/bin/env python3
"""Temporary released-product qualification; never a network-dependent CI test."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

BUNDLE = Path('/tmp/c1-sc-lint-release/sc-lint_0.8.4_aarch64-apple-darwin')
BINARY = BUNDLE / 'bin/sc-lint'
ARCHIVE = BUNDLE.parent / 'sc-lint_0.8.4_aarch64-apple-darwin.tar.gz'
assert hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() == '798e245719cad2383650eb51dc7ccabd8eaa63e44c53733a4313a3a1cc69dbf2'
ENV = dict(os.environ, PATH=str(BUNDLE / 'bin') + os.pathsep + os.environ['PATH'], SC_LINT_BIN=str(BINARY), CARGO_BUILD_JOBS='2')

def run(root, *args):
    result = subprocess.run(args, cwd=root, env=ENV, text=True, capture_output=True)
    print(json.dumps({'argv': list(args), 'status': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr}))
    result.check_returncode()
    return result.stdout + result.stderr

with tempfile.TemporaryDirectory(prefix='phase-c-lint-') as directory:
    root = Path(directory)
    (root/'crates/api-types/src').mkdir(parents=True)
    (root/'Cargo.toml').write_text('[workspace]\nresolver="2"\nmembers=["crates/api-types"]\n[workspace.package]\nversion="0.1.0"\nedition="2024"\nlicense="MIT"\nauthors=["Consumer"]\nhomepage="https://example.com/consumer"\nrepository="https://example.com/consumer"\nrust-version="1.94.1"\n')
    (root/'crates/api-types/Cargo.toml').write_text('[package]\nname="probe-api-types"\nversion.workspace=true\nedition.workspace=true\nlicense.workspace=true\nauthors.workspace=true\nhomepage.workspace=true\nrepository.workspace=true\nrust-version.workspace=true\n')
    (root/'crates/api-types/src/lib.rs').write_text('//! Application DTOs.\n/// Application DTO.\npub struct Request;\n')
    run(root, str(BINARY), 'init', '--just')
    config = root/'sc-lint.toml'
    text = config.read_text()
    for name in ['sc-boundary','sc-portability','sc-runtime']:
        text += '\n[[tool.sc-lint.lint]]\nname = '+json.dumps(name)+'\ncommand = '+json.dumps(['sc-lint','lint',name,'--root','.','--config','sc-lint.toml'])+'\n'
    text += '\n[tool.sc-lint.lint_profiles]\ndefault=["fmt","clippy","sc-boundary","sc-portability","sc-runtime"]\nfull=["fmt","clippy","sc-boundary","sc-portability","sc-runtime"]\n'
    config.write_text(text.replace('minimum_version = "0.7.3"','minimum_version = "0.8.4"'))
    policy=root/'boundaries/probe-api-types/probe-api-types.toml'
    policy.parent.mkdir(parents=True)
    (root/'boundaries/planning.toml').write_text('[planning]\ncurrent_sprint="A.1"\n')
    policy.write_text('boundary_id="BOUNDARY-probe-api-types"\nowner_package="probe-api-types"\nowner_crate_path="probe_api_types"\nname="probe-api-types"\n[public]\nfacade="probe_api_types::Request"\n[implementation]\ntype="marker"\nmodule="probe_api_types"\nvisibility="public"\nconstructor="none"\n[composition]\nroots=["Request"]\n[dependencies]\nallowed_dependencies=[]\nallowed_dependents=[]\nforbidden_edges=[]\n[references]\nscope="outside_owner_crate"\nforbidden=[]\n[testing]\nallowed_test_double_paths=[]\nforbidden_test_bypasses=[]\n[enforcement]\nlint_rules=[]\nreview_gates=[]\n[status]\nstate="concrete_landed"\n')
    with (root/'Justfile').open('a') as stream:
        stream.write('\n# Consumer-owned recipe.\ndb-prepare:\n    @echo consumer-db-prepare\n')
    policy.write_text(policy.read_text()+'\n# Consumer-owned customization.\n')
    before=(config.read_bytes(),policy.read_bytes(),(root/'Justfile').read_bytes())
    run(root,str(BINARY),'--json','lint','sc-boundary','--root','.','--config','sc-lint.toml')
    for profile in ['', 'default', 'full']:
            output=run(root,'just','lint',*([profile] if profile else []))
            assert 'lint.ci.consumer: pass' in output, output
    receipt = run(root,str(BINARY),'--json','lint','ci','--consumer','--profile','full','--config','sc-lint.toml')
    Path('/tmp/c1-consumer-receipt.json').write_text(receipt)
    steps = {step['name']: step for step in json.loads(receipt)['data']['steps']}
    for name in ['sc-boundary', 'sc-portability', 'sc-runtime']:
        assert steps[name]['command'] == f'sc-lint lint {name} --root . --config sc-lint.toml'
        assert steps[name]['status'] == 'pass'

    for iteration in range(2):
        result = subprocess.run([str(BINARY), 'init', '--just'], cwd=root, env=ENV, text=True, capture_output=True)
        print(json.dumps({'argv':[str(BINARY),'init','--just'],'status':result.returncode,'stderr':result.stderr,'waived_acceptance':'https://github.com/randlee/sc-runtime/issues/117'}))
        assert result.returncode == 3 and 'CLI.SC_LINT_INTEGRATION_CONFLICT' in result.stderr
        assert before==(config.read_bytes(),policy.read_bytes(),(root/'Justfile').read_bytes())
        assert 'db-prepare:' in (root/'Justfile').read_text()
    print('LINT_PROFILES_POLICY_PRESERVATION_PASS; REPEATED_INIT_EXIT3_TEMPORARILY_WAIVED_117')
