{% if cron_commands %}
//! Same management verbs for every canonical Rust/Python/Monty target kind.
use clap::Subcommand;
use sc_cron_commands::{CronOperation, CronRequest};
use serde_json::Value;
use crate::ParseError;

/// Cron management subcommands, each taking the complete versioned JSON request.
#[derive(Debug, Subcommand)]
pub enum CronAction {
    /// Start an application-authorized schedule.
    Start { #[arg(long)] request: String },
    /// End the specified schedule generation.
    End { #[arg(long)] request: String },
    /// Query a bounded page of schedule views.
    List { #[arg(long)] request: String },
}

pub(crate) fn parse(action: CronAction) -> Result<Value, ParseError> {
    let (expected, body) = match action {
        CronAction::Start { request } => ("cron.start", request),
        CronAction::End { request } => ("cron.end", request),
        CronAction::List { request } => ("cron.list", request),
    };
    if body.len() > sc_cron_commands::contract::MAX_REQUEST_BYTES {
        return Err(ParseError::usage("cron request exceeds the documented JSON limit"));
    }
    let request: CronRequest = serde_json::from_str(&body)
        .map_err(|_| ParseError::usage("invalid versioned cron request"))?;
    if verb(&request) != expected {
        return Err(ParseError::usage("cron request command does not match the selected subcommand"));
    }
    serde_json::to_value(request).map_err(|_| ParseError::usage("invalid versioned cron request"))
}

pub(crate) fn verb(request: &CronRequest) -> &'static str {
    match &request.operation {
        CronOperation::Start(_) => "cron.start",
        CronOperation::End(_) => "cron.end",
        CronOperation::List(_) => "cron.list",
    }
}
{% endif %}
