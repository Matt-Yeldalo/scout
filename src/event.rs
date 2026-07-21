use crate::request::Response;
use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

pub enum AppEvent {
    /// A keyboard event from the user.
    Key(KeyEvent),
    /// An HTTP response received from a background task.
    HttpResponse(Response),
}

/// Block until either a keyboard event arrives or an HTTP response is ready.
pub fn next(response_rx: &mut mpsc::Receiver<Response>) -> Result<AppEvent> {
    todo!()
}
