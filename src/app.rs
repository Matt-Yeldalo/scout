use crate::event::AppEvent;
use crate::http::send;
use crate::request::{Collection, Request, Response};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            input_mode: InputMode::Normal,
            focus: Focus::Collections,
            active_tab: RequestTab::Url,
            collections: Vec::new(),
            selected_collection: None,
            selected_request: None,
            active_request: Request::default(),
            response: None,
            is_loading: false,
            error_message: None,
        }
    }

    /// Route an incoming event to the right handler based on current state.
    pub fn update(&mut self, event: AppEvent) {
        self.error_message = Some(event.to_string());
        match event {
            AppEvent::Key(key) => self.handle_key(key),
            // HTTP response arrives from the background task in issue #5.
            AppEvent::HttpResponse(response) => {
                self.response = Some(response);
                self.is_loading = false;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Dispatch to the right handler depending on which mode we're in.
        // This is the core of the vim-style modal input model.
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Insert => self.handle_insert_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            // Quit. Raw mode intercepts Ctrl-C, so we handle it manually.
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Tab cycles keyboard focus between the three panels.
            KeyCode::Tab => self.cycle_focus(),

            // Enter Insert mode so the user can type into fields.
            KeyCode::Char('i') | KeyCode::Enter => {
                self.input_mode = InputMode::Insert;
            }

            KeyCode::Char('s') => self.handle_send_request(),

            // h / l switch request builder tabs, but only when that panel is focused.
            KeyCode::Char('h') if self.focus == Focus::RequestBuilder => self.prev_tab(),
            KeyCode::Char('l') if self.focus == Focus::RequestBuilder => self.next_tab(),

            _ => {}
        }
    }

    fn handle_send_request(&mut self) {
        self.is_loading = true;
        let request = self.active_request.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let response = send(request).await.unwrap_or_else(|e| Response {
                status: 0,
                status_text: "Error".to_string(),
                duration_ms: 0,
                headers: std::collections::HashMap::new(),
                body: format!("Error: {}", e),
            });
            tx.send(AppEvent::HttpResponse(response)).await.unwrap();
        });
    }

    fn handle_insert_key(&mut self, key: KeyEvent) {
        match key.code {
            // Esc always exits Insert mode, returning to Normal.
            KeyCode::Esc => self.input_mode = InputMode::Normal,

            _ => match self.focus {
                Focus::Collections => {}
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Url => {
                        let url = &mut self.active_request.url;
                        match key.code {
                            KeyCode::Char(c) => url.push(c),
                            // NOTE: This may be moved up in the chain depending on what the other
                            // focus panels need
                            KeyCode::Enter => {
                                self.input_mode = InputMode::Normal;
                            }
                            KeyCode::Backspace => {
                                url.pop();
                            }
                            _ => {}
                        }
                    }
                    RequestTab::Headers => {}
                    RequestTab::Body => {}
                    RequestTab::Auth => {}
                },
                Focus::Response => {}
                _ => {}
            },
        }
    }

    /// Advance focus to the next panel (wraps around).
    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Collections => Focus::RequestBuilder,
            Focus::RequestBuilder => Focus::Response,
            Focus::Response => Focus::Collections,
        };
    }

    fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            RequestTab::Url => RequestTab::Headers,
            RequestTab::Headers => RequestTab::Body,
            RequestTab::Body => RequestTab::Auth,
            RequestTab::Auth => RequestTab::Url, // wrap around
        };
    }

    fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            RequestTab::Url => RequestTab::Auth, // wrap around
            RequestTab::Headers => RequestTab::Url,
            RequestTab::Body => RequestTab::Headers,
            RequestTab::Auth => RequestTab::Body,
        };
    }
}
