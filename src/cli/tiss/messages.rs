use crate::worker::{self, Request};
use std::path::Path;

#[tracing::instrument(name = "tiss.messages.command", skip_all)]
pub async fn run(socket: &Path) -> std::io::Result<()> {
    worker::request(socket, Request::Messages).await
}
