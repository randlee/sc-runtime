//! Clap client that calls the spike REST route over a Unix socket.
//!
//! The binary compiles only on Unix. Other targets fail with `compile_error`
//! because `reqwest`'s Unix socket client is the spike's only transport.

#[cfg(not(unix))]
compile_error!(
    "spike-cli sends requests through reqwest's Unix socket client and only builds on Unix targets"
);

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "spike-cli",
    about = "Call the spike note route over a Unix socket."
)]
struct Args {
    /// Unix socket the spike server is listening on.
    #[arg(long)]
    socket: PathBuf,
    /// Note key.
    #[arg(long)]
    key: String,
    /// Note body.
    #[arg(long)]
    body: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let _default_marker = spike_stack::client_marker();
    let builder = reqwest::Client::builder();
    #[cfg(unix)]
    let builder = builder.unix_socket(&*args.socket);
    let client = builder.build().context("build reqwest client")?;
    let response = client
        .post("http://localhost/notes")
        .json(&json!({ "key": args.key, "body": args.body }))
        .send()
        .await
        .context("POST /notes")?;
    let status = response.status();
    let text = response.text().await.context("read body")?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}: {text}");
    }
    println!("{text}");
    Ok(())
}
