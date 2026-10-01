pub mod courses;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Manage courses
    #[command(subcommand)]
    Courses(courses::Command),
}

impl Command {
    pub fn run(self) -> std::io::Result<()> {
        match self {
            Self::Courses(command) => command.run(),
        }
    }
}
