pub mod exams;
pub mod get;
mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Exams(exams::Args),

    Get(get::Args),

    List,
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::Exams(args) => exams::run(args, socket).await,
            Self::Get(args) => get::run(args, socket).await,
            Self::List => list::run(),
        }
    }
}
