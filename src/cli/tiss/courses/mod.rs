mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// List courses
    List,
}

impl Command {
    pub fn run(self) -> std::io::Result<()> {
        match self {
            Self::List => list::run(),
        }
    }
}
