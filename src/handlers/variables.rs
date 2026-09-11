use crate::{
    app::{App, EditContext, InputMode},
    notifier::NotifyType,
};
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub(crate) fn handle_variables_normal(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('l') | KeyCode::Right => self.move_right(),
            KeyCode::Char('h') | KeyCode::Left => self.move_left(),
            KeyCode::Char('i') => {
                self.input_mode = InputMode::Insert;
                match self.edit_context {
                    EditContext::VariableKey => {
                        let current_key = self.current_key().unwrap_or_default();
                        self.active_input.value = current_key;
                        self.active_input.cursor = self.active_input.value.len();
                    }
                    EditContext::VariableValue => {
                        let current_value = self.current_value().unwrap_or_default();
                        self.active_input.value = current_value;
                        self.active_input.cursor = self.active_input.value.len();
                    }
                    _ => {}
                }
                // let current_key = self.current_key().unwrap_or_default();
                // self.active_input.value = current_key;
                // self.active_input.cursor = self.active_input.value.len();
                // self.edit_context = EditContext::VariableKey
            }
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Insert;
                self.edit_context = EditContext::VariableKey;
                let active_collection = self.get_active_collection_mut();
                if let Some(c) = active_collection {
                    c.variables.push(("".to_string(), "".to_string()));
                    let last_index = c.variables.len() - 1;
                    self.variables_table_state.select(Some(last_index));
                }
            }
            KeyCode::Delete => {
                if let Some(i) = self.variables_table_state.selected() {
                    if let Some(c) = self.get_active_collection_mut() {
                        c.variables.remove(i);
                        let new_selection = if i >= c.variables.len() {
                            c.variables.len().checked_sub(1)
                        } else {
                            Some(i)
                        };
                        self.variables_table_state.select(new_selection);
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle_variables_insert(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => self.active_input.move_cursor_left(),
            KeyCode::Right => self.active_input.move_cursor_right(),
            KeyCode::Backspace => self.active_input.delete(),
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.edit_context = EditContext::VariableKey;
                self.active_input.value.clear();
                // Cancelling an edit on a still-blank row (e.g. one just added with [a]) removes it.
                if let Some(i) = self.variables_table_state.selected() {
                    if let Some((k, v)) = self.variables().get(i).cloned() {
                        if k.is_empty() && v.is_empty() {
                            if let Some(c) = self.get_active_collection_mut() {
                                c.variables.remove(i);
                                let new_selection = if i >= c.variables.len() {
                                    c.variables.len().checked_sub(1)
                                } else {
                                    Some(i)
                                };
                                self.variables_table_state.select(new_selection);
                            }
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(_) = self.variables_table_state.selected() {
                    match self.edit_context {
                        EditContext::VariableKey => self.update_key(self.active_input.value.clone()), 
                        EditContext::VariableValue =>  self.update_value(self.active_input.value.clone()),
                        _ => {}
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                if let Some(_) = self.variables_table_state.selected() {
                    self.active_input.insert(c);
                }
            }
            _ => {}
        }
    }

    fn move_down(&mut self) {
        if self.variables_table_state.selected().is_none() {
            self.variables_table_state.select(Some(0));
        } else {
            let i = self.variables_table_state.selected().unwrap();
            if i >= self.variables().len() - 1 {
                self.variables_table_state.select(Some(0));
            } else {
                self.variables_table_state.select(Some(i + 1));
            }
        }
    }

    fn move_up(&mut self) {
        if self.variables_table_state.selected().is_none() {
            self.variables_table_state.select(Some(0));
        } else {
            let i = self.variables_table_state.selected().unwrap();
            if i == 0 {
                let end = self.variables().len() - 1;
                self.variables_table_state.select(Some(end));
            } else {
                self.variables_table_state.select(Some(i - 1));
            }
        }
    }

    fn move_right(&mut self) {
        match self.edit_context {
            EditContext::VariableKey => {
                let current_value = self.current_value().unwrap_or_default();
                self.active_input.value = current_value;
                self.edit_context = EditContext::VariableValue;
            }
            _ => {}
        }
    }

    fn move_left(&mut self) {
        match self.edit_context {
            EditContext::VariableValue => {
                let current_key = self.current_key().unwrap_or_default();
                self.active_input.value = current_key;
                self.edit_context = EditContext::VariableKey;
            }
            _ => {}
        }
    }

    fn update_value(&mut self, new_value: String) {
        let Some(i) = self.variables_table_state.selected() else {
            return;
        };
        let Some(c) = self.get_active_collection_mut() else {
            return;
        };
        c.variables[i].1 = new_value;
    }

    fn update_key(&mut self, new_key: String) {
        let Some(i) = self.variables_table_state.selected() else {
            return;
        };
        let mut collection = self.get_active_collection_mut();
        // let Some(c) = self.get_active_collection_mut() else {
        //     return;
        // };
        let Some(c) = collection.as_mut() else {
            return;
        };
        let old_key = c.variables[i].0.clone();
        // let mut key = self.current_key().unwrap_or_default();
        // key.clone_from(&new_key);
        if old_key != new_key {
            if c.variables.iter().any(|(k, _)| k == &new_key) {
                self.notifier
                    .notify(NotifyType::Error, "Key already exists".to_string());
                return;
            }
            c.variables[i].0 = new_key;
        }
    }

    fn current_variable(&mut self) -> Option<(String, String)> {
        let i = self.variables_table_state.selected()?;
        self.variables().get(i).cloned()
    }

    fn current_key(&mut self) -> Option<String> {
        let i = self.variables_table_state.selected()?;
        self.variables().get(i).map(|(k, _)| k.clone())
    }

    fn current_value(&mut self) -> Option<String> {
        let i = self.variables_table_state.selected()?;
        self.variables().get(i).map(|(_, v)| v.clone())
    }

    fn variables(&self) -> Vec<(String, String)> {
        self.get_active_collection()
            .map(|c| c.variables.clone())
            .unwrap_or_default()
    }
}
