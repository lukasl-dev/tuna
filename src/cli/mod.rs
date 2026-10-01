pub mod serve;
pub mod tiss;
pub mod tuwel;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about = "TU Wien automation")]
pub struct Cli {
    #[arg(long, global = true, value_enum, default_value = "text")]
    pub log_format: LogFormat,

    #[arg(long, global = true, env = "TUNA_SOCKET")]
    pub socket: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
pub enum Command {
    Serve(serve::Args),

    Stop,

    #[command(subcommand)]
    Tiss(tiss::Command),

    #[command(subcommand)]
    Tuwel(tuwel::Command),
}
