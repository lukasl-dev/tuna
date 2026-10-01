use crate::worker::{self, Request};
use std::path::Path;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    semester: String,

    #[arg(long)]
    course: String,

    #[arg(long)]
    group: String,
}

#[tracing::instrument(
    name = "tiss.groups.register.command",
    skip(args, socket),
    fields(semester = %args.semester, course = %args.course, group = %args.group)
)]
pub async fn run(args: Args, socket: &Path) -> std::io::Result<()> {
    worker::request(
        socket,
        Request::GroupsRegister {
            semester: args.semester,
            course: args.course,
            group: args.group,
        },
    )
    .await
}
