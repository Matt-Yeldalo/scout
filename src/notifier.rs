use ratatui::style::Style;

#[derive(Default)]
pub enum NotifyType {
    #[default]
    None,
    Info,
    Warning,
    Error,
}

#[derive(Default)]
pub struct Notifier {
    notify_type: NotifyType,
    message: String,
}

impl Notifier {
    pub fn notify(&mut self, notify_type: NotifyType, message: String) {
        self.notify_type = notify_type;
        self.message = message;
    }

    pub fn display(&self) -> String {
        match self.notify_type {
            NotifyType::None => "".to_string(),
            NotifyType::Info => format!("[INFO]: {}", self.message),
            NotifyType::Warning => format!("[WARNING]: {}", self.message),
            NotifyType::Error => format!("[ERROR]: {}", self.message),
        }
    }

    pub fn ready(&self) -> bool {
        !matches!(self.notify_type, NotifyType::None)
    }

    pub fn style(&self) -> Style {
        match self.notify_type {
            NotifyType::None => Style::default(),
            NotifyType::Info => Style::default().fg(ratatui::style::Color::Blue).bg(ratatui::style::Color::Black),
            NotifyType::Warning => Style::default().fg(ratatui::style::Color::Yellow).bg(ratatui::style::Color::Black),
            NotifyType::Error => Style::default().fg(ratatui::style::Color::Red).bg(ratatui::style::Color::Black),
        }
    }
}
