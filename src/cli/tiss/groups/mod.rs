pub mod list;
pub mod register;

use super::login;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    List(list::Args),

    Register(register::Args),
}

impl Command {
    pub async fn run(self, login: login::Args) -> std::io::Result<()> {
        match self {
            Self::List(args) => list::run(args, login).await,
            Self::Register(args) => register::run(args),
        }
    }
}
