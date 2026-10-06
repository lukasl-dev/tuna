pub mod list;

use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum Command {
    List,
}

impl Command {
    pub async fn run(self, socket: &Path) -> std::io::Result<()> {
        match self {
            Self::List => list::run(socket).await,
        }
    }
}
