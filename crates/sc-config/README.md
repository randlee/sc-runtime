# sc-config

`sc-config` 0.1.0 is published on crates.io under the MIT license. The repository is <https://github.com/randlee/sc-runtime>.

`sc-config` loads typed JSON configuration for one application. It reads
`default.json`, merges an optional `local.json`, then applies environment
variables prefixed by the application name. The loader is synchronous and
returns `ConfigError` instead of panicking.

```rust,no_run
use sc_config::{load, AppName};
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    port: u16,
}

fn main() -> Result<(), sc_config::ConfigError> {
    let app = AppName::new("my-app")?;
    let settings: Settings = load(app)?;
    println!("{}", settings.port);
    Ok(())
}
```

`Loader` takes an explicit directory and environment so tests do not change
the process. `AppName::env_prefix` builds the environment prefix: ASCII
letters are upper-cased, and both `-` and `_` become `_`, then `__`.
`my-app` and `my_app` both yield `MY_APP__`.

## Layers

`Loader` is a preset on one layered engine. `Layers` holds an ordered list of
`LayerSource`s, lowest first. It merges their JSON values with the same rule
as `default.json` over `local.json` (objects merge key by key, everything else
is replaced) and deserialises the result into your type once.

- `FileSource::new(path)` reads one JSON file. It is required unless
  `.optional()`; `.accepts([["a"], ["b", "c"]])` rejects keys outside those
  key paths with `ConfigError::UnknownKey`.
- `EnvSource::new(prefix, environment)` applies variables whose names start
  with the literal `prefix`, using the `__` naming of `Loader`; `.alias(var,
  ["key", "path"])` maps one exact variable name to a key path.
- `ValueSource::new(source, value)` contributes a value you built, for
  defaults or parsed command-line flags. `sc-config` does not parse flags.
- Implement `LayerSource` for anything else (a keychain, a remote store, other
  file formats). Report your own failure with `ConfigError::SourceFailed`.
- `Layers::resolve()` returns `Resolved`; `Resolved::rows()` names the source
  of every leaf, for a `config show` style command.

## Standard layer order

Products that share a family of settings use `StandardLayers`, which enforces
one order, lowest first: defaults, system, user, project, local, env, flags.
Fill the slots you want; call order does not matter. `insert(Slot::User,
source)` adds your own source directly above a slot; calling a slot method
again replaces its earlier value.

```rust,no_run
use sc_config::{AppName, StandardContext, StandardLayers};

# fn main() -> Result<(), sc_config::ConfigError> {
let family = AppName::new("sc-agent")?;
let environment = std::env::vars_os().collect();
let context = StandardContext::new(family.clone(), vec![family], environment)
    .with_project_root(".");
let value: serde_json::Value = StandardLayers::new(context)
    .defaults(serde_json::json!({"port": 7400}))
    .user()
    .project()
    .local()
    .env(["SC_AGENT__"])
    .load()?;
# let _ = value;
# Ok(())
# }
```

Each product reads `<product>.json`, then `<product>.local.json`, from:

- system: the folder you pass to `with_system_dir`. `sc-config` has no
  built-in location. Recommended: `/etc/<family>/` on Unix and
  `%ProgramData%\<family>\` on Windows.
- user: `with_home(dir)` if set; else the folder named by `<FAMILY>_HOME`
  (for example `SC_AGENT_HOME`; rename it with `with_home_var`) in the
  context's environment, if not empty; that variable is never applied as a
  setting; else `~/.<family>/`, where `~` is `HOME`
  (`USERPROFILE` on Windows).
- project: `<project_root>/.<family>/`. The `.json` files are the project
  slot and the `.local.json` files are the local slot.
