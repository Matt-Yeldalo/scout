use crate::request::Response;
use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::fmt;
use std::time::Duration;

pub enum AppEvent {
    /// A keyboard event from the user.
    Key(KeyEvent),
    /// An HTTP response arriving from a background task.
    /// This variant is wired up in issue #5 when we add the mpsc channel.
    HttpResponse(Response),
}

impl fmt::Display for AppEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppEvent::Key(key) => write!(f, "Key({:?})", key),
            AppEvent::HttpResponse(_) => write!(f, "HttpResponse(...)"),
        }
    }
}

/// Block until a keyboard event arrives, then return it.
///
/// We use `poll` with a short timeout instead of reading directly.
/// This avoids spinning the CPU when there's no input, and it leaves the
/// door open to also check the HTTP response channel in issue #5 without
/// restructuring the whole loop.
pub fn next() -> Result<AppEvent> {
    loop {
        // `poll` blocks for up to `timeout`. Returns `true` immediately if
        // an event is already waiting, or `false` once the duration elapses.
        if event::poll(Duration::from_millis(250))? {
            // `read` will not block here because `poll` just confirmed an
            // event is ready.
            if let Event::Key(key) = event::read()? {
                return Ok(AppEvent::Key(key));
            }
        }
    }
}
