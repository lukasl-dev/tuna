pub mod register;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Register for a course group
    Register(register::Args),
}

impl Command {
    pub fn run(self) -> std::io::Result<()> {
        match self {
            Self::Register(args) => register::run(args),
        }
    }
}
