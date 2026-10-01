pub mod list;

use super::login;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    List,
}

impl Command {
    pub async fn run(self, login: login::Args) -> std::io::Result<()> {
        match self {
            Self::List => list::run(login).await,
        }
    }
}
