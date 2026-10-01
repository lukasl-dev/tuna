use crate::worker::{self, Request};
use std::path::Path;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    semester: String,

    #[arg(long)]
    course: String,
}

#[tracing::instrument(
    name = "tiss.courses.exams.command",
    skip(args, socket),
    fields(semester = %args.semester, course = %args.course)
)]
pub async fn run(args: Args, socket: &Path) -> std::io::Result<()> {
    worker::request(
        socket,
        Request::CourseExams {
            semester: args.semester,
            course: args.course,
        },
    )
    .await
}
