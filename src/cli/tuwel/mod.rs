pub mod courses;
pub mod login;
pub mod notifications;
pub mod timeline;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Login,

    Timeline,

    #[command(subcommand)]
    Notifications(notifications::Command),

    #[command(subcommand)]
    Courses(courses::Command),
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::Login => login::run(socket).await,
            Self::Timeline => timeline::run(socket).await,
            Self::Notifications(command) => command.run(socket).await,
            Self::Courses(command) => command.run(),
        }
    }
}
