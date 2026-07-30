use crate::request::Response;
use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::fmt;

pub enum AppEvent {
    Key(KeyEvent),
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

pub async fn next(
    receiver: &mut tokio::sync::mpsc::Receiver<AppEvent>,
    event_stream: &mut crossterm::event::EventStream,
) -> Result<AppEvent> {
    tokio::select! {
        // maybe_event = event_stream.try_next() => {
        maybe_event = futures::StreamExt::next(event_stream) => {
            if let Some(Ok(Event::Key(key))) = maybe_event {
                return Ok(AppEvent::Key(key));
            } else{
                return Err(anyhow::anyhow!("Failed to read event"));
            }
        }
        maybe_response = receiver.recv() => {
            if let Some(response) = maybe_response {
                return Ok(response);
            } else {
                return Err(anyhow::anyhow!("Channel closed"));
            }
        }
    }
}
