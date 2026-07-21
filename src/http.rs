use crate::request::{Request, Response};
use anyhow::Result;

/// Fire an HTTP request and return the response.
/// Intended to be called inside a `tokio::spawn` task.
pub async fn send(request: Request) -> Result<Response> {
    todo!()
}
