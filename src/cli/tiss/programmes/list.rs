use crate::cli::tiss::login;
use std::io::{self, Write};

#[tracing::instrument(name = "tiss.programmes.command", skip_all)]
pub async fn run(login: login::Args) -> io::Result<()> {
    let driver = login.connect().await?;
    let result = tuna::tiss::programmes::programmes(&driver).await;
    let cleanup = driver.quit().await;

    let items = match result {
        Ok(items) => items,
        Err(error) => {
            if let Err(cleanup_error) = cleanup {
                tracing::warn!(error = %cleanup_error, "Failed to close browser session");
            }
            return Err(io::Error::other(error));
        }
    };
    cleanup.map_err(io::Error::other)?;

    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &items)
        .map_err(io::Error::other)?;
    writeln!(stdout)
}
