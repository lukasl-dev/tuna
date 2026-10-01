#[derive(clap::Args)]
pub struct Args {
    /// Course number
    #[arg(long)]
    pub course: String,

    /// Group identifier
    #[arg(long)]
    pub group: String,
}

pub fn run(args: Args) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "tiss groups register is not implemented yet \
             (course {}, group {})",
            args.course, args.group
        ),
    ))
}
