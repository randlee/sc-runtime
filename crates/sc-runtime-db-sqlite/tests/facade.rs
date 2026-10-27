use std::collections::BTreeSet;

#[test]
fn facade_lists_every_export() {
    let lib_rs = include_str!("../src/lib.rs");
    let manifest =
        include_str!("../../../boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml");
    let exports = exported_names(lib_rs);
    let facade = facade_names(manifest);
    let roots = composition_roots(manifest);
    assert_eq!(
        exports, facade,
        "boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml [public] facade must list every pub export"
    );
    assert_eq!(
        exports, roots,
        "boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml [composition] roots must list every pub export"
    );
}

fn exported_names(lib_rs: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut lines = lib_rs.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            let name = rest
                .trim_end_matches(';')
                .split_whitespace()
                .next()
                .expect("module name");
            names.insert(name.to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("pub const ") {
            if rest.starts_with("fn ") {
                continue;
            }
            let name = rest
                .split([':', '=', ' '])
                .next()
                .expect("const name")
                .trim();
            if !name.is_empty() {
                names.insert(name.to_owned());
            }
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("pub use ") else {
            continue;
        };
        let mut chunk = rest.to_owned();
        if chunk.contains('{') && !chunk.contains('}') {
            for next in lines.by_ref() {
                chunk.push(' ');
                chunk.push_str(next.trim());
                if next.contains('}') {
                    break;
                }
            }
        }
        let chunk = chunk.trim_end_matches(';').trim().to_owned();
        if let Some((_path, list)) = chunk.split_once("::{") {
            let list = list.trim_end_matches('}');
            for name in list.split(',') {
                let name = name.trim();
                if !name.is_empty() {
                    names.insert(name.to_owned());
                }
            }
        } else {
            let name = chunk.rsplit("::").next().unwrap_or(&chunk);
            names.insert(name.to_owned());
        }
    }
    names
}

fn facade_names(manifest: &str) -> BTreeSet<String> {
    let line = manifest
        .lines()
        .find(|line| line.trim_start().starts_with("facade = "))
        .expect("facade");
    let value = line
        .split_once('=')
        .expect("facade assignment")
        .1
        .trim()
        .trim_matches('"');
    value
        .split(',')
        .map(|item| {
            item.trim()
                .rsplit("::")
                .next()
                .expect("facade path")
                .to_owned()
        })
        .filter(|item| !item.is_empty())
        .collect()
}

fn composition_roots(manifest: &str) -> BTreeSet<String> {
    let line = manifest
        .lines()
        .find(|line| line.trim_start().starts_with("roots = "))
        .expect("roots");
    let value = line.split_once('=').expect("roots assignment").1.trim();
    value
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}
