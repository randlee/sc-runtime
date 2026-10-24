# sc-runtime

`sc-runtime` 0.1.0 is published on crates.io under the MIT license. The repository is <https://github.com/randlee/sc-runtime>. It is published after `sc-config`, `sc-transport`, and `sc-command`.

`sc-runtime` assembles one daemon: the instance lock, the project's stores,
one router, and one listener. The application loads configuration before
`Daemon::builder`. This crate does not read a config file and does not depend
on `sc-config`.

`run` is available only after `stores` and `routes`. `mcp` is optional.
Inside `run` the order is lock, stores, routes and optional MCP, then bind.
SIGINT and SIGTERM shut the listener down and return `Ok(())`.

```rust,no_run
use sc_runtime::{Daemon, DaemonConfig};
use sc_transport::AppName;
use utoipa_axum::router::OpenApiRouter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), sc_runtime::RuntimeError> {
    let app = AppName::new("my-app").expect("app name");
    let config = DaemonConfig::default();
    Daemon::builder(&app, &config)
        .stores(|| async { Ok(()) })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await?;
    Ok(())
}
```

`GET /health` is a liveness probe. It returns status 200 and an
`sc_command::Envelope` success body (`{ "status": "ok" }`) while the process
is serving HTTP. It does not check stores or other dependencies. A project
that needs dependency-aware readiness registers its own route, such as
`GET /ready`, on the project router. `sc-runtime` does not reserve that path
and does not treat `/health` as readiness.

`GET /openapi.json` returns the OpenAPI document from the project's router.
An rmcp service passed to `mcp` is mounted at `/mcp`.

The in-process fixture `sc_runtime::testing::DaemonFixture` is behind the
non-default feature `test-util`. A production build leaves that feature off.
