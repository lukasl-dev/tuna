pub mod courses;
pub mod groups;
pub mod login;
pub mod messages;
pub mod programmes;

use clap::Subcommand;

#[derive(clap::Args)]
pub struct Args {
    #[command(flatten)]
    login: login::Args,

    #[command(subcommand)]
    command: Command,
}

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

impl Args {
    pub async fn run(self) -> std::io::Result<()> {
        match self.command {
            Command::Login => login::run(self.login).await,
            Command::Messages => messages::run(self.login).await,
            Command::Programmes(command) => command.run(self.login).await,
            Command::Courses(command) => command.run(),
            Command::Groups(command) => command.run(self.login).await,
        }
    }
}
