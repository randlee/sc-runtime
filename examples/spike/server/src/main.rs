//! Spike server binary. Composition lives in `spike_stack::serve`.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "spike-server",
    about = "Serve the spike router on a Unix socket."
)]
struct Args {
    /// Unix socket to bind.
    #[arg(long)]
    socket: PathBuf,
    /// SQLite database file.
    #[arg(long)]
    database: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    spike_stack::serve(&args.socket, &args.database).await
}
