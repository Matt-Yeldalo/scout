use crate::{
    app::{App, Collection, CollectionState, Focus, InputMode, RequestTab},
    request::{Auth, AuthDraft},
};
use crossterm::event::{KeyCode, KeyEvent};
use uuid::Uuid;

impl App {
    pub(crate) fn handle_collections_normal(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.handle_normal_enter(),
            KeyCode::Char('s') => {
                self.update_request();
                self.local_store
                    .save_all(self.collections.clone())
                    .unwrap_or_else(|e| {
                        self.error_message = Some(format!("Error saving collections: {}", e));
                    });
                // TODO: message to user - need to think about generic way to do this
                // this will be used for other messages as well, like errors, etc.
            }
            KeyCode::Char('a') => {
                if let Some(selected_collection) = self.collection_list_state.selected() {
                    if let Some(collection) = self.collections.get_mut(selected_collection) {
                        collection.requests.push(self.active_request.clone());
                    }
                }
            }
            KeyCode::Char('n') => {
                let id = Uuid::new_v4().to_string();
                self.collections.push(Collection {
                    id,
                    name: "New Collection".to_string(),
                    requests: Vec::new(),
                });
                self.input_mode = InputMode::Insert;
                self.update_collection_list_state(Some(self.collections.len() - 1));
            }
            KeyCode::Left => match self.collection_ui_state {
                CollectionState::RequestList(_) => {
                    self.collection_ui_state = CollectionState::CollectionList;
                    self.selected_request = None;
                }
                _ => {}
            },
            KeyCode::Up | KeyCode::Char('k') => match self.collection_ui_state {
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
                _ => {}
            },
            KeyCode::Down | KeyCode::Char('j') => match self.collection_ui_state {
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
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_normal_enter(&mut self) {
        match self.collection_ui_state {
            CollectionState::CollectionList => {
                if let Some(selected_collection) = self.collection_list_state.selected() {
                    self.collection_ui_state = CollectionState::RequestList(selected_collection);
                    self.request_list_state.select(Some(0));
                }
            }
            CollectionState::RequestList(collection_index) => {
                if let Some(selected_request) = self.request_list_state.selected() {
                    if let Some(collection) = self.collections.get(collection_index) {
                        if let Some(request) = collection.requests.get(selected_request) {
                            self.active_request = request.clone();
                            self.auth_draft = AuthDraft::from(&self.active_request.auth);
                            self.auth_kind = self.auth_draft.kind.clone();
                            self.active_tab = RequestTab::Url;
                            self.focus = Focus::RequestBuilder;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn handle_collections_insert(&mut self, key: KeyEvent) {
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

    fn update_collection_list_state(&mut self, collection_index: Option<usize>) {
        if let Some(index) = collection_index {
            self.collection_list_state.select(Some(index));
        } else {
            self.collection_list_state.select(None);
        }
    }
}
