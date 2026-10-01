pub mod list;
pub mod register;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    List(list::Args),

    Register(register::Args),
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::List(args) => list::run(args, socket).await,
            Self::Register(args) => register::run(args, socket).await,
        }
    }
}
