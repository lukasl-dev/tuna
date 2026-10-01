pub mod tiss;
pub mod tuwel;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "TU Wien automation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Tiss(tiss::Command),

    #[command(subcommand)]
    Tuwel(tuwel::Command),
}
