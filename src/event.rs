use crate::request::Response;
use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    /// A keyboard event from the user.
    Key(KeyEvent),
    /// An HTTP response arriving from a background task.
    /// This variant is wired up in issue #5 when we add the mpsc channel.
    HttpResponse(Response),
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
            // Non-key events (terminal resize, mouse, focus change) are
            // silently dropped — we don't need them yet.
        }
    }
}
