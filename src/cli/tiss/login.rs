use crate::worker::{self, Request};
use std::path::Path;

pub async fn run(socket: &Path) -> std::io::Result<()> {
    worker::request(socket, Request::Login).await
}
