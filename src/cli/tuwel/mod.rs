pub mod courses;
pub mod login;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    Login,

    #[command(subcommand)]
    Courses(courses::Command),
}

impl Command {
    pub async fn run(self, socket: &std::path::Path) -> std::io::Result<()> {
        match self {
            Self::Login => login::run(socket).await,
            Self::Courses(command) => command.run(),
        }
    }
}
