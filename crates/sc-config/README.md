# sc-config

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
the process.
