use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let Ok(name) = sc_config::AppName::new("spike") else {
        return ExitCode::from(1);
    };
    let Ok(cfg) = sc_config::load::<spike_stack::SpikeConfig>(name) else {
        return ExitCode::from(1);
    };
    let Ok(app) = sc_transport::AppName::new("spike") else {
        return ExitCode::from(1);
    };
    let database = cfg.database.clone();
    match sc_runtime::Daemon::builder(&app, &cfg.daemon)
        .stores(move || spike_stack::stores(database))
        .routes(spike_stack::routes)
        .mcp(spike_stack::mcp)
        .run()
        .await
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}
