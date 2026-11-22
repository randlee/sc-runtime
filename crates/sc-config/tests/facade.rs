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

/// Every exported name is reachable from outside the crate, and `ConfigError`
/// can only be matched with a wildcard arm (REQ-CFG-0009 criterion 2).
#[test]
fn layered_exports_are_usable_from_outside_the_crate() {
    use sc_config::{
        After, ConfigError, EnvSource, FileSource, KeyPath, Layer, LayerSource, Layers, Provenance,
        Resolved, ResolvedRow, Slot, Source, SourceKind, StandardContext, StandardLayers,
        ValueSource,
    };

    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Layers>();
    assert_send_sync::<FileSource>();
    assert_send_sync::<EnvSource>();
    assert_send_sync::<ValueSource>();
    assert_send_sync::<StandardLayers>();
    let _: Option<Box<dyn LayerSource>> = None;
    fn exported<T>() {}
    exported::<Layer>();
    exported::<Resolved>();
    exported::<Provenance>();
    exported::<ResolvedRow>();
    exported::<KeyPath>();
    exported::<Source>();
    exported::<SourceKind>();
    exported::<StandardContext>();
    assert_ne!(After::User, After::Env);
    assert_ne!(Slot::User, Slot::Flags);

    let error = ConfigError::DuplicateSlot { slot: Slot::User };
    let named = match error {
        ConfigError::DuplicateSlot { .. } => "duplicate",
        _ => "other",
    };
    assert_eq!(named, "duplicate");
}
