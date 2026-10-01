pub mod courses;
pub mod groups;
pub mod login;
pub mod messages;
pub mod programmes;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Login,

    Messages,

    #[command(subcommand)]
    Programmes(programmes::Command),

    #[command(subcommand)]
    Courses(courses::Command),

    #[command(subcommand)]
    Groups(groups::Command),
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::Login => login::run(socket).await,
            Self::Messages => messages::run(socket).await,
            Self::Programmes(command) => command.run(socket).await,
            Self::Courses(command) => command.run(socket).await,
            Self::Groups(command) => command.run(socket).await,
        }
    }
}
