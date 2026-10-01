#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    pub course: String,

    #[arg(long)]
    pub group: String,
}

#[tracing::instrument(
    name = "tiss.groups.register",
    skip(args),
    fields(course = %args.course, group = %args.group)
)]
pub fn run(args: Args) -> std::io::Result<()> {
    tracing::debug!("Registering for course group");

    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "tiss groups register is not implemented yet \
             (course {}, group {})",
            args.course, args.group
        ),
    ))
}
