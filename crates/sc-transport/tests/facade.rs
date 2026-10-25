//! The boundary facade is the complete non-hidden crate-root export set.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn facade_matches_crate_root_exports() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap();
    let exports = visible_exports(&source);
    let manifests = [manifest_dir.join("../../boundaries/sc-transport/sc-transport.toml")];
    for path in manifests {
        let text =
            fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let facade = quoted_list(&text, "facade");
        let roots = bracket_list(&text, "roots");
        assert_eq!(facade, exports, "{}", path.display());
        assert_eq!(roots, exports, "{}", path.display());
        assert!(!facade.contains("ListenerIo"), "{}", path.display());
        assert!(!facade.contains("ListenerAddr"), "{}", path.display());
        assert!(
            !facade.contains("instance_root_from_base"),
            "{}",
            path.display()
        );
    }
}

fn visible_exports(src: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut offset = 0;
    while let Some(rel) = src[offset..].find("pub use ") {
        let start = offset + rel;
        let window_start = start.saturating_sub(160);
        let hidden = src[window_start..start].contains("doc(hidden)");
        let after = &src[start + "pub use ".len()..];
        let end = after
            .find(';')
            .unwrap_or_else(|| panic!("pub use without semicolon"));
        let stmt = &after[..end];
        offset = start + "pub use ".len() + end + 1;
        if hidden {
            continue;
        }
        if let Some(open) = stmt.find('{') {
            let close = stmt
                .rfind('}')
                .unwrap_or_else(|| panic!("unclosed pub use"));
            for part in stmt[open + 1..close].split(',') {
                let name = part.split("//").next().unwrap_or("").trim();
                if !name.is_empty() {
                    names.insert(name.to_owned());
                }
            }
        } else {
            let name = stmt.rsplit("::").next().unwrap_or("").trim();
            assert!(!name.is_empty(), "{stmt}");
            names.insert(name.to_owned());
        }
    }
    names
}

fn quoted_list(text: &str, key: &str) -> BTreeSet<String> {
    let prefix = format!("{key} = \"");
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with(&prefix))
        .unwrap_or_else(|| panic!("missing {key}"));
    let value = line.split_once('=').unwrap().1.trim();
    let value = value.trim_matches('"');
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn bracket_list(text: &str, key: &str) -> BTreeSet<String> {
    let marker = format!("{key} = [");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {key}"));
    let rest = &text[start + marker.len()..];
    let end = rest.find(']').unwrap_or_else(|| panic!("unclosed {key}"));
    rest[..end]
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}
