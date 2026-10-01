pub mod get;
mod list;

use super::login;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Get(get::Args),

    List,
}

impl Command {
    pub async fn run(self, login: login::Args) -> std::io::Result<()> {
        match self {
            Self::Get(args) => get::run(args, login).await,
            Self::List => list::run(),
        }
    }
}
