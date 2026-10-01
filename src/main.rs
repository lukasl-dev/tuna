mod cli;
mod worker;

use clap::Parser;
use cli::{Cli, Command, LogFormat};
use std::io::IsTerminal;
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tuna=info".into()),
        )
        .with_writer(std::io::stderr);

    match cli.log_format {
        LogFormat::Text => {
            subscriber.with_ansi(std::io::stderr().is_terminal()).init()
        }
        LogFormat::Json => subscriber.json().init(),
    }

    let result = async {
        let socket = worker::socket_path(cli.socket)?;
        match cli.command {
            Command::Serve(args) => worker::serve(&socket, args).await,
            Command::Stop => {
                worker::request(&socket, worker::Request::Stop).await
            }
            Command::Tiss(command) => command.run(&socket).await,
            Command::Tuwel(command) => command.run(),
        }
    }
    .await;

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = %error, kind = ?error.kind(), "Command failed");
            ExitCode::FAILURE
        }
    }
}
