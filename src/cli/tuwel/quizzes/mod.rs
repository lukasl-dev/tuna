pub mod get;

use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum Command {
    #[command(
        about = "Read a quiz overview",
        long_about = "Read a quiz overview without activating quiz controls. Moodle may log the view, update completion tracking, or finalise an already-expired attempt when opening this page."
    )]
    Get(get::Args),
}

impl Command {
    pub async fn run(self, socket: &Path) -> std::io::Result<()> {
        match self {
            Self::Get(args) => get::run(args, socket).await,
        }
    }
}
