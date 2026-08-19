use crate::app::{App, CollectionState};
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub(crate) fn handle_response_normal(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
            KeyCode::Char('y') => self.copy_response(),
            _ => {}
        }
    }

    fn copy_response(&mut self) {
        if let Some(response) = &self.response {
            self.clipboard.set_text(response.body.clone()).unwrap();
            self.notifier.notify(crate::notifier::NotifyType::Info, "Response copied to clipboard".to_string());
        }
    }

    fn scroll_down(&mut self) {
        self.response_scroll = self.response_scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.response_scroll = self.response_scroll.saturating_sub(1);
    }
}
