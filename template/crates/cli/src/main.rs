//! CLI process. Report-only commands print a result. `daemon start` execs.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse(&args) {
        Ok(cli::Command::DaemonStart) => cli::exec_daemon(),
        Ok(command) => {
            let (code, text) = cli::execute(command).await;
            if !text.is_empty() {
                println!("{text}");
            }
            code
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}
