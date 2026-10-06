use crate::worker::{self, Request};
use std::path::Path;

#[tracing::instrument(name = "tuwel.login.command", skip(socket))]
pub async fn run(socket: &Path) -> std::io::Result<()> {
    worker::request(socket, Request::Login).await
}
