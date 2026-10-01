pub mod courses;
pub mod groups;
pub mod login;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Log in to TISS
    Login(login::Args),

    /// Manage courses
    #[command(subcommand)]
    Courses(courses::Command),

    /// Manage course groups
    #[command(subcommand)]
    Groups(groups::Command),
}

impl Command {
    pub async fn run(self) -> std::io::Result<()> {
        match self {
            Self::Login(args) => login::run(args).await,
            Self::Courses(command) => command.run(),
            Self::Groups(command) => command.run(),
        }
    }
}
