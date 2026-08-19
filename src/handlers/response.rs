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

    pub(crate) fn handle_response_mouse(&mut self, mouse_event: crossterm::event::MouseEvent) {
        if !self.is_intersecting(mouse_event) {
            return;
        }

        match mouse_event.kind {
            crossterm::event::MouseEventKind::ScrollDown => self.scroll_down(),
            crossterm::event::MouseEventKind::ScrollUp => self.scroll_up(),
            _ => {}
        }
    }

    fn is_intersecting(&self, mouse_event: crossterm::event::MouseEvent) -> bool {
        let (x, y) = (mouse_event.column, mouse_event.row);
        let response_area = self.ui_dimensions.response_area;
        x >= response_area.x
            && x < response_area.x + response_area.width
            && y >= response_area.y
            && y < response_area.y + response_area.height
    }

    fn copy_response(&mut self) {
        if let Some(response) = &self.response {
            self.clipboard.set_text(response.body.clone()).unwrap();
            self.notifier.notify(
                crate::notifier::NotifyType::Info,
                "Response copied to clipboard".to_string(),
            );
        }
    }

    fn scroll_down(&mut self) {
        self.response_scroll = self.response_scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.response_scroll = self.response_scroll.saturating_sub(1);
    }
}
