use crate::cli::tiss::login;
use std::io::{self, Write};

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    semester: String,

    #[arg(long)]
    course: String,
}

#[tracing::instrument(
    name = "tiss.courses.get.command",
    skip(args, login),
    fields(semester = %args.semester, course = %args.course)
)]
pub async fn run(args: Args, login: login::Args) -> io::Result<()> {
    let driver = login.connect().await?;
    let result =
        tuna::tiss::courses::get(&driver, &args.semester, &args.course).await;
    let cleanup = driver.quit().await;

    let course = match result {
        Ok(course) => course,
        Err(error) => {
            if let Err(cleanup_error) = cleanup {
                tracing::warn!(error = %cleanup_error, "Failed to close browser session");
            }
            return Err(io::Error::other(error));
        }
    };
    cleanup.map_err(io::Error::other)?;

    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &course)
        .map_err(io::Error::other)?;
    writeln!(stdout)
}
