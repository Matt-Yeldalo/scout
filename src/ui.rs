use crate::app::{App, CollectionState, Focus, InputMode, RequestTab};
use crate::request::{AuthKind, HttpMethod};
use crate::toolbar::toolbar_hints;
use ratatui::layout::Margin;
use ratatui::style::Stylize;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, ScrollbarState, Tabs},
};

/// Entry point called by the main loop on every frame.
/// This function must never mutate `app` — it is a pure read.
pub fn render(frame: &mut Frame, app: &App) {
    // --- Top-level split ---
    // `Length(1)` takes exactly 1 row. `Min(0)` takes all remaining rows.
    let [main_area, status_bar] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

    // --- Horizontal split ---
    let [sidebar_area, right_area] =
        // Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
        //     .areas(main_area);
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(main_area);

    // --- Top and bottm of right panel ---
    let [builder_area, response_area] =
        // Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)])
        //     .areas(right_area);
        Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]).areas(right_area);

    let mut response_scrollbar = app.response_scrollbar.clone();

    render_collections(frame, app, sidebar_area);
    render_debug_info(frame, app, sidebar_area);

    render_request_builder(frame, app, builder_area);

    render_response(frame, app, response_area);
    render_vertical_scrollbar(frame, response_area, &mut response_scrollbar);

    render_status_bar(frame, app, status_bar);
    render_error(frame, app);
}

fn render_error(frame: &mut Frame, app: &App) {
    if app.error_message.is_none() {
        return;
    }

    let message = app.error_message.as_ref().unwrap();

    if message.is_empty() {
        return;
    }

    let area = Frame::area(&frame);
    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    frame.render_widget(
        Paragraph::new(message.as_str())
            .style(Style::default().fg(Color::Red))
            .block(block),
        area,
    );
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn render_vertical_scrollbar(frame: &mut Frame, area: Rect, vertical: &mut ScrollbarState) {
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    frame.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        vertical,
    );
}

/// A bordered panel whose border turns yellow when it has focus.
/// The visual highlight tells you at a glance which panel will receive input.
fn focused_block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
}

// ---------------------------------------------------------------------------
// Collections sidebar
// ---------------------------------------------------------------------------

enum ListItem {
    Collection {
        idx: usize,
        name: String,
    },
    Request {
        col_idx: usize,
        req_idx: usize,
        name: String,
    },
}

fn render_collections(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Collections;

    match app.collection_ui_state {
        CollectionState::RequestList(_) => {
            render_collection_request_list(frame, app, area, focused)
        }
        CollectionState::CollectionList => render_collection_list(frame, app, area, focused),
    }
}

fn render_collection_list(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let items: Vec<ratatui::widgets::ListItem> = app
        .collections
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let prefix = if app.expanded_collections.contains(&i) {
                "▼"
            } else {
                "▶"
            };
            ratatui::widgets::ListItem::new(format!(" {} {}", prefix, col.name))
        })
        .collect();

    frame.render_stateful_widget(
        ratatui::widgets::List::new(items)
            .block(focused_block("Collections", focused))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> "),
        area,
        &mut app.collection_list_state.clone(),
    );
}

fn render_collection_request_list(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let selected_collection = app
        .collection_list_state
        .selected()
        .and_then(|i| app.collections.get(i));

    if let Some(collection) = selected_collection {
        let items: Vec<ratatui::widgets::ListItem> = collection
            .requests
            .iter()
            // .map(|req| ratatui::widgets::ListItem::new(format!("   {} {}", "↳", req.name)))
            .map(|req| ratatui::widgets::ListItem::new(format!("   {}", req)))
            .collect();
        let title = format!("Requests ({})", collection.name);

        frame.render_stateful_widget(
            ratatui::widgets::List::new(items)
                .block(focused_block(title.as_str(), focused))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> "),
            area,
            &mut app.request_list_state.clone(),
        );
    } else {
        frame.render_widget(
            Paragraph::new("(no collection selected)")
                .style(Style::default().fg(Color::DarkGray))
                .block(focused_block("Requests", focused)),
            area,
        );
    }
}

// ---------------------------------------------------------------------------
// Request builder
// ---------------------------------------------------------------------------

fn render_request_builder(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::RequestBuilder;
    let outer_block = focused_block("Request", focused);

    // `inner()` returns the Rect that sits *inside* the block's border.
    // We draw the block first, then render tab bar + content inside that Rect.
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Divide the inner area: 1 row for tabs, 1 row for a divider, rest for content.
    let [tabs_row, divider_row, content_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    // `Tabs` renders a row of labels with the selected one highlighted.
    // We map the enum variant to an index because Tabs works with integers.
    let selected = match app.active_tab {
        RequestTab::Url => 0,
        RequestTab::Headers => 1,
        RequestTab::Body => 2,
        RequestTab::Auth => 3,
    };

    frame.render_widget(
        Tabs::new(["URL", "Headers", "Body", "Auth"])
            .select(selected)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider("│"),
        tabs_row,
    );

    // A Block with only a BOTTOM border acts as a thin horizontal separator.
    frame.render_widget(Block::default().borders(Borders::BOTTOM), divider_row);

    // Render whichever tab is active. The other tabs are implemented later.
    match app.active_tab {
        RequestTab::Url => render_url_tab(frame, app, content_area),
        RequestTab::Headers => render_headers_tab(frame, app, content_area),
        RequestTab::Body => render_body_tab(frame, app, content_area),
        RequestTab::Auth => render_auth_tab(frame, app, content_area),
        // RequestTab::Auth => render_placeholder(frame, "Auth — issue #8", content_area),
    }
}

fn render_url_tab(frame: &mut Frame, app: &App, area: Rect) {
    // Method badge on the left (fixed width) with a right border acting as
    // a visual separator, then the URL field fills the rest of the row.
    let [method_area, url_area] =
        Layout::horizontal([Constraint::Length(7), Constraint::Min(0)]).areas(area);

    let method_label = match app.active_request.method {
        HttpMethod::Get => " GET",
        HttpMethod::Post => "POST",
    };

    frame.render_widget(
        Paragraph::new(method_label)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::RIGHT)),
        method_area,
    );

    // Show a grey hint when the URL is empty; white text when it has a value.
    let url_str = app.active_request.url.as_str();
    let (url_display, url_style) = if url_str.is_empty() {
        ("https://...", Style::default().fg(Color::DarkGray))
    } else {
        (url_str, Style::default().fg(Color::White))
    };

    frame.render_widget(Paragraph::new(url_display).style(url_style), url_area);
}

fn render_headers_tab(frame: &mut Frame, app: &App, area: Rect) {
    let headers: Vec<(String, String)> = app.active_request.headers.iter().cloned().collect();
    let widths = [Constraint::Length(20), Constraint::Min(0)];
    let mut headers_state = app.headers_table_state.clone();

    if headers.is_empty() {
        render_placeholder(frame, "(no headers)", area);
        return;
    }

    frame.render_stateful_widget(
        ratatui::widgets::Table::new(
            headers.iter().map(|(k, v)| {
                // let key = Span::styled(k, Style::default().fg(Color::Cyan));
                // let value = Span::styled(v, Style::default().fg(Color::White));
                // ratatui::widgets::Row::new(vec![key, value])
                ratatui::widgets::Row::new(vec![k.as_str(), v])
            }),
            widths,
        )
        .header(ratatui::widgets::Row::new(vec![
            Span::styled("Key", Style::default().bold()),
            Span::styled("Value", Style::default().bold()),
        ]))
        .block(Block::default())
        .widths(&[Constraint::Length(20), Constraint::Min(0)])
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray)
        .cell_highlight_style(Style::new().reversed().yellow())
        .highlight_symbol("> "),
        area,
        &mut headers_state,
    );
}

fn render_auth_tab(frame: &mut Frame, app: &App, area: Rect) {
    let auth_draft = app.auth_draft.clone();
    let auth_text = match app.auth_kind {
        AuthKind::Bearer => auth_draft.bearer_token,
        AuthKind::Basic => format!(
            "Username: {}\nPassword: {}",
            auth_draft.basic_username, auth_draft.basic_password
        ),
        AuthKind::ApiKey => format!(
            "Header: {}\nValue: {}",
            auth_draft.api_key_header, auth_draft.api_key_value
        ),
        AuthKind::None => "(no authentication)".to_string(),
    };

    frame.render_widget(
        Paragraph::new(auth_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Authentication"),
            ),
        area,
    );
}

fn render_body_tab(frame: &mut Frame, app: &App, area: Rect) {
    if app.input_mode == InputMode::Insert {
        frame.render_widget(&app.body_editor, area);
        return;
    }

    let body = app.active_request.body.as_deref().unwrap_or_default();
    if body.is_empty() {
        render_placeholder(frame, "(no body — press i to edit)", area);
    } else {
        frame.render_widget(
            Paragraph::new(body).style(Style::default().fg(Color::White)),
            area,
        );
    }
}

fn render_placeholder(frame: &mut Frame, label: &str, area: Rect) {
    frame.render_widget(
        Paragraph::new(label).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

// ---------------------------------------------------------------------------
// Response pane
// ---------------------------------------------------------------------------

fn render_response(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Response;
    let block = focused_block("Response", focused);
    // let scrollbar = ratatui::widgets::Scrollbar::default()
    //                     .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
    //                     .style(Style::default().fg(Color::DarkGray));

    // Three possible states: loading, no response yet, or a response arrived.
    if app.is_loading {
        frame.render_widget(
            Paragraph::new("Sending…")
                .style(Style::default().fg(Color::Yellow))
                .block(block),
            area,
        );
        return;
    }

    let Some(resp) = &app.response else {
        frame.render_widget(
            Paragraph::new("No response yet.\n\nPress [s] to send the request.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    };

    // Colour the status line green/yellow/red based on the HTTP status code.
    let status_color = match resp.status {
        200..=299 => Color::Green,
        300..=399 => Color::Yellow,
        _ => Color::Red,
    };

    let text = build_response_text(app, resp);

    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(status_color))
            .block(block),
        area,
    );
}

fn build_response_text(app: &App, resp: &crate::request::Response) -> String {
    let body = if resp.body.is_empty() {
        "(empty)".to_string()
    } else {
        resp.body.clone()
    };

    if resp
        .headers
        .get("content-type")
        .map(|v| v.contains("application/json"))
        == Some(true)
    {
        // Pretty-print JSON if possible.
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body) {
            return format!(
                "{} {}   {}ms\n\n{}",
                resp.status,
                resp.status_text,
                resp.duration_ms,
                serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| body)
            );
        }
    }

    if app.debug {
        let headers_text = resp
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        return format!(
            "{} {}   {}ms\n\n{}\n\nHeaders:\n{}",
            resp.status, resp.status_text, resp.duration_ms, body, headers_text
        );
    }

    format!(
        "{} {}   {}ms\n\n{}",
        resp.status, resp.status_text, resp.duration_ms, body
    )
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    // The mode badge uses a coloured background to make it stand out.
    // Blue = Normal (calm, navigating), Green = Insert (active, editing).
    let (mode_label, mode_style) = match app.input_mode {
        InputMode::Normal => (
            " NORMAL ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Insert => (
            " INSERT ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    };

    // Show context-sensitive hints so the user always knows what keys do something.
    // let hints = match app.input_mode {
    //     InputMode::Normal => {
    //         "  [tab] cycle focus   [h/l] switch tab   [i] insert   [m] method   [s] send   [q] quit"
    //     }
    //     InputMode::Insert => "  [esc] back to normal",
    // };

    // how do I use this method that live in src/toolbar.rs?
    // you can call the `toolbar_hints` function from the `toolbar` module. First, make sure to import it at the top of your file:
    // use crate::toolbar::toolbar_hints;
    // Then, you can call it like this:
    // its saying could not find in crate root
    // I added mod toolbar but now it wants me to to move the toolbar.rs file into a folder called toolbar and rename it mod.rs. I don't want to do that. I just want to call the function in toolbar.rs from ui.rs. How do I do that?
    // You can call the function directly by using the module path. Since `toolbar.rs`
    // is in the same directory as `ui.rs`, you can call the function like this:
    //
    let hints = toolbar_hints(app);

    // `Line::from` builds a single terminal line from styled `Span`s.
    // Spans within a Line share the same row but can have different styles.
    let line = Line::from(vec![Span::styled(mode_label, mode_style), Span::raw(hints)]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_debug_info(frame: &mut Frame, app: &App, area: Rect) {
    if !app.debug {
        return;
    }

    let debug_text = format!(
        // "focus: {:?}\ttab: {:?}\tmode: {:?}\n
        //  response: {:?}\n
        //  selected_header_row: {:?}\tediting_header_field: {:?}\theaders_table_selected: {:?}\n
        //  collection_ui_state: {:?}",
        "focus: {:?}\ttab: {:?}\tmode: {:?}\nresponse: {:?}\nselected_header_row: {:?}\tediting_header_field: {:?}\theaders_table_selected: {:?}\ncollection_ui_state: {:?}",
        app.focus,
        app.active_tab,
        app.input_mode,
        app.response,
        app.selected_header_row,
        app.editing_header_field.as_ref(),
        app.headers_table_state.selected(),
        app.collection_ui_state,
    );

    let debug_area = Rect {
        x: area.x,
        y: area.y + area.height - 20,
        width: area.width,
        height: 20,
    };

    frame.render_widget(
        Paragraph::new(debug_text)
            .style(Style::default().fg(Color::LightYellow))
            .block(Block::default().title("Debug").borders(Borders::ALL)),
        debug_area,
    );
}
