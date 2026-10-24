//! Clap client that calls the spike REST route through `sc-transport`.

use std::process::ExitCode;

use clap::Parser;
use sc_transport::{Client, Endpoint};
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(
    name = "spike-cli",
    about = "Call the spike note route through sc-transport."
)]
struct Args {
    /// Endpoint text (`uds:<path>` or `tcp:<socket-addr>`).
    #[arg(long, default_value = "uds:instance/daemon.sock")]
    endpoint: String,
    /// Note key.
    #[arg(long)]
    key: String,
    /// Note body.
    #[arg(long)]
    body: String,
}

#[derive(Debug, Deserialize)]
struct Note {
    key: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct Reply {
    ok: bool,
    data: Option<Note>,
    #[allow(dead_code)]
    error: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let _default_marker = spike_stack::client_marker();
    let Ok(endpoint) = args.endpoint.parse::<Endpoint>() else {
        eprintln!("invalid endpoint");
        return ExitCode::from(1);
    };
    let Ok(client) = Client::new(endpoint) else {
        eprintln!("the transport client could not be built");
        return ExitCode::from(1);
    };
    let reply = client
        .post::<Reply, _>(
            "/notes",
            &serde_json::json!({ "key": args.key, "body": args.body }),
        )
        .await;
    let Ok(reply) = reply else {
        eprintln!("the note request failed");
        return ExitCode::from(1);
    };
    if !reply.ok {
        eprintln!("the note request was rejected");
        return ExitCode::from(1);
    }
    let Some(note) = reply.data else {
        eprintln!("the note response had no data");
        return ExitCode::from(1);
    };
    match serde_json::to_string(&serde_json::json!({ "key": note.key, "body": note.body })) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(_) => ExitCode::from(1),
    }
}
