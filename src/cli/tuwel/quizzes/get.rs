use crate::worker::{self, Request};
use std::path::Path;

#[derive(clap::Args)]
pub struct Args {
    #[arg(
        long,
        help = "Course-module id from the quiz's view.php URL",
        value_parser = clap::value_parser!(u64).range(1..=9_007_199_254_740_991)
    )]
    id: u64,
}

#[tracing::instrument(name = "tuwel.quizzes.get.command", skip(args, socket), fields(id = args.id))]
pub async fn run(args: Args, socket: &Path) -> std::io::Result<()> {
    worker::request(socket, Request::TuwelQuizGet { id: args.id }).await
}
