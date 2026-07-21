use crate::request::{Collection, Request, Response};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

/// Which panel currently has keyboard focus.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Focus {
    #[default]
    Collections,
    RequestBuilder,
    Response,
}

/// Which tab is active in the request builder.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum RequestTab {
    #[default]
    Url,
    Headers,
    Body,
    Auth,
}

pub struct App {
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub focus: Focus,
    pub active_tab: RequestTab,
    pub collections: Vec<Collection>,
    pub selected_collection: Option<usize>,
    pub selected_request: Option<usize>,
    pub active_request: Request,
    pub response: Option<Response>,
    pub is_loading: bool,
}

impl App {
    pub fn new() -> Self {
        todo!()
    }

    /// Handle an incoming event and update state accordingly.
    pub fn update(&mut self, event: crate::event::AppEvent) {
        todo!()
    }
}
