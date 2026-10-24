//! Clap client that calls the spike REST route over a Unix socket.

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
    if spike_stack::client_marker().is_empty() {
        anyhow::bail!("spike stack default marker was empty");
    }
    let client = reqwest::Client::builder()
        .unix_socket(&*args.socket)
        .build()
        .context("build reqwest client")?;
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
