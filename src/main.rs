mod cli;

use clap::Parser;
use cli::{Cli, Command};

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Tiss(command) => command.run(),
        Command::Tuwel(command) => command.run(),
    }
}
