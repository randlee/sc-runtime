use std::collections::BTreeSet;

#[test]
fn facade_lists_every_export() {
    let lib_rs = include_str!("../src/lib.rs");
    let manifest = include_str!("../../../boundaries/sc-config/sc-config.toml");
    let exports = exported_names(lib_rs);
    let facade = facade_names(manifest);
    let roots = composition_roots(manifest);
    assert_eq!(
        exports, facade,
        "boundaries/sc-config/sc-config.toml [public] facade must list every pub use"
    );
    assert_eq!(
        exports, roots,
        "boundaries/sc-config/sc-config.toml [composition] roots must list every pub use"
    );
}

fn exported_names(lib_rs: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in lib_rs.lines() {
        let Some(rest) = line.trim().strip_prefix("pub use ") else {
            continue;
        };
        let rest = rest.trim_end_matches(';').trim();
        if let Some((_path, list)) = rest.split_once("::{") {
            let list = list.trim_end_matches('}');
            for name in list.split(',') {
                names.insert(name.trim().to_owned());
            }
        } else {
            let name = rest.rsplit("::").next().unwrap_or(rest);
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
