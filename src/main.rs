mod app;
mod event;
mod handlers;
mod http;
mod input;
mod local_store;
mod notifier;
mod request;
mod toolbar;
mod ui;
mod config;

use crate::{event::AppEvent, ui::UiDimensions};
use anyhow::Result;
use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::TableState};
use std::io::{self, Stdout};

// A type alias so we don't repeat `CrosstermBackend<Stdout>` in every signature.
type Term = Terminal<CrosstermBackend<Stdout>>;

#[tokio::main]
async fn main() -> Result<()> {
    // --- Panic hook ---
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    enable_raw_mode()?;

    // --- Alternate screen ---
    // This switches to a separate terminal buffer so we have a clean canvas.
    // When we leave it later the user's shell output is restored exactly as it was.
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<AppEvent>(100);
    let mut table_state = TableState::default();
    table_state.select_first();
    table_state.select_first_column();

    let app = App::new(sender.clone(), table_state);
    let result = run(&mut terminal, app, &mut receiver).await;

    // --- Teardown (always runs) ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?; // cursor is hidden by default in many terminals

    result
}

async fn run(
    terminal: &mut Term,
    mut app: App,
    receiver: &mut tokio::sync::mpsc::Receiver<AppEvent>,
) -> Result<()> {
    let mut event_stream = crossterm::event::EventStream::new();
    let mut last_dimenions = UiDimensions::default();
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
        terminal.draw(|frame| last_dimenions = ui::render(frame, &app))?;

        let event = event::next(receiver, &mut event_stream, &mut interval).await?;
        app.ui_dimensions = last_dimenions.clone();
        app.update(event);

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
