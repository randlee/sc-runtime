//! Test helper for the process-environment entry point.
//!
//! The library tests start this binary with `Command`. It is not part of the
//! supported library facade.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<serde_json::Value, String> {
    let mut args = std::env::args();
    let _program = args.next();
    let mode = args.next().ok_or_else(|| "missing mode".to_owned())?;
    let app = sc_config::AppName::new("my-app").map_err(|err| err.to_string())?;
    match mode.as_str() {
        "load" => sc_config::load(app).map_err(|err| err.to_string()),
        "loader" => {
            let directory = args.next().ok_or_else(|| "missing directory".to_owned())?;
            let loader = sc_config::Loader::new(app, PathBuf::from(directory), Vec::new());
            loader.load().map_err(|err| err.to_string())
        }
        _ => Err(format!("unknown mode {mode}")),
    }
}
