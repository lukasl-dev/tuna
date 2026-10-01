pub mod courses;
pub mod groups;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Manage courses
    #[command(subcommand)]
    Courses(courses::Command),

    /// Manage course groups
    #[command(subcommand)]
    Groups(groups::Command),
}

impl Command {
    pub fn run(self) -> std::io::Result<()> {
        match self {
            Self::Courses(command) => command.run(),
            Self::Groups(command) => command.run(),
        }
    }
}
