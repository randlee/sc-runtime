//! CLI process. Report-only commands print a result. `daemon start` execs.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse(&args) {
        Ok(cli::Command::DaemonStart { json }) => {
            let report = cli::exec_daemon(json);
            if !report.stderr.is_empty() {
                eprintln!("{}", report.stderr);
            }
            if !report.stdout.is_empty() {
                println!("{}", report.stdout);
            }
            report.code
        }
        Ok(command) => {
            let (code, text) = cli::execute(command).await;
            if !text.is_empty() {
                println!("{text}");
            }
            code
        }
        Err(error) => {
            if let Some(help) = &error.help {
                print!("{help}");
            } else {
                eprintln!("{error}");
            }
            ExitCode::from(error.exit_code)
        }
    }
}
