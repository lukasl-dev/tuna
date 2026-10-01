use std::io;
use thirtyfour::prelude::*;
use totp_rs::TOTP;

#[derive(clap::Args)]
pub struct Args {
    /// TISS username
    #[arg(long)]
    username: String,

    /// TISS password
    #[arg(long)]
    password: String,

    /// Generate a code from an otpauth://totp/... URI
    #[arg(long, conflicts_with = "totp_code")]
    totp_url: Option<String>,

    /// Use an existing TOTP code (leading zeros are preserved)
    #[arg(long, conflicts_with = "totp_url")]
    totp_code: Option<String>,

    /// WebDriver server URL
    #[arg(long, default_value = "http://localhost:9515")]
    webdriver: String,
}

#[tracing::instrument(name = "tiss.login.command", skip_all)]
pub async fn run(args: Args) -> io::Result<()> {
    let totp_code = match (args.totp_code, args.totp_url) {
        (Some(code), _) => code,
        (_, Some(url)) => TOTP::from_url(url)
            // URL parsing errors may contain the secret; do not forward them.
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid TOTP URL: expected a valid otpauth://totp/... URI",
                )
            })?
            .generate_current()
            .map_err(io::Error::other)?,
        _ => String::new(),
    };

    let driver = WebDriver::new(&args.webdriver, DesiredCapabilities::chrome())
        .await
        .map_err(io::Error::other)?;

    let result = tuna::tiss::login::login(
        &driver,
        &args.username,
        &args.password,
        &totp_code,
    )
    .await;
    let cleanup = driver.quit().await;

    if let Err(error) = result {
        if let Err(cleanup_error) = cleanup {
            tracing::warn!(error = %cleanup_error, "Failed to close browser session");
        }
        return Err(io::Error::other(error));
    }
    cleanup.map_err(io::Error::other)?;

    tracing::info!("Logged in to TISS successfully");
    Ok(())
}
