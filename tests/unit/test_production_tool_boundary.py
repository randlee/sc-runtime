"""Reject tool Rust packages/imports in every supported production graph.

The boundary companion resolves package identities, not dependency aliases.
Dev-only dependency/import fixtures are accepted; external transitive edges
are checked by Cargo's normal/build trees rather than workspace-only lint.
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
import adr_run_0011_grep as module_context

RELEASE = ("sc-config", "sc-transport", "sc-command", "sc-runtime", "sc-cron")
FEATURES = {"sc-config": (None,), "sc-transport": (None, "server"),
            "sc-command": (None, "server"), "sc-runtime": (None,),
            "sc-cron": (None, "monty")}


def tool_package(name):
    return name in {"sc-compose", "agent-team-mail", "agent-team-mail-core"} or name.startswith(("sc-compose-", "atm-"))


def production_dependencies(manifest, workspace):
    tables = [manifest]
    tables.extend(manifest.get("target", {}).values())
    for table in tables:
        for kind in ("dependencies", "build-dependencies"):
            for alias, spec in table.get(kind, {}).items():
                if isinstance(spec, dict) and spec.get("workspace"):
                    spec = workspace.get(alias, spec)
                yield spec.get("package", alias) if isinstance(spec, dict) else alias


def cfg_possibilities_without_test(expression):
    """Conservatively evaluate cfg with test=false and other predicates unknown."""
    expression = expression.strip()
    if expression == "test":
        return False, True
    call = re.fullmatch(r"(all|any|not)\s*\((.*)\)", expression, re.S)
    if not call:
        return True, True
    pieces, start, depth = [], 0, 0
    args = call[2]
    for index, char in enumerate(args):
        depth += (char == "(") - (char == ")")
        if char == "," and depth == 0:
            pieces.append(args[start:index]); start = index + 1
    if args[start:].strip():
        pieces.append(args[start:])
    values = [cfg_possibilities_without_test(piece) for piece in pieces]
    if call[1] == "not":
        return (values[0][1], values[0][0]) if len(values) == 1 else (True, True)
    if call[1] == "all":
        return all(value[0] for value in values), any(value[1] for value in values)
    return any(value[0] for value in values), all(value[1] for value in values)


def rust_production(source):
    # Remove lexical comments/string literals before recognizing paths. Preserve
    # delimiters elsewhere so test-only modules can be excluded by balanced braces.
    source = re.sub(r'r(\#*)".*?"\1|"(?:\\.|[^"\\])*"|/\*.*?\*/|//[^\n]*', lambda match: " " * len(match[0]), source, flags=re.S)
    marker = re.compile(r'#\s*\[\s*cfg\s*\(')
    offset = 0
    while match := marker.search(source, offset):
        start, end, depth = match.start(), match.end(), 1
        while end < len(source) and depth:
            depth += (source[end] == '(') - (source[end] == ')')
            end += 1
        expression = source[match.end():end - 1]
        close = re.match(r'\s*\]', source[end:])
        if close is None:
            offset = end
            continue
        end += close.end()
        if cfg_possibilities_without_test(expression)[0]:
            offset = end
            continue
        rest = source[end:]
        item = re.search(r'[;{]', rest)
        if item is None:
            source = source[:start]
            break
        end += item.end()
        if item[0] == "{":
            depth = 1
            while end < len(source) and depth:
                depth += (source[end] == "{") - (source[end] == "}")
                end += 1
        source = source[:start] + " " * (end - start) + source[end:]
        offset = end
    return source


def source_violations(source):
    code = rust_production(source)
    names = re.findall(r'\b([A-Za-z_]\w*)\s*::|\bextern\s+crate\s+([A-Za-z_]\w*)', code)
    return sorted({name for pair in names for name in pair if name and
                   (tool_package(name.replace("_", "-")) or name == "atm_core")})


def production_path_targets(source, production, parent, candidates):
    """Collect possible #[path] targets; unknown targets cannot be exempted."""
    targets = set()
    index = 0
    while index < len(production):
        start = production.find('#', index)
        if start < 0:
            break
        end = module_context._skip_attribute(production, start)
        if end is None:
            index = start + 1
            continue
        masked = production[start:end]
        path = re.search(r'\bpath\s*=', masked)
        if path:
            # cfg_attr(test, ...) cannot introduce a production path.
            conditional = re.match(r'#\s*\[\s*cfg_attr\s*\(', masked)
            parsed = module_context._parse_predicate(masked, conditional.end()) if conditional else None
            if parsed and not cfg_possibilities_without_test(masked[conditional.end():parsed[1]])[0]:
                index = end
                continue
            following = module_context._skip_ws_comments(source, end)
            if module_context._cfg_requires_test_at(source, following):
                index = end
                continue
            original = source[start + path.end():end]
            literal = re.match(r'\s*"([^"\\]*)"', original)
            if literal is None:
                return set(candidates)
            relative = Path(literal[1])
            targets.add((parent.parent / relative).resolve())
            # Inline modules change the base directory. Match possible candidate
            # suffixes conservatively when the enclosing module is unknown.
            prefix = production[:start]
            if prefix.count('{') > prefix.count('}'):
                targets.update(candidate for candidate in candidates
                               if candidate.parts[-len(relative.parts):] == relative.parts)
        index = end
    return targets


def test_only_module_files(files):
    """Keep only test-context modules with no production-reachable path."""
    exempt = module_context._exempt_module_files(files)
    queue = [path.resolve() for path in files if path.resolve() not in exempt]
    seen = set(queue)
    while queue:
        parent = queue.pop()
        source = parent.read_text()
        production = rust_production(source)
        for child in production_path_targets(source, production, parent, exempt):
            if child.is_file() and child not in seen:
                seen.add(child)
                queue.append(child)
        for name in module_context._module_includes(production):
            child = module_context._module_file(parent, name)
            if child is not None and child not in seen:
                seen.add(child)
                queue.append(child)
    return exempt - seen


def scan(root):
    workspace = tomllib.loads((root / "Cargo.toml").read_text()).get("workspace", {}).get("dependencies", {})
    violations = []
    for crate in RELEASE:
        folder = root / "crates" / crate
        manifest = tomllib.loads((folder / "Cargo.toml").read_text())
        for package in production_dependencies(manifest, workspace):
            if tool_package(package):
                violations.append(f"{crate}: production dependency {package}")
        files = list((folder / "src").rglob("*.rs"))
        exempt = test_only_module_files(files)
        for path in files:
            if path.resolve() in exempt:
                continue
            for name in source_violations(path.read_text()):
                violations.append(f"{path.relative_to(root)}: production Rust tool reference {name}")
    return violations


def tree_violations(text):
    return sorted({line.split()[0] for line in text.splitlines() if line.split() and tool_package(line.split()[0])})


def check_trees(root):
    violations = []
    for package, features in FEATURES.items():
        for feature in features:
            cmd = ["cargo"] + (["+1.96.1"] if feature == "monty" else [])
            cmd += ["tree", "-p", package, "-e", "normal,build", "--prefix", "none"]
            if feature:
                cmd += ["--features", feature]
            result = subprocess.run(cmd, cwd=root, capture_output=True, text=True, check=False)
            if result.returncode:
                violations.append(f"{package}/{feature or 'default'} tree failed: {result.stderr.strip()}")
            violations.extend(f"{package}/{feature or 'default'} links {name}" for name in tree_violations(result.stdout))
    return violations


class ProductionToolBoundaryTests(unittest.TestCase):
    def test_production_manifest_package_identities(self):
        for kind in ("dependencies", "build-dependencies"):
            for name in ("sc-compose", "sc-compose-render", "agent-team-mail", "agent-team-mail-core", "atm-client"):
                manifest = {kind: {"innocent_alias": {"package": name, "version": "1", "optional": True}}}
                self.assertTrue(any(tool_package(p) for p in production_dependencies(manifest, {})))
        target = {"target": {"cfg(unix)": {"build-dependencies": {"alias": {"package": "atm-client"}}}}}
        self.assertIn("atm-client", list(production_dependencies(target, {})))
        inherited = {"dependencies": {"alias": {"workspace": True}}}
        self.assertIn("sc-compose", list(production_dependencies(inherited, {"alias": {"package": "sc-compose"}})))

    def test_dev_only_manifest_and_imports_are_accepted(self):
        manifest = {"dev-dependencies": {"alias": {"package": "agent-team-mail-core"}}}
        self.assertEqual(list(production_dependencies(manifest, {})), [])
        self.assertEqual(source_violations('#[cfg(test)] mod tests { use atm_core::X; fn nested() { sc_compose::run(); } }\npub fn x() {}'), [])
        self.assertEqual(source_violations('#[cfg(test)] use atm_core::X;\npub fn x() {}'), [])
        self.assertEqual(source_violations('let program = "sc-compose"; // use atm_core::X;\n'), [])

    def test_nested_cfg_test_only_allowance_does_not_allow_production_branches(self):
        for cfg in ('all(test, unix)', 'any(all(test, unix), all(test, windows))', 'not(not(test))'):
            self.assertEqual(source_violations(f'#[cfg({cfg})] mod tests {{ use atm_core::X; }}'), [], cfg)
        for cfg in ('not(test)', 'any(test, unix)', 'all(unix, not(test))'):
            self.assertEqual(source_violations(f'#[cfg({cfg})] use atm_core::X;'), ['atm_core'], cfg)

    def test_production_rust_imports_and_fully_qualified_paths_rejected(self):
        for source in ('use atm_core::X;', 'extern crate atm_core;', 'fn x() { sc_compose::run(); }', 'use agent_team_mail_core::X;'):
            self.assertTrue(source_violations(source), source)

    def test_transitive_tree_checks_package_names(self):
        self.assertEqual(tree_violations('sc-runtime v0.1.1\ninnocent v1.0\nagent-team-mail-core v1.0\n'), ['agent-team-mail-core'])
        self.assertEqual(tree_violations('sc-cron v0.1.1\nmonty v1.4.0\n'), [])

    def test_real_fixture_manifests_reject_production_and_allow_dev_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/*"]\n')
            for name in RELEASE:
                crate = root / "crates" / name
                (crate / "src").mkdir(parents=True)
                (crate / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion = "0.1.1"\n')
                (crate / "src/lib.rs").write_text('pub fn ok() {}\n')
            manifest = root / "crates/sc-config/Cargo.toml"
            base = manifest.read_text()
            manifest.write_text(base + '[dependencies]\ninnocent = { package = "sc-compose-render", version = "1", optional = true }\n')
            self.assertTrue(any("sc-compose-render" in error for error in scan(root)))
            manifest.write_text(base + '[build-dependencies]\ninnocent = { package = "agent-team-mail-core", version = "1" }\n')
            self.assertTrue(any("agent-team-mail-core" in error for error in scan(root)))
            manifest.write_text(base + '[dev-dependencies]\ninnocent = { package = "agent-team-mail-core", version = "1" }\n')
            source = root / "crates/sc-config/src/lib.rs"
            source.write_text('#[cfg(test)] mod tests { use atm_core::X; }\npub fn ok() {}\n')
            self.assertEqual(scan(root), [])
            source.write_text('use atm_core::X;\n')
            self.assertTrue(any("production Rust tool reference atm_core" in error for error in scan(root)))

    def test_out_of_line_test_module_and_production_reachability(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text('[workspace]\nmembers=["crates/*"]\nexclude=["tool/atm-core"]\n')
            for name in RELEASE:
                crate = root / "crates" / name
                (crate / "src").mkdir(parents=True)
                (crate / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.1.1"\n')
                (crate / "src/lib.rs").write_text('pub fn ok() {}\n')
            crate = root / "crates/sc-config"
            with (crate / "Cargo.toml").open("a") as manifest:
                manifest.write('[dev-dependencies]\natm_core = {package="agent-team-mail-core", path="../../tool/atm-core"}\n')
            tool = root / "tool/atm-core"
            (tool / "src").mkdir(parents=True)
            (tool / "Cargo.toml").write_text('[package]\nname="agent-team-mail-core"\nversion="1.0.0"\n[lib]\nname="atm_core"\n')
            (tool / "src/lib.rs").write_text('pub struct X;\n')
            source = crate / "src/lib.rs"
            (crate / "src/test_support.rs").write_text('use atm_core::X;\n#[test]\nfn test_only() { let _ = X; }\n')
            source.write_text('#[cfg(test)]\nmod test_support;\npub fn ok() {}\n')
            self.assertEqual(scan(root), [])
            for declarations in ('mod test_support;', '#[cfg(test)] mod test_support;\nmod test_support;'):
                source.write_text(declarations + '\n')
                self.assertTrue(any('test_support.rs: production Rust tool reference atm_core' in error for error in scan(root)))
            test_decl = '#[cfg(test)] mod test_support;\n'
            (crate / "src/production.rs").write_text('pub fn production() {}\n')
            for declaration in ('#[path="test_support.rs"] mod production_support;',
                                '#[cfg_attr(unix, path="test_support.rs")] mod production_support;',
                                'mod nested { #[path="test_support.rs"] mod production_support; }',
                                '#[path=concat!("test", "_support.rs")] mod production_support;'):
                source.write_text(test_decl + declaration + '\n')
                self.assertTrue(any('test_support.rs: production Rust tool reference atm_core' in error for error in scan(root)), declaration)
            for declaration in ('#[path="production.rs"] mod production;',
                                '#[cfg_attr(unix, path="production.rs")] mod production;',
                                '#[cfg_attr(test, path="test_support.rs")] mod production;',
                                '#[cfg_attr(all(test, unix), path="test_support.rs")] mod production;',
                                '#[path="test_support.rs"] #[cfg(test)] mod other_test;'):
                source.write_text(test_decl + declaration + '\n')
                self.assertEqual(scan(root), [], declaration)

    def test_actual_release_crates(self):
        self.assertEqual(scan(ROOT), [])


if __name__ == "__main__":
    if "--check" in sys.argv:
        parser = argparse.ArgumentParser()
        parser.add_argument("--check", action="store_true")
        parser.add_argument("--tree", action="store_true")
        args = parser.parse_args()
        errors = scan(ROOT) + (check_trees(ROOT) if args.tree else [])
        if errors:
            print("\n".join(errors), file=sys.stderr)
            raise SystemExit(1)
        print("production tool boundary: PASS (all five release crates)")
    else:
        unittest.main()
