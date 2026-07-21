mod app;
mod event;
mod http;
mod request;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

// A type alias so we don't repeat `CrosstermBackend<Stdout>` in every signature.
type Term = Terminal<CrosstermBackend<Stdout>>;

#[tokio::main]
async fn main() -> Result<()> {
    // --- Panic hook ---
    // If the app panics we must restore the terminal *before* Rust prints the
    // panic message. Without this the terminal is left in raw mode with the
    // alternate screen still active, making it very hard to read the error.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // --- Raw mode ---
    // In raw mode the terminal stops processing keystrokes before your program
    // sees them. That means Ctrl-C no longer sends SIGINT, Enter no longer
    // adds a newline, etc. — we handle every key ourselves.
    enable_raw_mode()?;

    // --- Alternate screen ---
    // This switches to a separate terminal buffer so we have a clean canvas.
    // When we leave it later the user's shell output is restored exactly as it was.
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // Run the app. We capture the result so we can always restore the terminal,
    // even if `run` returns an error.
    let result = run(&mut terminal, App::new());

    // --- Teardown (always runs) ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?; // cursor is hidden by default in many terminals

    result
}

fn run(terminal: &mut Term, mut app: App) -> Result<()> {
    loop {
        // `draw` calls our render function with a fresh `Frame`, then compares
        // the result against the previous frame and only redraws the cells that
        // changed (diffing). This is why ratatui can be fast even at high rates.
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Block here until the next keyboard event arrives.
        // In issue #5 we'll also poll the HTTP response channel here.
        let event = event::next()?;
        app.update(event);

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
