pub mod courses;
pub mod login;
pub mod notifications;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Login,

    #[command(subcommand)]
    Notifications(notifications::Command),

    #[command(subcommand)]
    Courses(courses::Command),
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::Login => login::run(socket).await,
            Self::Notifications(command) => command.run(socket).await,
            Self::Courses(command) => command.run(),
        }
    }
}
