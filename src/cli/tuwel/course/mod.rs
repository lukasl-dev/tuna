pub mod get;

use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum Command {
    Get(get::Args),
}

impl Command {
    pub async fn run(self, socket: &Path) -> std::io::Result<()> {
        match self {
            Self::Get(args) => get::run(args, socket).await,
        }
    }
}
