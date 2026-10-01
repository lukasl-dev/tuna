#[tracing::instrument(name = "tuwel.courses.list")]
pub fn run() -> std::io::Result<()> {
    tracing::debug!("Listing courses");

    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "tuwel courses list is not implemented yet",
    ))
}
