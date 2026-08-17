use crate::{
    event::AppEvent,
    http::send,
    request::{AuthDraft, AuthKind},
};
use crate::{
    local_store::{LocalStore, StoreKey},
    request::{Request, Response},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::{ListState, ScrollbarState, TableState};
use serde::{Deserialize, Serialize};
use tui_textarea::TextArea;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Focus {
    #[default]
    Collections,
    RequestBuilder,
    Response,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum RequestTab {
    #[default]
    Url,
    Headers,
    Body,
    Auth,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub requests: Vec<Request>,
}
impl std::fmt::Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}, name: {}, requests: {}",
            self.id,
            self.name,
            self.requests.len()
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum CollectionState {
    #[default]
    CollectionList,
    RequestList(usize), // index of the selected collection
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthField {
    BearerToken,
    BasicUsername,
    BasicPassword,
    ApiKeyHeader,
    ApiKeyValue,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum HeaderField {
    #[default]
    Key,
    Value,
}

#[derive(Clone)]
pub struct App {
    pub local_store: LocalStore,
    pub debug: bool,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub focus: Focus,
    pub active_tab: RequestTab,
    pub collections: Vec<Collection>,
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
    pub auth_draft: AuthDraft,
    pub auth_kind: AuthKind,
    pub editing_auth_field: Option<AuthField>,
}

impl App {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<AppEvent>,
        headers_table_state: TableState,
    ) -> Self {
        let local_store = LocalStore::new();
        Self {
            local_store: local_store.clone(),
            sender,
            headers_table_state,
            debug: true,
            should_quit: false,
            input_mode: InputMode::Normal,
            focus: Focus::Collections,
            active_tab: RequestTab::Url,
            collections: local_store.load_all().unwrap_or_else(|_| Vec::new()),
            selected_request: None,
            active_request: Request::new(),
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
            auth_draft: AuthDraft::default(),
            auth_kind: AuthKind::None,
            editing_auth_field: None,
        }
    }

    pub fn handle_send_request(&mut self) {
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

    pub fn update(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::HttpResponse(response) => {
                let collection = self
                    .collections
                    .get(self.active_collection.unwrap_or(0))
                    .cloned();
                let collection = match collection {
                    Some(c) => c,
                    None => {
                        self.error_message =
                            Some("No active collection to save response".to_string());
                        return;
                    }
                };
                self.response = Some(response);
                self.is_loading = false;

                let response = self.response.clone().unwrap();

                // maybe consider making key private again and letting local state handle conversion
                let key = StoreKey::from(&collection, &self.active_request);
                self.local_store.update_responses(key, &response);
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Insert => self.handle_insert_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.cycle_focus();
                self.tab_changed_callback();
            }
            _ => {}
        }

        match self.focus {
            Focus::Collections => self.handle_collections_normal(key),
            Focus::RequestBuilder => self.handle_request_builder_normal(key),
            Focus::Response => self.handle_response_normal(key),
        }
    }

    fn handle_insert_key(&mut self, key: KeyEvent) {
        match self.focus {
            Focus::Collections => self.handle_collections_insert(key),
            Focus::RequestBuilder => self.handle_request_builder_insert(key),
            Focus::Response => {}
        }
    }

    fn tab_changed_callback(&mut self) {}
    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Collections => Focus::RequestBuilder,
            Focus::RequestBuilder => Focus::Response,
            Focus::Response => Focus::Collections,
        };
    }
    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            RequestTab::Url => RequestTab::Headers,
            RequestTab::Headers => RequestTab::Body,
            RequestTab::Body => RequestTab::Auth,
            RequestTab::Auth => RequestTab::Url, // wrap around
        };
    }
    pub fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            RequestTab::Url => RequestTab::Auth, // wrap around
            RequestTab::Headers => RequestTab::Url,
            RequestTab::Body => RequestTab::Headers,
            RequestTab::Auth => RequestTab::Body,
        };
    }
}
