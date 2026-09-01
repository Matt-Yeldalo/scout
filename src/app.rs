use crate::{
    event::AppEvent,
    http::send,
    input::TextInput,
    notifier::{Notifier, NotifyType},
    request::{AuthDraft, AuthKind},
    ui::UiDimensions,
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
pub enum EditContext {
    #[default]
    None,
    RequestUrl,
    VariableKey,
    HeaderKey(usize),
    HeaderValue(usize),
    AuthField(AuthField),
    VariableValue,
}

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
    Variables,
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
    pub variables: Vec<(String, String)>,
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
    pub show_help: bool,
    pub error_message: Option<String>,
    pub sender: tokio::sync::mpsc::Sender<AppEvent>,
    pub selected_header_row: usize,
    pub editing_header_field: Option<HeaderField>,
    pub headers_table_state: TableState,
    pub response_scrollbar: ScrollbarState,
    pub response_scroll: u16,
    pub active_collection: Option<usize>,
    pub collection_list_state: ListState,
    pub request_list_state: ListState,
    pub expanded_collections: Vec<usize>,
    pub collection_ui_state: CollectionState,
    pub body_editor: TextArea<'static>,
    pub auth_draft: AuthDraft,
    pub auth_kind: AuthKind,
    pub editing_auth_field: Option<AuthField>,
    pub clipboard: arboard::Clipboard,
    pub notifier: Notifier,
    pub ui_dimensions: UiDimensions,
    pub variables_table_state: TableState,
    pub auth_basic_table_state: TableState,
    pub edit_context: EditContext,
    pub active_input: TextInput,
}

impl App {
    pub fn new(
        sender: tokio::sync::mpsc::Sender<AppEvent>,
        headers_table_state: TableState,
    ) -> Self {
        let mut auth_basic_table_state = TableState::default();
        auth_basic_table_state.select_first();
        auth_basic_table_state.select_last_column();

        let local_store = LocalStore::new();
        let mut notifier = Notifier::default();
        let collections = match local_store.load_all() {
            Ok(collections) => {
                if collections.is_empty() {
                    eprintln!("No collections found. Please create a collection first.");
                }
                collections
            }
            Err(e) => {
                notifier.notify(
                    NotifyType::Error,
                    format!("Failed to load collections: {}", e),
                );
                Vec::new()
            }
        };
        let clipboard = arboard::Clipboard::new().unwrap_or_else(|_| {
            eprintln!("Failed to initialize clipboard");
            std::process::exit(1);
        });
        let mut collection_list_state = ListState::default();
        if collections.is_empty() {
            collection_list_state.select(None);
        } else {
            collection_list_state.select(Some(0));
        }
        Self {
            local_store: local_store.clone(),
            sender,
            headers_table_state,
            auth_basic_table_state,
            collections,
            collection_list_state,
            clipboard,
            notifier,
            debug: true,
            should_quit: false,
            is_loading: false,
            show_help: false,
            selected_header_row: 0,
            response_scroll: 0,
            selected_request: None,
            response: None,
            error_message: None,
            editing_header_field: None,
            active_collection: None,
            auth_kind: AuthKind::None,
            editing_auth_field: None,
            edit_context: EditContext::None,
            input_mode: InputMode::Normal,
            focus: Focus::Collections,
            active_tab: RequestTab::Url,
            active_request: Request::new(),
            response_scrollbar: ScrollbarState::default(),
            request_list_state: ListState::default(),
            expanded_collections: Vec::new(),
            collection_ui_state: CollectionState::default(),
            body_editor: TextArea::default(),
            auth_draft: AuthDraft::default(),
            ui_dimensions: UiDimensions::default(),
            variables_table_state: TableState::default(),
            active_input: TextInput::new(),
        }
    }

    pub fn handle_send_request(&mut self) {
        self.is_loading = true;
        let mut request = self.active_request.clone();
        let populate_result = request.populate_placeholders(
            &self
                .get_active_collection()
                .map(|c| c.variables.iter().cloned().collect())
                .unwrap_or_default(),
        );
        match populate_result {
            Ok(_) => {}
            Err(e) => {
                self.is_loading = false;
                self.notifier.notify(NotifyType::Error, e);
                return;
            }
        }
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
            AppEvent::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::HttpResponse(response) => {
                let collection = self
                    .collections
                    .get(self.active_collection.unwrap_or(0))
                    .cloned();
                let collection = match collection {
                    Some(c) => c,
                    None => {
                        self.notifier.notify(
                            NotifyType::Error,
                            "No active collection to save response".to_string(),
                        );
                        // self.error_message =
                        //     Some("No active collection to save response".to_string());
                        return;
                    }
                };
                self.response = Some(response);
                self.is_loading = false;

                let response = self.response.clone().unwrap();

                // maybe consider making key private again and letting local state handle conversion
                let key = StoreKey::from(&collection, &self.active_request);
                let result = self.local_store.update_responses(key, &response);
                if let Err(e) = result {
                    // self.error_message = Some(format!("Failed to save response: {}", e));
                    self.notifier
                        .notify(NotifyType::Error, format!("Failed to save response: {}", e));
                }
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: crossterm::event::MouseEvent) {
        match self.focus {
            // Focus::Collections => handle_collections_mouse(self, mouse_event),
            // Focus::RequestBuilder => handle_request_builder_mouse(self, mouse_event),
            Focus::Response => self.handle_response_mouse(mouse_event),
            _ => {}
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
            KeyCode::Char('h') => self.show_help = !self.show_help,
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
            Focus::Variables => self.handle_variables_normal(key),
        }
    }

    fn handle_insert_key(&mut self, key: KeyEvent) {
        match self.focus {
            Focus::Collections => self.handle_collections_insert(key),
            Focus::RequestBuilder => self.handle_request_builder_insert(key),
            Focus::Response => {}
            Focus::Variables => self.handle_variables_insert(key),
        }
    }

    pub fn get_active_collection(&self) -> Option<&Collection> {
        self.collections.get(self.active_collection.unwrap_or(0))
    }

    pub fn get_active_collection_mut(&mut self) -> Option<&mut Collection> {
        let index = self.active_collection.unwrap_or(0);
        self.collections.get_mut(index)
    }

    fn tab_changed_callback(&mut self) {
        match self.focus {
            Focus::Variables => {
                self.variables_table_state.select(Some(0));
                self.edit_context = EditContext::VariableKey;
            }
            _ => {}
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Collections => Focus::RequestBuilder,
            Focus::RequestBuilder => Focus::Response,
            Focus::Response => Focus::Variables,
            Focus::Variables => Focus::Collections, // wrap around
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
