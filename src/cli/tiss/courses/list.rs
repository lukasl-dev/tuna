#[tracing::instrument(name = "tiss.courses.list")]
pub fn run() -> std::io::Result<()> {
    tracing::debug!("Listing courses");

    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "tiss courses list is not implemented yet",
    ))
}
