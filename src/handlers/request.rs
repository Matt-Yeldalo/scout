use crate::{
    app::{App, HeaderField, InputMode, RequestTab},
    request::{AuthKind, HttpMethod},
};
use crossterm::event::{KeyCode, KeyEvent};
use tui_textarea::{CursorMove, TextArea};

impl App {
    // Normal Mode
    pub(crate) fn handle_request_builder_normal(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('s') => self.handle_send_request(),
            // Left
            KeyCode::Left => self.handle_normal_left(),
            KeyCode::Char('h') => self.prev_tab(),
            // Right
            KeyCode::Right => self.handle_normal_right(),
            KeyCode::Char('l') => self.next_tab(),
            // Up
            KeyCode::Up | KeyCode::Char('k') => self.handle_normal_up(key),
            // Down
            KeyCode::Down | KeyCode::Char('j') => self.handle_normal_down(key),
            KeyCode::Char('m') => {
                self.active_request.method = match self.active_request.method {
                    HttpMethod::Get => HttpMethod::Post,
                    HttpMethod::Post => HttpMethod::Get,
                };
            }
            KeyCode::Char('i') => match self.active_tab {
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
            KeyCode::Char('a') => match self.active_tab {
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
        }
    }

    fn handle_normal_left(&mut self) {
        match self.active_tab {
            RequestTab::Headers => match self.editing_header_field {
                Some(HeaderField::Value) => {
                    self.editing_header_field = Some(HeaderField::Key);
                    self.headers_table_state.select_column(Some(0));
                }
                None => {
                    self.headers_table_state.select_column(Some(0));
                }
                _ => {}
            },
            RequestTab::Auth => match self.auth_kind {
                AuthKind::Bearer => {
                    self.auth_kind = AuthKind::Basic;
                }
                AuthKind::Basic => {
                    self.auth_kind = AuthKind::ApiKey;
                }
                AuthKind::ApiKey => {
                    self.auth_kind = AuthKind::None;
                }
                AuthKind::None => {
                    self.auth_kind = AuthKind::Bearer;
                }
            },
            _ => {}
        }
    }

    fn handle_normal_right(&mut self) {
        match self.active_tab {
            RequestTab::Headers => match self.editing_header_field {
                Some(HeaderField::Key) => {
                    self.editing_header_field = Some(HeaderField::Value);
                    self.headers_table_state.select_column(Some(1));
                }
                None => {
                    self.headers_table_state.select_column(Some(1));
                }
                _ => {}
            },
            RequestTab::Auth => match self.auth_kind {
                AuthKind::Bearer => {
                    self.auth_kind = AuthKind::None;
                }
                AuthKind::Basic => {
                    self.auth_kind = AuthKind::Bearer;
                }
                AuthKind::ApiKey => {
                    self.auth_kind = AuthKind::Basic;
                }
                AuthKind::None => {
                    self.auth_kind = AuthKind::ApiKey;
                }
            },
            _ => {}
        }
    }

    fn handle_normal_up(&mut self, key: KeyEvent) {
        match self.active_tab {
            RequestTab::Headers => {
                if self.selected_header_row > 0 {
                    self.selected_header_row -= 1;
                    self.headers_table_state.select_previous();
                }
            }
            _ => {}
        }
    }

    fn handle_normal_down(&mut self, key: KeyEvent) {
        match self.active_tab {
            RequestTab::Headers => {
                if self.selected_header_row < self.active_request.headers.len() - 1 {
                    self.selected_header_row += 1;
                    self.headers_table_state.select_next();
                }
            }
            _ => {}
        }
    }

    // Insert Mode
    pub(crate) fn handle_request_builder_insert(&mut self, key: KeyEvent) {
        match self.active_tab {
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
            RequestTab::Body => self.handle_insert_body(key),
            RequestTab::Auth => self.handle_insert_auth(key),
        }
    }

    fn handle_insert_auth(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.update_request();
            }
            KeyCode::Char(c) => match self.auth_kind {
                AuthKind::Basic => self.auth_draft.basic_username.push(c),
                AuthKind::Bearer => self.auth_draft.bearer_token.push(c),
                AuthKind::ApiKey => self.auth_draft.api_key_value.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.auth_kind {
                AuthKind::Basic => {
                    self.auth_draft.basic_username.pop();
                }
                AuthKind::Bearer => {
                    self.auth_draft.bearer_token.pop();
                }
                AuthKind::ApiKey => {
                    self.auth_draft.api_key_value.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_insert_body(&mut self, key: KeyEvent) {
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
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Normal;
                            self.editing_header_field = None;
                        }
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
        } else {
            // If no field is being edited, start editing the key of the selected row.
            self.editing_header_field = Some(HeaderField::Key);
        }
    }

    pub fn update_request(&mut self) {
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
}
