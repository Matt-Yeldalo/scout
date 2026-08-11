use crate::event::AppEvent;
use crate::http::send;
use crate::request::{Collection, HttpMethod, Request, Response};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::{ListState, ScrollbarState, TableState};
use tui_textarea::{CursorMove, Input, TextArea};

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

#[derive(Debug, Default, Clone, PartialEq)]
pub enum CollectionState {
    #[default]
    CollectionList,
    RequestList(usize), // index of the selected collection
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum HeaderField {
    #[default]
    Key,
    Value,
}

#[derive(Clone)]
pub struct App {
    pub debug: bool,
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
    pub sender: tokio::sync::mpsc::Sender<AppEvent>,
    pub selected_header_row: usize,
    pub editing_header_field: Option<HeaderField>,
    pub headers_table_state: TableState,
    pub response_scrollbar: ScrollbarState,
    pub active_collection: Option<usize>,
    pub collection_list_state: ListState,
    pub request_list_state: ListState,
    pub expanded_collections: Vec<usize>,
    pub collection_ui_state: CollectionState,
    pub body_editor: TextArea<'static>,
}

impl App {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<AppEvent>,
        headers_table_state: TableState,
    ) -> Self {
        Self {
            sender,
            headers_table_state,
            debug: true,
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
            selected_header_row: 0,
            editing_header_field: None,
            response_scrollbar: ScrollbarState::default(),
            active_collection: None,
            collection_list_state: ListState::default(),
            request_list_state: ListState::default(),
            expanded_collections: Vec::new(),
            collection_ui_state: CollectionState::default(),
            body_editor: TextArea::default(),
        }
    }

    pub fn update(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => self.handle_key(key, event),
            AppEvent::HttpResponse(response) => {
                self.response = Some(response);
                self.is_loading = false;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent, event: AppEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key, event),
            InputMode::Insert => self.handle_insert_key(key, event),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent, event: AppEvent) {
        match key.code {
            // Quit. Raw mode intercepts Ctrl-C, so we handle it manually.
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Tab cycles keyboard focus between the three panels.
            KeyCode::Tab => {
                self.cycle_focus();
                self.tab_changed_callback();
            }

            KeyCode::Enter => match self.focus {
                Focus::Collections => match self.collection_ui_state {
                    CollectionState::CollectionList => {
                        if let Some(selected_collection) = self.collection_list_state.selected() {
                            self.collection_ui_state =
                                CollectionState::RequestList(selected_collection);
                            self.request_list_state.select(Some(0));
                            // self.selected_request = Some(0);
                            // self.active_request = self.collections[selected_collection].requests[0].clone();
                        }
                    }
                    CollectionState::RequestList(collection_index) => {
                        if let Some(selected_request) = self.request_list_state.selected() {
                            if let Some(collection) = self.collections.get(collection_index) {
                                if let Some(request) = collection.requests.get(selected_request) {
                                    self.active_request = request.clone();
                                    self.active_tab = RequestTab::Url;
                                    self.focus = Focus::RequestBuilder;
                                }
                            }
                        }
                    }
                },
                _ => {}
            },

            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;

                match self.focus {
                    Focus::RequestBuilder => match self.active_tab {
                        RequestTab::Url => {}
                        RequestTab::Headers => {
                            // self.editing_header_field = Some(HeaderField::Key);
                            match self.headers_table_state.selected_column() {
                                Some(0) => self.editing_header_field = Some(HeaderField::Key),
                                Some(1) => self.editing_header_field = Some(HeaderField::Value),
                                _ => self.editing_header_field = Some(HeaderField::Key),
                            }
                        }
                        RequestTab::Body => {
                            if let Some(body) = &self.active_request.body {
                                self.body_editor = TextArea::from(body.lines());
                            }
                        }
                        RequestTab::Auth => {}
                    },
                    _ => {}
                }
            }

            KeyCode::Char('s') => self.handle_send_request(),

            KeyCode::Char('h') => match self.focus {
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Url => {}
                    RequestTab::Headers => self.prev_tab(),
                    RequestTab::Body => self.prev_tab(),
                    RequestTab::Auth => self.prev_tab(),
                },
                _ => {}
            },
            KeyCode::Char('l') => match self.focus {
                Focus::RequestBuilder => self.next_tab(),
                _ => {}
            },

            KeyCode::Char('m') => match self.focus {
                Focus::RequestBuilder => {
                    self.active_request.method = match self.active_request.method {
                        HttpMethod::Get => HttpMethod::Post,
                        HttpMethod::Post => HttpMethod::Get,
                    };
                }
                _ => {}
            },

            KeyCode::Char('a') => match self.focus {
                Focus::Collections => {
                    if let Some(selected_collection) = self.collection_list_state.selected() {
                        if let Some(collection) = self.collections.get_mut(selected_collection) {
                            collection.requests.push(self.active_request.clone());
                        }
                    }
                }
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Headers => {
                        self.active_request
                            .headers
                            .push(("".to_string(), "".to_string()));
                        self.selected_header_row = self.active_request.headers.len() - 1;
                        self.editing_header_field = Some(HeaderField::Key);
                        self.input_mode = InputMode::Insert;
                        self.headers_table_state
                            .select(Some(self.selected_header_row));
                    }
                    _ => {}
                },
                _ => {}
            },

            KeyCode::Char('n') => match self.focus {
                Focus::Collections => {
                    self.collections.push(Collection {
                        name: "New Collection".to_string(),
                        requests: Vec::new(),
                    });
                    self.input_mode = InputMode::Insert;
                    self.update_collection_list_state(Some(self.collections.len() - 1));
                }
                _ => {}
            },

            KeyCode::Right => match self.focus {
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Headers => match self.editing_header_field {
                        Some(HeaderField::Key) => {
                            self.editing_header_field = Some(HeaderField::Value);
                            // self.headers_table_state.select_next_column();
                            self.headers_table_state.select_column(Some(1));
                        }
                        None => {
                            self.headers_table_state.select_column(Some(1));
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            KeyCode::Left => match self.focus {
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Headers => match self.editing_header_field {
                        Some(HeaderField::Value) => {
                            self.editing_header_field = Some(HeaderField::Key);
                            self.headers_table_state.select_column(Some(0));
                            // self.headers_table_state.select_previous_column();
                        }
                        None => {
                            self.headers_table_state.select_column(Some(0));
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Focus::Collections => match self.collection_ui_state {
                    CollectionState::RequestList(_) => {
                        self.collection_ui_state = CollectionState::CollectionList;
                        self.selected_request = None;
                    }
                    _ => {}
                },
                _ => {}
            },
            KeyCode::Up | KeyCode::Char('k') => match self.focus {
                Focus::Collections => match self.collection_ui_state {
                    CollectionState::CollectionList => {
                        if let Some(selected) = self.collection_list_state.selected() {
                            if selected > 0 {
                                self.collection_list_state.select(Some(selected - 1));
                            }
                        }
                    }
                    CollectionState::RequestList(collection_index) => {
                        if let Some(selected) = self.request_list_state.selected() {
                            if selected > 0 {
                                self.request_list_state.select(Some(selected - 1));
                            }
                        }
                    }
                },
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Headers => {
                        if self.selected_header_row > 0 {
                            self.selected_header_row -= 1;
                            self.headers_table_state.select_previous();
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            KeyCode::Down | KeyCode::Char('j') => match self.focus {
                Focus::Collections => match self.collection_ui_state {
                    CollectionState::CollectionList => {
                        if let Some(selected) = self.collection_list_state.selected() {
                            if selected < self.collections.len() - 1 {
                                self.collection_list_state.select(Some(selected + 1));
                            }
                        } else if !self.collections.is_empty() {
                            self.collection_list_state.select(Some(0));
                        }
                    }
                    CollectionState::RequestList(collection_index) => {
                        if let Some(collection) = self.collections.get(collection_index) {
                            if let Some(selected) = self.request_list_state.selected() {
                                if selected < collection.requests.len() - 1 {
                                    self.request_list_state.select(Some(selected + 1));
                                }
                            } else if !collection.requests.is_empty() {
                                self.request_list_state.select(Some(0));
                            }
                        }
                    }
                },
                Focus::RequestBuilder => match self.active_tab {
                    RequestTab::Headers => {
                        if self.selected_header_row < self.active_request.headers.len() - 1 {
                            self.selected_header_row += 1;
                            self.headers_table_state.select_next();
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_send_request(&mut self) {
        self.is_loading = true;
        let request = self.active_request.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let response = send(request).await.unwrap_or_else(|e| Response {
                status: 0,
                status_text: "Error".to_string(),
                duration_ms: 0,
                headers: std::collections::HashMap::new(),
                body: format!("Error: {}", e),
            });
            sender.send(AppEvent::HttpResponse(response)).await.unwrap();
        });
    }

    fn handle_collection_toggle(&mut self, collection_index: usize) {
        if let Some(pos) = self
            .expanded_collections
            .iter()
            .position(|&i| i == collection_index)
        {
            self.expanded_collections.remove(pos);
        } else {
            self.expanded_collections.push(collection_index);
        }
    }

    fn update_collection_list_state(&mut self, collection_index: Option<usize>) {
        if let Some(index) = collection_index {
            self.collection_list_state.select(Some(index));
        } else {
            self.collection_list_state.select(None);
        }
    }

    fn handle_insert_key(&mut self, key: KeyEvent, event: AppEvent) {
        // Esc always exits Insert mode, returning to Normal.
        // KeyCode::Esc => self.input_mode = InputMode::Normal,
        match self.focus {
            Focus::Collections => self.handle_insert_collection(key),
            Focus::RequestBuilder => match self.active_tab {
                RequestTab::Url => {
                    let url = &mut self.active_request.url;
                    match key.code {
                        KeyCode::Char(c) => url.push(c),
                        // NOTE: This may be moved up in the chain depending on what the other
                        // focus panels need
                        KeyCode::Enter => {
                            self.input_mode = InputMode::Normal;
                            self.update_request();
                        }
                        KeyCode::Backspace => {
                            url.pop();
                        }
                        _ => {}
                    }
                }
                RequestTab::Headers => self.handle_insert_header(key),
                RequestTab::Body => self.handle_insert_body(key, event),
                RequestTab::Auth => {}
            },
            Focus::Response => {}
            _ => {}
        }
    }

    fn update_request(&mut self) {
        if let Some(selected_collection) = self.collection_list_state.selected() {
            if let Some(selected_request) = self.request_list_state.selected() {
                if let Some(collection) = self.collections.get_mut(selected_collection) {
                    if let Some(request) = collection.requests.get_mut(selected_request) {
                        *request = self.active_request.clone();
                    }
                }
            }
        }
    }

    fn handle_insert_collection(&mut self, key: KeyEvent) {
        match self.collection_ui_state {
            CollectionState::CollectionList => match key.code {
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.collections[self.collection_list_state.selected().unwrap()]
                        .requests
                        .push(self.active_request.clone());
                    let added_collection_index = self.collections.len() - 1;
                    self.update_collection_list_state(Some(added_collection_index));
                }
                KeyCode::Char(c) => {
                    if let Some(active_collection) = self.collection_list_state.selected() {
                        self.collections[active_collection].name.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(active_collection) = self.collection_list_state.selected() {
                        self.collections[active_collection].name.pop();
                    }
                }
                _ => {}
            },
            CollectionState::RequestList(_) => match key.code {
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    if let Some(active_collection) = self.collection_list_state.selected() {
                        if let Some(selected_request) = self.request_list_state.selected() {
                            self.collections[active_collection].requests[selected_request]
                                .name
                                .push(c);
                        }
                    }
                }
                KeyCode::Backspace => {
                    if let Some(active_collection) = self.collection_list_state.selected() {
                        if let Some(selected_request) = self.request_list_state.selected() {
                            self.collections[active_collection].requests[selected_request]
                                .name
                                .pop();
                        }
                    }
                }
                _ => {}
            },
        }
    }

    fn handle_insert_body(&mut self, key: KeyEvent, event: AppEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.active_request.body = Some(self.body_editor.lines().join("\n"));
                self.body_editor = TextArea::default();
                self.update_request();
            }
            KeyCode::Char(c) => {
                self.body_editor.insert_char(c);
            }
            KeyCode::Enter => {
                self.body_editor.insert_newline();
            }
            KeyCode::Backspace => {
                self.body_editor.delete_char();
            }
            KeyCode::Left => {
                self.body_editor.move_cursor(CursorMove::Back);
            }
            KeyCode::Right => {
                self.body_editor.move_cursor(CursorMove::Forward);
            }
            KeyCode::Up => {
                self.body_editor.move_cursor(CursorMove::Up);
            }
            KeyCode::Down => {
                self.body_editor.move_cursor(CursorMove::Down);
            }
            _ => {}
        }
    }

    fn handle_insert_header(&mut self, key: KeyEvent) {
        if let Some(field) = &self.editing_header_field {
            let headers = &mut self.active_request.headers;
            if let Some((header_key, value)) = headers.get_mut(self.selected_header_row) {
                match field {
                    HeaderField::Key => match key.code {
                        KeyCode::Backspace => {
                            header_key.pop();
                        }
                        KeyCode::Delete => {
                            headers.remove(self.selected_header_row);
                            if self.selected_header_row > 0 {
                                self.selected_header_row -= 1;
                            }
                            self.headers_table_state
                                .select(Some(self.selected_header_row));
                        }
                        // KeyCode::Up => {
                        //     if self.selected_header_row > 0 {
                        //         self.selected_header_row -= 1;
                        //         self.headers_table_state
                        //             .select(Some(self.selected_header_row));
                        //     }
                        // }
                        // KeyCode::Down => {
                        //     if self.selected_header_row < headers.len() - 1 {
                        //         self.selected_header_row += 1;
                        //         self.headers_table_state
                        //             .select(Some(self.selected_header_row));
                        //     }
                        // }
                        KeyCode::Left => {
                            self.editing_header_field = Some(HeaderField::Key);
                            self.headers_table_state.select_column(Some(0));
                        }
                        KeyCode::Right => {
                            self.editing_header_field = Some(HeaderField::Value);
                            self.headers_table_state.select_column(Some(1));
                        }
                        KeyCode::Enter => {
                            self.editing_header_field = Some(HeaderField::Value);
                            self.headers_table_state.select_next_column();
                        }
                        KeyCode::Char(c) => header_key.push(c),
                        _ => {}
                    },
                    HeaderField::Value => match key.code {
                        KeyCode::Backspace => {
                            value.pop();
                        }
                        KeyCode::Enter => {
                            self.editing_header_field = None;
                            self.input_mode = InputMode::Normal;
                        }
                        // KeyCode::Up => {
                        //     if self.selected_header_row > 0 {
                        //         self.selected_header_row -= 1;
                        //         self.headers_table_state
                        //             .select(Some(self.selected_header_row));
                        //     }
                        // }
                        // KeyCode::Down => {
                        //     if self.selected_header_row < headers.len() - 1 {
                        //         self.selected_header_row += 1;
                        //         self.headers_table_state
                        //             .select(Some(self.selected_header_row));
                        //     }
                        // }
                        KeyCode::Left => {
                            self.editing_header_field = Some(HeaderField::Key);
                            self.headers_table_state.select_column(Some(0));
                        }
                        KeyCode::Right => {
                            self.editing_header_field = Some(HeaderField::Value);
                            self.headers_table_state.select_column(Some(1));
                        }
                        KeyCode::Char(c) => value.push(c),
                        _ => {}
                    },
                }
            }
        }
    }

    fn tab_changed_callback(&mut self) {}

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
