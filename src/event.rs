use crate::request::Response;
use anyhow::Result;
use crossterm::event::{Event, KeyEvent, MouseEvent};
use tokio::time::Interval;
use std::fmt;

pub enum AppEvent {
    Key(KeyEvent),
    HttpResponse(Response),
    Mouse(MouseEvent),
    Tick,
}

impl fmt::Display for AppEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppEvent::Tick => write!(f, "Tick"),
            AppEvent::Key(key) => write!(f, "Key({:?})", key),
            AppEvent::HttpResponse(_) => write!(f, "HttpResponse(...)"),
            AppEvent::Mouse(mouse_event) => write!(f, "Mouse({:?})", mouse_event),
        }
    }
}

pub async fn next(
    receiver: &mut tokio::sync::mpsc::Receiver<AppEvent>,
    event_stream: &mut crossterm::event::EventStream,
    interval: &mut Interval,
) -> Result<AppEvent> {
    tokio::select! {
        maybe_tick = interval.tick() => {
            Ok(AppEvent::Tick)
        }
        maybe_event = futures::StreamExt::next(event_stream) => {
            match maybe_event {
                Some(Ok(Event::Key(key))) => return Ok(AppEvent::Key(key)),
                Some(Ok(Event::Mouse(mouse_event))) => return Ok(AppEvent::Mouse(mouse_event)),
                Some(Err(e)) => return Err(anyhow::anyhow!("Error reading event: {}", e)),
                None => return Err(anyhow::anyhow!("Event stream closed")),
                // _ => return Err(anyhow::anyhow!("Unexpected event type")),
                _ => Ok(AppEvent::Key(KeyEvent::from(crossterm::event::KeyCode::Null))),
            }
            // if let Some(Ok(Event::Key(key))) = maybe_event {
            //     return Ok(AppEvent::Key(key));
            // } else{
            //     return Ok(AppEvent::Key(KeyEvent::from(crossterm::event::KeyCode::Null)));
            // }
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
