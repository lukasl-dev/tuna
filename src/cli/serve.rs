use std::io;
use totp_rs::TOTP;

#[derive(clap::Args)]
pub struct Args {
    #[arg(long, env = "TUNA_TISS_USER", hide_env_values = true)]
    pub username: String,

    #[arg(long, env = "TUNA_TISS_PASSWORD", hide_env_values = true)]
    pub password: String,

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
    pub webdriver: String,

    #[arg(long)]
    pub headed: bool,
}

impl Args {
    pub fn totp_code(&self) -> io::Result<String> {
        match (&self.totp_code, &self.totp_url) {
            (Some(code), _) => Ok(code.clone()),
            (_, Some(url)) => {
                let totp = TOTP::from_url(url).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid TOTP URL: expected a valid otpauth://totp/... URI",
                    )
                })?;
                totp.generate_current().map_err(io::Error::other)
            }
            _ => Ok(String::new()),
        }
    }
}
