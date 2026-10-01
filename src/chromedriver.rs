use std::io;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

pub struct ChromeDriver {
    pub child: Child,
    pub url: String,
}

impl ChromeDriver {
    pub async fn start() -> io::Result<Self> {
        let mut child = Command::new("chromedriver")
            .args(["--port=0", "--log-level=SEVERE"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!(
                        "could not start chromedriver: {error}; \
                         install it on PATH or use --webdriver URL"
                    ),
                )
            })?;

        let mut output = BufReader::new(child.stdout.take().unwrap());
        let port = tokio::time::timeout(Duration::from_secs(10), async {
            let mut line = String::new();
            loop {
                line.clear();
                if output.read_line(&mut line).await? == 0 {
                    let status = child.wait().await?;
                    return Err(io::Error::other(format!(
                        "chromedriver exited before becoming ready: {status}"
                    )));
                }

                if let Some(port) = line
                    .trim()
                    .strip_prefix(
                        "ChromeDriver was started successfully on port ",
                    )
                    .and_then(|port| port.strip_suffix('.'))
                    .and_then(|port| port.parse::<u16>().ok())
                    .filter(|port| *port != 0)
                {
                    return Ok(port);
                }
            }
        })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "chromedriver did not become ready within 10 seconds",
            )
        })??;

        tokio::spawn(async move {
            let _ = tokio::io::copy(&mut output, &mut tokio::io::sink()).await;
        });

        tracing::info!(port, "Started ChromeDriver");
        Ok(Self {
            child,
            url: format!("http://127.0.0.1:{port}"),
        })
    }
}
