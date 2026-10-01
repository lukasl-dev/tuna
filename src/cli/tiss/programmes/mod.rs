pub mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    List,
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::List => list::run(socket).await,
        }
    }
}
