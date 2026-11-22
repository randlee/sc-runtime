//! Engine, provenance and `Loader` preset tests: REQ-CFG-0005, 0007 and 0008.

#[path = "support/mod.rs"]
mod support;

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use sc_config::{
    AppName, ConfigError, KeyPath, Layer, LayerSource, Layers, Loader, Source, SourceKind,
    ValueSource,
};
use serde::{Deserialize, Deserializer};
use serde_json::{Value, json};
use support::{TempDir, environment, write_file};

fn val(value: Value) -> ValueSource {
    ValueSource::new(Source::default("test"), value)
}

fn resolved(layers: &Layers) -> Value {
    layers.resolve().unwrap().value
}

#[test]
fn precedence_is_push_order() {
    let forward = Layers::new()
        .push(val(json!({"a": 1, "b": 1})))
        .push(val(json!({"a": 2})));
    assert_eq!(resolved(&forward), json!({"a": 2, "b": 1}));
    let reverse = Layers::new()
        .push(val(json!({"a": 2})))
        .push(val(json!({"a": 1, "b": 1})));
    assert_eq!(resolved(&reverse), json!({"a": 1, "b": 1}));
}

/// Implements only `source` and `load`: the trait has no other required method.
struct Fixed(Option<Layer>);

impl LayerSource for Fixed {
    fn source(&self) -> Source {
        Source::custom("keychain", "login")
    }

    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        Ok(self.0.clone())
    }
}

fn fixed(value: Value) -> Fixed {
    Fixed(Some(Layer::new(Source::custom("keychain", "login"), value)))
}

#[test]
fn a_consumer_source_contributes_its_value() {
    let layers = Layers::new()
        .push(val(json!({"a": 1})))
        .push(fixed(json!({"b": 2})));
    assert_eq!(resolved(&layers), json!({"a": 1, "b": 2}));
}

#[test]
fn an_absent_source_changes_nothing() {
    let layers = Layers::new().push(val(json!({"a": 1}))).push(Fixed(None));
    let result = layers.resolve().unwrap();
    assert_eq!(result.value, json!({"a": 1}));
    assert_eq!(result.provenance.rows().len(), 1);
}

struct Counting {
    calls: Arc<AtomicUsize>,
    fail: bool,
}

impl LayerSource for Counting {
    fn source(&self) -> Source {
        Source::custom("count", "n")
    }

    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            Err(ConfigError::source_failed(self.source(), "boom"))
        } else {
            Ok(None)
        }
    }
}

#[test]
fn the_first_error_is_returned_and_later_sources_are_not_called() {
    let below = Arc::new(AtomicUsize::new(0));
    let above = Arc::new(AtomicUsize::new(0));
    let layers = Layers::new()
        .push(Counting {
            calls: below.clone(),
            fail: true,
        })
        .push(Counting {
            calls: above.clone(),
            fail: false,
        });
    let error = layers.load::<Value>().unwrap_err();
    assert_eq!(error.code(), "CFG.SOURCE");
    let text = error.to_string();
    assert!(text.contains("count n"), "{text}");
    assert!(text.contains("boom"), "{text}");
    assert_eq!(below.load(Ordering::SeqCst), 1);
    assert_eq!(above.load(Ordering::SeqCst), 0);
}

struct SeesBase(Arc<Mutex<Option<Value>>>);

impl LayerSource for SeesBase {
    fn source(&self) -> Source {
        Source::custom("probe", "p")
    }

    fn load(&self, base: &Value) -> Result<Option<Layer>, ConfigError> {
        *self.0.lock().unwrap() = Some(base.clone());
        Ok(None)
    }
}

#[test]
fn a_source_sees_the_value_below_it() {
    let seen = Arc::new(Mutex::new(None));
    let layers = Layers::new()
        .push(val(json!({"a": 1})))
        .push(val(json!({"b": 2})))
        .push(SeesBase(seen.clone()));
    layers.resolve().unwrap();
    assert_eq!(*seen.lock().unwrap(), Some(json!({"a": 1, "b": 2})));
}

#[test]
fn replace_at_replaces_instead_of_merging() {
    let upper =
        Layer::new(Source::custom("keychain", "login"), json!({"a": {"c": 1}})).replace_at(["a"]);
    let layers = Layers::new()
        .push(val(json!({"a": {"b": 1}})))
        .push(Fixed(Some(upper)));
    assert_eq!(resolved(&layers), json!({"a": {"c": 1}}));
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
struct Defaulted {
    port: u16,
}

#[test]
fn an_empty_stack_loads_a_default_struct() {
    let loaded: Defaulted = Layers::new().load().unwrap();
    assert_eq!(loaded, Defaulted::default());
}

thread_local! {
    /// Each test runs on its own thread, so this counter is private to one test.
    static DECODES: Cell<usize> = const { Cell::new(0) };
}

struct Counted;

impl<'de> Deserialize<'de> for Counted {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        DECODES.with(|count| count.set(count.get() + 1));
        Value::deserialize(deserializer).map(|_| Counted)
    }
}

#[test]
fn the_target_type_is_deserialised_once() {
    let layers = Layers::new()
        .push(val(json!({"a": 1})))
        .push(val(json!({"b": 2})))
        .push(val(json!({"c": 3})));
    layers.load::<Counted>().unwrap();
    assert_eq!(DECODES.with(Cell::get), 1);
}

#[test]
fn deserialize_error_lists_files_and_variables_from_the_sources() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"port":"x"}"#);
    let loader = Loader::new(
        AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        environment(&[("MY_APP__NAME", "n")]),
    );
    match loader.layers().load::<Defaulted>().unwrap_err() {
        ConfigError::Deserialize {
            files, variables, ..
        } => {
            assert_eq!(files, vec![dir.path().join("default.json")]);
            assert_eq!(variables, vec!["MY_APP__NAME".to_owned()]);
        }
        other => panic!("unexpected {other}"),
    }
}

// Provenance (REQ-CFG-0007).

fn path(segments: &[&str]) -> KeyPath {
    KeyPath::from_segments(segments.iter().map(|s| (*s).to_owned()))
}

#[test]
fn env_leaf_names_its_variable_and_file_leaf_keeps_the_file() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":{"b":1,"c":2}}"#);
    let loader = Loader::new(
        AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        environment(&[("MY_APP__A__B", "3")]),
    );
    let rows = loader.layers().resolve().unwrap().provenance.rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].path.to_string(), "a.b");
    assert_eq!(rows[0].source, Source::env("MY_APP__A__B"));
    assert_eq!(rows[0].value, json!(3));
    assert_eq!(rows[1].path.to_string(), "a.c");
    assert_eq!(rows[1].source.kind, SourceKind::File);
    assert!(rows[1].source.name.ends_with("default.json"));
    assert_eq!(rows[1].value, json!(2));
}

#[test]
fn a_replaced_leaf_is_gone_from_provenance() {
    let layers = Layers::new()
        .push(val(json!({"a": {"b": 1}})))
        .push(val(json!({"a": 5})));
    let provenance = layers.resolve().unwrap().provenance;
    assert!(provenance.source_of(&path(&["a"])).is_some());
    assert!(provenance.source_of(&path(&["a", "b"])).is_none());
    assert_eq!(provenance.rows().len(), 1);
}

#[test]
fn merging_an_empty_object_keeps_the_leaves_below() {
    let layers = Layers::new()
        .push(val(json!({"a": {"b": 1}})))
        .push(val(json!({"a": {}})));
    let resolved = layers.resolve().unwrap();
    assert_eq!(resolved.value, json!({"a": {"b": 1}}));
    assert!(resolved.provenance.source_of(&path(&["a", "b"])).is_some());
    assert!(resolved.provenance.source_of(&path(&["a"])).is_none());
}

#[test]
fn a_custom_source_is_reported_and_source_of_matches_rows() {
    let layers = Layers::new()
        .push(val(json!({"a": 1, "k": 0})))
        .push(fixed(json!({"k": 1})));
    let provenance = layers.resolve().unwrap().provenance;
    let source = provenance.source_of(&path(&["k"])).unwrap();
    assert_eq!(source, &Source::custom("keychain", "login"));
    assert_eq!(source.to_string(), "keychain login");
    for row in provenance.rows() {
        assert_eq!(provenance.source_of(&row.path), Some(&row.source));
    }
    let rendered = serde_json::to_value(provenance.rows()).unwrap();
    assert_eq!(rendered[1]["path"], json!("k"));
    assert_eq!(rendered[1]["value"], json!(1));
}

#[test]
fn the_deepest_attribute_wins() {
    let outer = Source::custom("outer", "o");
    let inner = Source::custom("inner", "i");
    let layer = Layer::new(
        Source::custom("base", "b"),
        json!({"a": {"b": 1, "c": 2}, "d": 3}),
    )
    .attribute(["a"], outer.clone())
    .attribute(["a", "b"], inner.clone());
    let provenance = Layers::new()
        .push(Fixed(Some(layer)))
        .resolve()
        .unwrap()
        .provenance;
    assert_eq!(provenance.source_of(&path(&["a", "b"])), Some(&inner));
    assert_eq!(provenance.source_of(&path(&["a", "c"])), Some(&outer));
    assert_eq!(
        provenance.source_of(&path(&["d"])),
        Some(&Source::custom("base", "b"))
    );
}

#[test]
fn a_segment_with_a_dot_is_written_as_a_json_string() {
    let key = KeyPath::from_segments(["a.b".to_owned(), "c".to_owned()]);
    assert_eq!(key.to_string(), r#""a.b".c"#);
}

// Loader on the engine (REQ-CFG-0008).

#[test]
fn a_pushed_value_source_overrides_the_environment() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":0}"#);
    let loader = Loader::new(
        AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        environment(&[("MY_APP__A", "1")]),
    );
    let flags = Source::flag("--a");
    let value: Value = loader
        .layers()
        .push(ValueSource::new(flags, json!({"a": 2})))
        .load()
        .unwrap();
    assert_eq!(value, json!({"a": 2}));
}

#[test]
fn loader_layers_reports_a_missing_default_as_a_missing_file() {
    let dir = TempDir::new();
    let loader = Loader::new(
        AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        Vec::new(),
    );
    let engine = loader.layers().load::<Value>().unwrap_err();
    assert_eq!(engine.code(), "CFG.MISSING_FILE");
    let preset = loader.load::<Value>().unwrap_err();
    assert_eq!(preset.code(), "CFG.MISSING_DEFAULT");
}
