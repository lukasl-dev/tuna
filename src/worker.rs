use crate::chromedriver::ChromeDriver;
use crate::cli::serve;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{
    DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt,
};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thirtyfour::error::WebDriverErrorInner;
use thirtyfour::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

#[derive(Deserialize, Serialize)]
#[serde(
    tag = "command",
    content = "arguments",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Request {
    Login,
    Messages,
    Programmes,
    CourseGet {
        semester: String,
        course: String,
    },
    CourseExams {
        semester: String,
        course: String,
    },
    GroupsList {
        semester: String,
        course: String,
    },
    GroupsRegister {
        semester: String,
        course: String,
        group: String,
    },
    Stop,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
enum Response {
    Ok(Value),
    Error(String),
}

pub fn socket_path(path: Option<PathBuf>) -> io::Result<PathBuf> {
    if let Some(path) = path {
        return Ok(path);
    }

    let directory = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(dirs::cache_dir)
        .ok_or_else(|| {
            io::Error::other("cannot determine worker socket directory")
        })?;

    Ok(directory.join("tuna").join("worker.sock"))
}

fn check_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(path)?;

    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "worker socket directory must be owned by you with permissions 0700",
        ));
    }

    Ok(())
}

struct WorkerSocket {
    path: PathBuf,
    listener: UnixListener,
    _lock: File,
}

impl WorkerSocket {
    async fn bind(path: &Path) -> io::Result<Self> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("invalid socket path"))?;

        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
        check_directory(parent)?;

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path.with_extension("lock"))?;

        lock.try_lock().map_err(|error| {
            io::Error::other(format!(
                "cannot start another worker on this socket: {error}"
            ))
        })?;

        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if !metadata.file_type().is_socket()
                    || metadata.uid() != unsafe { libc::geteuid() }
                {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "refusing to replace a non-owned socket",
                    ));
                }

                match UnixStream::connect(path).await {
                    Ok(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::AddrInUse,
                            "worker socket is already active",
                        ));
                    }
                    Err(error)
                        if error.kind() == io::ErrorKind::ConnectionRefused =>
                    {
                        fs::remove_file(path)?;
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        let socket = Self {
            listener: UnixListener::bind(path)?,
            path: path.to_owned(),
            _lock: lock,
        };

        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(socket)
    }
}

impl Drop for WorkerSocket {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

async fn read_message<T: serde::de::DeserializeOwned>(
    stream: &mut UnixStream,
    limit: u64,
) -> io::Result<T> {
    let mut bytes = Vec::new();
    stream.take(limit + 1).read_to_end(&mut bytes).await?;

    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "worker message too large",
        ));
    }

    serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

async fn send_response(
    stream: &mut UnixStream,
    response: Response,
) -> io::Result<()> {
    let bytes = serde_json::to_vec(&response).map_err(io::Error::other)?;

    stream.write_all(&bytes).await?;
    stream.shutdown().await
}

pub async fn request(socket: &Path, request: Request) -> io::Result<()> {
    let parent = socket
        .parent()
        .ok_or_else(|| io::Error::other("invalid socket path"))?;

    check_directory(parent).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "worker not running; start `tuna serve` first",
            )
        } else {
            error
        }
    })?;

    let mut stream = UnixStream::connect(socket).await.map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "cannot connect to worker; start `tuna serve` first: {error}"
            ),
        )
    })?;

    let bytes = serde_json::to_vec(&request).map_err(io::Error::other)?;
    stream.write_all(&bytes).await?;
    stream.shutdown().await?;

    let response: Response = tokio::time::timeout(
        Duration::from_secs(180),
        read_message(&mut stream, 16 * 1024 * 1024),
    )
    .await
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "worker did not reply within 180 seconds",
        )
    })??;

    match response {
        Response::Ok(value) => {
            if !value.is_null() {
                let mut stdout = io::stdout().lock();
                serde_json::to_writer_pretty(&mut stdout, &value)
                    .map_err(io::Error::other)?;
                writeln!(stdout)?;
            }

            Ok(())
        }
        Response::Error(error) => Err(io::Error::other(error)),
    }
}

struct Worker {
    options: serve::Args,
    webdriver: String,
    driver: Option<WebDriver>,
}

fn session_lost(error: &WebDriverError) -> bool {
    matches!(
        error.as_inner(),
        WebDriverErrorInner::InvalidSessionId(_)
            | WebDriverErrorInner::NoSuchWindow(_)
    )
}

impl Worker {
    async fn close(&mut self) -> WebDriverResult<()> {
        if let Some(driver) = self.driver.take() {
            driver.quit().await?;
        }

        Ok(())
    }

    async fn ensure_authenticated(&mut self) -> WebDriverResult<()> {
        if self.driver.is_none() {
            self.open_browser().await?;
        }

        let result = self.authenticate().await;
        if !result.as_ref().is_err_and(session_lost) {
            return result;
        }

        let _ = self.close().await;
        self.open_browser().await?;
        self.authenticate().await
    }

    async fn open_browser(&mut self) -> WebDriverResult<()> {
        let mut caps = DesiredCapabilities::chrome();
        if !self.options.headed {
            caps.set_headless()?;
        }

        let driver = WebDriver::new(&self.webdriver, caps).await?;
        self.driver = Some(driver);

        tracing::info!("Opened browser session");
        Ok(())
    }

    async fn authenticate(&self) -> WebDriverResult<()> {
        let driver = self.driver.as_ref().expect("browser has been opened");

        tuna::tiss::login::ensure_authenticated(
            driver,
            &self.options.username,
            &self.options.password,
            || self.options.totp_code().map_err(WebDriverError::IoError),
        )
        .await
    }

    async fn execute(&mut self, request: Request) -> WebDriverResult<Value> {
        if matches!(request, Request::Stop) {
            self.close().await?;
            return Ok(Value::Null);
        }

        self.ensure_authenticated().await?;
        let driver = self.driver.as_ref().unwrap();

        let result: WebDriverResult<Value> = async {
            Ok(match request {
                Request::Login | Request::Stop => Value::Null,
                Request::Messages => serde_json::to_value(
                    tuna::tiss::messages::messages(driver).await?,
                )?,
                Request::Programmes => serde_json::to_value(
                    tuna::tiss::programmes::programmes(driver).await?,
                )?,
                Request::CourseGet { semester, course } => {
                    serde_json::to_value(
                        tuna::tiss::courses::get(driver, &semester, &course)
                            .await?,
                    )?
                }
                Request::CourseExams { semester, course } => {
                    serde_json::to_value(
                        tuna::tiss::courses::exams::list(
                            driver, &semester, &course,
                        )
                        .await?,
                    )?
                }
                Request::GroupsList { semester, course } => {
                    serde_json::to_value(
                        tuna::tiss::list_groups::list_groups(
                            driver, &semester, &course,
                        )
                        .await?,
                    )?
                }
                Request::GroupsRegister {
                    semester,
                    course,
                    group,
                } => serde_json::to_value(
                    tuna::tiss::register_group::register_group(
                        driver, &semester, &course, &group,
                    )
                    .await?,
                )?,
            })
        }
        .await;

        if result.as_ref().is_err_and(session_lost) {
            let _ = self.close().await;
        }

        result
    }
}

pub async fn serve(socket: &Path, options: serve::Args) -> io::Result<()> {
    let endpoint = WorkerSocket::bind(socket).await?;
    let mut terminate = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )?;
    let mut interrupt = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::interrupt(),
    )?;

    let mut chromedriver = if options.webdriver.is_none() {
        let driver = tokio::select! {
            driver = ChromeDriver::start() => driver?,
            _ = interrupt.recv() => return Ok(()),
            _ = terminate.recv() => return Ok(()),
        };
        Some(driver)
    } else {
        None
    };

    let webdriver = options
        .webdriver
        .clone()
        .unwrap_or_else(|| chromedriver.as_ref().unwrap().url.clone());
    let mut worker = Worker {
        options,
        webdriver,
        driver: None,
    };

    tracing::info!(socket = %socket.display(), "Worker ready");

    let result: io::Result<()> = async {
        loop {
            let mut stream = tokio::select! {
                connection = endpoint.listener.accept() => connection?.0,
                _ = interrupt.recv() => break,
                _ = terminate.recv() => break,
                status = async {
                    match &mut chromedriver {
                        Some(driver) => driver.child.wait().await,
                        None => std::future::pending().await,
                    }
                } => {
                    return Err(io::Error::other(format!(
                        "chromedriver exited unexpectedly: {}", status?
                    )));
                },
            };

            let incoming = tokio::time::timeout(
                Duration::from_secs(10),
                read_message(&mut stream, 64 * 1024),
            )
            .await;

            let request = match incoming {
                Ok(Ok(request)) => request,
                error => {
                    let message = match error {
                        Ok(Err(error)) => error.to_string(),
                        _ => "worker request timed out".into(),
                    };

                    let _ = send_response(
                        &mut stream,
                        Response::Error(message),
                    )
                    .await;

                    continue;
                }
            };

            let stopping = matches!(request, Request::Stop);
            let response = match worker.execute(request).await {
                Ok(value) => Response::Ok(value),
                Err(error) => {
                    tracing::warn!(error = %error, "Worker operation failed");
                    Response::Error(error.to_string())
                }
            };

            if let Err(error) = send_response(&mut stream, response).await {
                tracing::debug!(error = %error, "Client disconnected before receiving response");
            }

            if stopping {
                break;
            }
        }

        Ok(())
    }
    .await;

    let cleanup = worker.close().await.map_err(io::Error::other);
    let driver_cleanup = match &mut chromedriver {
        Some(driver) => driver.child.kill().await,
        None => Ok(()),
    };

    tracing::info!("Worker stopped");
    result.and(cleanup).and(driver_cleanup)
}
