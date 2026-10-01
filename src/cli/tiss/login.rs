use std::io;
use thirtyfour::prelude::*;
use totp_rs::TOTP;

#[derive(clap::Args)]
#[group(id = "login")]
pub struct Args {
    #[arg(long, env = "TUNA_TISS_USER", hide_env_values = true)]
    username: String,

    #[arg(long, env = "TUNA_TISS_PASSWORD", hide_env_values = true)]
    password: String,

    #[arg(
        long,
        env = "TUNA_TISS_TOTP_URL",
        hide_env_values = true,
        conflicts_with = "totp_code"
    )]
    totp_url: Option<String>,

    #[arg(
        long,
        env = "TUNA_TISS_TOTP_CODE",
        hide_env_values = true,
        conflicts_with = "totp_url"
    )]
    totp_code: Option<String>,

    #[arg(long, default_value = "http://localhost:9515")]
    webdriver: String,

    #[arg(long)]
    headed: bool,
}

impl Args {
    #[tracing::instrument(name = "tiss.session.login", skip_all)]
    pub async fn connect(self) -> io::Result<WebDriver> {
        let totp_code = match (self.totp_code, self.totp_url) {
            (Some(code), _) => code,
            (_, Some(url)) => {
                let totp = TOTP::from_url(url).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid TOTP URL: expected a valid otpauth://totp/... URI",
                    )
                })?;
                totp.generate_current().map_err(io::Error::other)?
            }
            _ => String::new(),
        };

        let mut caps = DesiredCapabilities::chrome();
        if !self.headed {
            caps.set_headless().map_err(io::Error::other)?;
        }
        let driver = WebDriver::new(&self.webdriver, caps)
            .await
            .map_err(io::Error::other)?;

        let result = tuna::tiss::login::login(
            &driver,
            &self.username,
            &self.password,
            &totp_code,
        )
        .await;
        if let Err(error) = result {
            if let Err(cleanup_error) = driver.quit().await {
                tracing::warn!(error = %cleanup_error, "Failed to close browser session");
            }
            return Err(io::Error::other(error));
        }
        tracing::info!("Logged in to TISS successfully");
        Ok(driver)
    }
}

pub async fn run(args: Args) -> io::Result<()> {
    let driver = args.connect().await?;
    driver.quit().await.map_err(io::Error::other)
}
