//! `StandardLayers`: REQ-CFG-0010.

#[path = "support/mod.rs"]
mod support;

use std::fs;
use std::path::Path;

use sc_config::{
    AppName, ConfigError, Layer, LayerSource, Slot, Source, StandardContext, StandardLayers,
};
use serde_json::{Value, json};
use support::{TempDir, environment};

fn name(text: &str) -> AppName {
    AppName::new(text).unwrap()
}

fn write(folder: &Path, file: &str, contents: &str) {
    fs::create_dir_all(folder).unwrap();
    fs::write(folder.join(file), contents).unwrap();
}

fn context(products: &[&str], env: &[(&str, &str)]) -> StandardContext {
    StandardContext::new(
        name("sc-agent"),
        products.iter().map(|p| name(p)).collect(),
        environment(env),
    )
}

struct Dirs {
    system: TempDir,
    home: TempDir,
    project: TempDir,
}

fn dirs() -> Dirs {
    Dirs {
        system: TempDir::new(),
        home: TempDir::new(),
        project: TempDir::new(),
    }
}

fn full(dirs: &Dirs, env: &[(&str, &str)]) -> StandardLayers {
    let user = dirs.home.path().join(".sc-agent");
    let project = dirs.project.path().join(".sc-agent");
    write(dirs.system.path(), "sc-agent.json", r#"{"v":"system"}"#);
    write(&user, "sc-agent.json", r#"{"v":"user"}"#);
    write(&project, "sc-agent.json", r#"{"v":"project"}"#);
    write(&project, "sc-agent.local.json", r#"{"v":"local"}"#);
    let context = context(&["sc-agent"], env)
        .with_system_dir(dirs.system.path())
        .with_home(&user)
        .with_project_root(dirs.project.path());
    StandardLayers::new(context)
}

fn slots_in_order(layers: StandardLayers) -> StandardLayers {
    layers
        .defaults(json!({"v": "defaults"}))
        .system()
        .user()
        .project()
        .local()
        .env(["SC_AGENT__"])
        .flags(json!({"v": "flags"}))
}

fn slots_reversed(layers: StandardLayers) -> StandardLayers {
    layers
        .flags(json!({"v": "flags"}))
        .env(["SC_AGENT__"])
        .local()
        .project()
        .user()
        .system()
        .defaults(json!({"v": "defaults"}))
}

fn winner(layers: StandardLayers) -> Value {
    layers.load::<Value>().unwrap()["v"].clone()
}

#[test]
fn the_highest_filled_slot_wins_in_any_call_order() {
    let d = dirs();
    let env = [("SC_AGENT__V", "env")];
    assert_eq!(winner(slots_in_order(full(&d, &env))), json!("flags"));
    assert_eq!(winner(slots_reversed(full(&d, &env))), json!("flags"));
    let without_flags = full(&d, &env)
        .defaults(json!({"v": "defaults"}))
        .system()
        .user()
        .project()
        .local()
        .env(["SC_AGENT__"]);
    assert_eq!(winner(without_flags), json!("env"));
    let files_only = full(&d, &[])
        .defaults(json!({"v": "defaults"}))
        .system()
        .user()
        .project();
    assert_eq!(winner(files_only), json!("project"));
    let down_to_user = full(&d, &[])
        .defaults(json!({"v": "defaults"}))
        .system()
        .user();
    assert_eq!(winner(down_to_user), json!("user"));
    let down_to_system = full(&d, &[]).defaults(json!({"v": "defaults"})).system();
    assert_eq!(winner(down_to_system), json!("system"));
    let defaults_only = full(&d, &[]).defaults(json!({"v": "defaults"}));
    assert_eq!(winner(defaults_only), json!("defaults"));
}

#[test]
fn local_slot_overrides_project_slot() {
    let d = dirs();
    assert_eq!(winner(full(&d, &[]).project().local()), json!("local"));
}

#[test]
fn calling_a_slot_again_replaces_the_earlier_value() {
    let layers = StandardLayers::new(context(&[], &[]))
        .defaults(json!({"v": 1}))
        .defaults(json!({"v": 2}));
    assert_eq!(winner(layers), json!(2));
}

struct Inserted(&'static str);

impl LayerSource for Inserted {
    fn load(&self, _base: &Value) -> Result<Option<Layer>, ConfigError> {
        Ok(Some(Layer::new(
            Source::custom("test", self.0),
            json!({"v": self.0}),
        )))
    }
}

#[test]
fn an_inserted_source_sits_directly_above_its_slot() {
    let d = dirs();
    let layers = full(&d, &[])
        .user()
        .project()
        .insert(Slot::User, Inserted("inserted"));
    // Above the user file, below the project file.
    assert_eq!(winner(layers), json!("project"));
    let layers = full(&d, &[])
        .user()
        .insert(Slot::User, Inserted("inserted"));
    assert_eq!(winner(layers), json!("inserted"));
    let first = full(&d, &[])
        .user()
        .insert(Slot::User, Inserted("one"))
        .insert(Slot::User, Inserted("two"));
    assert_eq!(winner(first), json!("two"));
}

#[test]
fn products_layer_in_list_order_and_local_files_win() {
    let project = TempDir::new();
    let folder = project.path().join(".sc-agent");
    write(
        &folder,
        "sc-agent.json",
        r#"{"a":"agent","b":"agent","c":"agent"}"#,
    );
    write(&folder, "decidr.json", r#"{"b":"decidr","c":"decidr"}"#);
    write(&folder, "decidr.local.json", r#"{"c":"decidr-local"}"#);
    let context = context(&["sc-agent", "decidr"], &[]).with_project_root(project.path());
    let value: Value = StandardLayers::new(context.clone())
        .project()
        .load()
        .unwrap();
    assert_eq!(value, json!({"a": "agent", "b": "decidr", "c": "decidr"}));
    let value: Value = StandardLayers::new(context)
        .project()
        .local()
        .load()
        .unwrap();
    assert_eq!(
        value,
        json!({"a": "agent", "b": "decidr", "c": "decidr-local"})
    );
}

#[test]
fn system_and_user_folders_read_local_files_after_plain_files() {
    let system = TempDir::new();
    write(system.path(), "decidr.local.json", r#"{"v":"local"}"#);
    write(system.path(), "sc-agent.json", r#"{"v":"plain","w":1}"#);
    let context = context(&["sc-agent", "decidr"], &[]).with_system_dir(system.path());
    let value: Value = StandardLayers::new(context).system().load().unwrap();
    assert_eq!(value, json!({"v": "local", "w": 1}));
}

#[test]
fn an_unset_folder_reads_nothing() {
    let value: Value = StandardLayers::new(context(&["sc-agent"], &[]))
        .system()
        .user()
        .project()
        .local()
        .load()
        .unwrap();
    assert_eq!(value, json!({}));
}

#[test]
fn accepts_applies_to_one_products_files_in_every_slot() {
    let project = TempDir::new();
    let folder = project.path().join(".sc-agent");
    write(&folder, "decidr.json", r#"{"log_root":"x"}"#);
    write(&folder, "sc-agent.json", r#"{"log_root":"y"}"#);
    let context = context(&["sc-agent", "decidr"], &[]).with_project_root(project.path());
    let error = StandardLayers::new(context.clone())
        .project()
        .accepts(&name("decidr"), [["model"]])
        .load::<Value>()
        .unwrap_err();
    assert_eq!(error.code(), "CFG.UNKNOWN_KEY");
    let ignored = StandardLayers::new(context)
        .project()
        .accepts(&name("other"), [["model"]])
        .load::<Value>();
    assert!(ignored.is_ok());
}

#[test]
fn later_env_prefixes_win_and_aliases_apply_last() {
    let env = [("A_X", "1"), ("B_X", "2"), ("SC_X", "3")];
    let layers = StandardLayers::new(context(&["sc-agent"], &env))
        .env(["A_", "B_"])
        .alias("SC_X", ["y"]);
    assert_eq!(layers.load::<Value>().unwrap(), json!({"x": 2, "y": 3}));
}

#[test]
fn the_user_folder_comes_from_the_home_variable_chain() {
    let home = TempDir::new();
    let relocated = TempDir::new();
    let named = TempDir::new();
    write(
        &home.path().join(".sc-agent"),
        "sc-agent.json",
        r#"{"from":"home"}"#,
    );
    write(
        relocated.path(),
        "sc-agent.json",
        r#"{"from":"sc-agent-home"}"#,
    );
    write(named.path(), "sc-agent.json", r#"{"from":"atm-home"}"#);
    let home_name = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let home_value = home.path().to_string_lossy().into_owned();
    let relocated_value = relocated.path().to_string_lossy().into_owned();
    let named_value = named.path().to_string_lossy().into_owned();

    let user = |env: &[(&str, &str)], var: Option<&str>| -> Value {
        let mut context = context(&["sc-agent"], env);
        if let Some(var) = var {
            context = context.with_home_var(var);
        }
        StandardLayers::new(context).user().load().unwrap()
    };
    let base = [(home_name, home_value.as_str())];
    assert_eq!(user(&base, None), json!({"from": "home"}));
    let both = [
        (home_name, home_value.as_str()),
        ("SC_AGENT_HOME", relocated_value.as_str()),
        ("ATM_HOME", named_value.as_str()),
    ];
    assert_eq!(user(&both, None), json!({"from": "sc-agent-home"}));
    assert_eq!(user(&both, Some("ATM_HOME")), json!({"from": "atm-home"}));
}

#[test]
fn with_home_wins_over_the_variables() {
    let explicit = TempDir::new();
    let relocated = TempDir::new();
    write(explicit.path(), "sc-agent.json", r#"{"from":"explicit"}"#);
    write(relocated.path(), "sc-agent.json", r#"{"from":"variable"}"#);
    let relocated_value = relocated.path().to_string_lossy().into_owned();
    let context = context(
        &["sc-agent"],
        &[("SC_AGENT_HOME", relocated_value.as_str())],
    )
    .with_home(explicit.path());
    let value: Value = StandardLayers::new(context).user().load().unwrap();
    assert_eq!(value, json!({"from": "explicit"}));
}

#[test]
fn defaults_and_flags_carry_default_and_flag_sources() {
    let layers = StandardLayers::new(context(&[], &[]))
        .defaults(json!({"a": 1}))
        .flags(json!({"b": 2}));
    let rows = layers.resolve().unwrap().rows();
    let kinds: Vec<String> = rows.iter().map(|r| r.source.kind.to_string()).collect();
    assert_eq!(kinds, ["default", "flag"]);
}

#[test]
fn an_empty_home_variable_is_ignored_and_never_a_setting() {
    let home = TempDir::new();
    write(
        &home.path().join(".sc-agent"),
        "sc-agent.json",
        r#"{"from":"home"}"#,
    );
    let home_value = home.path().to_string_lossy().into_owned();
    let home_name = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let env = [(home_name, home_value.as_str()), ("SC_AGENT_HOME", "")];
    let value: Value = StandardLayers::new(context(&["sc-agent"], &env))
        .user()
        .env(["SC_AGENT_"])
        .load()
        .unwrap();
    assert_eq!(value, json!({"from": "home"}));
}

#[test]
fn an_alias_alone_fills_the_env_slot() {
    let layers =
        StandardLayers::new(context(&[], &[("SC_PROJECT", "x")])).alias("SC_PROJECT", ["project"]);
    assert_eq!(layers.load::<Value>().unwrap(), json!({"project": "x"}));
}

#[test]
fn an_alias_wins_over_prefixed_variables_in_either_call_order() {
    let env = [("SC_AGENT_PROJECT", "prefixed"), ("SC_PROJECT", "alias")];
    let env_first = StandardLayers::new(context(&[], &env))
        .env(["SC_AGENT_"])
        .alias("SC_PROJECT", ["project"]);
    let alias_first = StandardLayers::new(context(&[], &env))
        .alias("SC_PROJECT", ["project"])
        .env(["SC_AGENT_"]);
    let expected = json!({"project": "alias"});
    assert_eq!(env_first.load::<Value>().unwrap(), expected);
    assert_eq!(alias_first.load::<Value>().unwrap(), expected);
}

#[test]
fn the_home_variable_is_never_applied_as_a_setting() {
    let relocated = TempDir::new();
    write(relocated.path(), "sc-agent.json", r#"{"from":"user"}"#);
    let value = relocated.path().to_string_lossy().into_owned();
    for (variable, builder) in [("SC_AGENT_HOME", None), ("ATM_HOME", Some("ATM_HOME"))] {
        let mut context = context(&["sc-agent"], &[(variable, value.as_str())]);
        if let Some(name) = builder {
            context = context.with_home_var(name);
        }
        let loaded: Value = StandardLayers::new(context)
            .user()
            .env(["SC_AGENT_", "ATM_"])
            .load()
            .unwrap();
        assert_eq!(loaded, json!({"from": "user"}), "{variable}");
    }
}

#[test]
fn a_family_with_trailing_underscores_keeps_its_own_home_variable() {
    let relocated = TempDir::new();
    write(relocated.path(), "a__.json", r#"{"from":"relocated"}"#);
    let value = relocated.path().to_string_lossy().into_owned();
    let family = name("a__");
    let user = |variable: &str| -> Value {
        let env = environment(&[(variable, value.as_str())]);
        let context = StandardContext::new(family.clone(), vec![family.clone()], env);
        StandardLayers::new(context).user().load().unwrap()
    };
    // The prefix is `A____`; only its last `__` is replaced by `_HOME`.
    assert_eq!(user("A___HOME"), json!({"from": "relocated"}));
    assert_eq!(user("A_HOME"), json!({}));
}

#[test]
fn an_empty_home_variable_is_unset() {
    let home_name = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let loaded: Value = StandardLayers::new(context(&["sc-agent"], &[(home_name, "")]))
        .user()
        .load()
        .unwrap();
    assert_eq!(loaded, json!({}));
}
