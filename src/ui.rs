use crate::app::{App, CollectionState, EditContext, Focus, InputMode, RequestTab};
use crate::request::{AuthKind, HttpMethod};
use crate::toolbar::toolbar_hints;
use ratatui::layout::Margin;
use ratatui::style::Stylize;
use ratatui::widgets::{Padding, Scrollbar, ScrollbarOrientation};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, ScrollbarState, Tabs},
};

#[derive(Default, Clone)]
pub struct UiDimensions {
    pub collections_area: Rect,
    pub request_builder_area: Rect,
    pub response_area: Rect,
    pub notifier_area: Rect,
    pub toolbar_area: Rect,
}

pub fn render(frame: &mut Frame, app: &App) -> UiDimensions {
    // --- Top-level split ---
    // `Length(1)` takes exactly 1 row. `Min(0)` takes all remaining rows.
    let [main_area, notifier_area, toolbar] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(if app.notifier.ready() { 1 } else { 0 }),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // --- Horizontal split ---
    let [sidebar_area, right_area] =
        Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
            .areas(main_area);
    // Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(main_area);

    // --- Top and bottm of right panel ---
    let [builder_area, response_area] =
        Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)])
            .areas(right_area);
    // Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]).areas(right_area);

    let mut response_scrollbar = app.response_scrollbar.clone();

    render_collections(frame, app, sidebar_area);
    // render_debug_info(frame, app, sidebar_area);
    render_variables(frame, app, sidebar_area);

    render_request_builder(frame, app, builder_area);

    render_response(frame, app, response_area);
    // render_debug_info(frame, app, response_area);
    render_vertical_scrollbar(frame, response_area, &mut response_scrollbar);

    render_notifier(frame, app, notifier_area);
    render_toolbar(frame, app, toolbar);
    render_error(frame, app);

    UiDimensions {
        collections_area: sidebar_area,
        request_builder_area: builder_area,
        response_area,
        notifier_area,
        toolbar_area: toolbar,
    }
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

    if app.input_mode == InputMode::Insert && app.active_tab == RequestTab::Url {
        let spans = url_str
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == app.active_input.cursor {
                    Span::styled(
                        c.to_string(),
                        Style::default().bg(Color::Yellow).fg(Color::Black),
                    )
                } else {
                    Span::raw(c.to_string())
                }
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(Line::from(spans))
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::NONE)),
            url_area,
        );
        return;
    }

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
    match app.auth_kind {
        // AuthKind::Bearer => auth_draft.bearer_token,
        AuthKind::Bearer => {
            let rows = vec![ratatui::widgets::Row::new(vec![
                "Token",
                auth_draft.bearer_token.as_str(),
            ])];
            let table =
                ratatui::widgets::Table::new(rows, [Constraint::Length(20), Constraint::Min(0)])
                    .header(ratatui::widgets::Row::new(vec![
                        Span::styled("Key", Style::default().bold()),
                        Span::styled("Value", Style::default().bold()),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title(auth_tab_menu(app)))
                    .widths(&[Constraint::Length(20), Constraint::Min(0)])
                    .row_highlight_style(Style::new().on_black().bold())
                    .column_highlight_style(Color::Gray)
                    .cell_highlight_style(Style::new().reversed().yellow())
                    .highlight_symbol("> ");
            frame.render_widget(table, area);
        }
        AuthKind::Basic => {
            let rows = vec![
                ratatui::widgets::Row::new(vec!["Username:", auth_draft.basic_username.as_str()]),
                ratatui::widgets::Row::new(vec!["Password:", auth_draft.basic_password.as_str()]),
            ];
            let table =
                ratatui::widgets::Table::new(rows, [Constraint::Length(20), Constraint::Min(0)])
                    .block(Block::default().borders(Borders::ALL).title(auth_tab_menu(app)))
                    .widths(&[Constraint::Length(20), Constraint::Min(0)])
                    .row_highlight_style(Style::new().on_black().bold())
                    .column_highlight_style(Color::Gray)
                    .cell_highlight_style(Style::new().reversed().yellow())
                    .highlight_symbol("> ");
            frame.render_stateful_widget(table, area, &mut app.auth_basic_table_state.clone());
        }
        AuthKind::ApiKey => {
            let rows = vec![
                ratatui::widgets::Row::new(vec!["Header", auth_draft.api_key_header.as_str()]),
                ratatui::widgets::Row::new(vec!["Value", auth_draft.api_key_value.as_str()]),
            ];
            let table =
                ratatui::widgets::Table::new(rows, [Constraint::Length(20), Constraint::Min(0)])
                    .header(ratatui::widgets::Row::new(vec![
                        Span::styled("Key", Style::default().bold()),
                        Span::styled("Value", Style::default().bold()),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title(auth_tab_menu(app)))
                    .widths(&[Constraint::Length(20), Constraint::Min(0)])
                    .row_highlight_style(Style::new().on_black().bold())
                    .column_highlight_style(Color::Gray)
                    .cell_highlight_style(Style::new().reversed().yellow())
                    .highlight_symbol("> ");
            frame.render_widget(table, area);
        }
        AuthKind::None => {
            let rows = vec![ratatui::widgets::Row::new(vec!["", "(no authentication)"])];
            let table =
                ratatui::widgets::Table::new(rows, [Constraint::Length(20), Constraint::Min(0)])
                    .header(ratatui::widgets::Row::new(vec![
                        Span::styled("Key", Style::default().bold()),
                        Span::styled("Value", Style::default().bold()),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title(auth_tab_menu(app)))
                    .widths(&[Constraint::Length(20), Constraint::Min(0)])
                    .row_highlight_style(Style::new().on_black().bold())
                    .column_highlight_style(Color::Gray)
                    .cell_highlight_style(Style::new().reversed().yellow())
                    .highlight_symbol("> ");
            frame.render_widget(table, area);
        }
    };
    // let title = match app.auth_kind {
    //     AuthKind::Bearer => "Bearer Token",
    //     AuthKind::Basic => "Basic Auth",
    //     AuthKind::ApiKey => "API Key",
    //     AuthKind::None => "No Auth",
    // };

    // frame.render_stateful_widget(
    //     table
    //     .header(ratatui::widgets::Row::new(vec![
    //         Span::styled("Key", Style::default().bold()),
    //         Span::styled("Value", Style::default().bold()),
    //     ]))
    //     .block(Block::default())
    //     .widths(&[Constraint::Length(20), Constraint::Min(0)])
    //     .row_highlight_style(Style::new().on_black().bold())
    //     .column_highlight_style(Color::Gray)
    //     .cell_highlight_style(Style::new().reversed().yellow())
    //     .highlight_symbol("> "),
    //     area,
    //     &mut headers_state,
    // );

    // frame.render_widget(
    //     Paragraph::new(auth_text)
    //         .style(Style::default().fg(Color::White))
    //         .block(Block::default().borders(Borders::ALL).title(title)),
    //     area,
    // );
}

fn auth_tab_menu(app: &App) -> Vec<Span> {
    let mut menu = Vec::new();
    let auth_kinds = vec![
        AuthKind::Bearer,
        AuthKind::Basic,
        AuthKind::ApiKey,
        AuthKind::None,
    ];

    for kind in auth_kinds {
        let style = if app.auth_kind == kind {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let label = match kind {
            AuthKind::Bearer => "Bearer Token",
            AuthKind::Basic => "Basic Auth",
            AuthKind::ApiKey => "API Key",
            AuthKind::None => "No Auth",
        };
        menu.push(Span::styled(label, style));
    }

    menu
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
            .scroll((app.response_scroll, 0))
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

fn render_variables(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Variables;
    let block = focused_block("Variables", focused);
    let widths = [Constraint::Length(20), Constraint::Min(0)];
    let variables_area = Rect {
        x: area.x,
        y: area.y + area.height - 20,
        width: area.width,
        height: 20,
    };
    let active_collection = app.get_active_collection();
    let blank_variables = match active_collection {
        Some(col) => {
            if col.variables.is_empty() {
                "(no variables defined)"
            } else {
                ""
            }
        }
        None => "(no collection selected)",
    };

    if blank_variables != "" {
        frame.render_widget(
            Paragraph::new(blank_variables)
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            variables_area,
        );
        return;
    }

    let mut row_variables = Vec::new();
    for (i, v) in active_collection.unwrap().variables.iter().enumerate() {
        let v = v.clone();
        let key: Line = if is_editing_variable_field(&app, EditContext::VariableKey, i) {
            // Span::styled(&app.active_input.value, Style::default().fg(Color::Yellow))
            highlighted_variable_field(i, &app.active_input.value)
        } else {
            // Span::styled(v.0, Style::default().fg(Color::Cyan))
            Line::from(Span::styled(v.0, Style::default().fg(Color::Cyan)))
        };
        let value = if is_editing_variable_field(&app, EditContext::VariableValue, i) {
            // Span::styled(&app.active_input.value, Style::default().fg(Color::Yellow))
            highlighted_variable_field(i, &app.active_input.value)
        } else {
            // Span::styled(v.1, Style::default().fg(Color::White))
            Line::from(Span::styled(v.1, Style::default().fg(Color::White)))
        };

        let row = ratatui::widgets::Row::new(vec![key, value]);
        row_variables.push(row);
    }
    // let variables = active_collection.unwrap().variables.iter().map(|(k, v)| {
    //     let key = Span::styled(k, Style::default().fg(Color::Cyan));
    //     let value = Span::styled(v, Style::default().fg(Color::White));
    //     ratatui::widgets::Row::new(vec![key, value])
    // });

    frame.render_stateful_widget(
        ratatui::widgets::Table::new(row_variables, widths)
            .header(ratatui::widgets::Row::new(vec![
                Span::styled("Key", Style::default().bold()),
                Span::styled("Value", Style::default().bold()),
            ]))
            .block(block)
            .widths(&[Constraint::Length(20), Constraint::Min(0)])
            .row_highlight_style(Style::new().on_black().bold())
            .column_highlight_style(Color::Gray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("> "),
        variables_area,
        &mut app.variables_table_state.clone(),
    );
}

fn is_editing_variable_field(app: &App, edit_context: EditContext, row: usize) -> bool {
    if app.input_mode == InputMode::Normal || app.focus != Focus::Variables {
        return false;
    }

    app.variables_table_state.selected() == Some(row) && app.edit_context == edit_context
}

fn highlighted_variable_field(current_index: usize, value: &String) -> Line {
    let text = value.chars().enumerate().map(|(i, c)| {
        if i == current_index {
            Span::styled(
                c.to_string(),
                Style::default().bg(Color::Yellow).fg(Color::Black),
            )
        } else {
            Span::raw(c.to_string())
        }
    });
    Line::from(text.collect::<Vec<_>>()).clone()
}

// ---------------------------------------------------------------------------
// Notifier
// ---------------------------------------------------------------------------
fn render_notifier(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(
        Paragraph::new(app.notifier.display()).style(app.notifier.style()),
        area,
    );
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------
fn render_toolbar(frame: &mut Frame, app: &App, area: Rect) {
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
        "focus: {:?} | tab: {:?} | mode: {:?}\nresponse: {:?}\nselected_header_row: {:?} | editing_header_field: {:?} | headers_table_selected: {:?}\ncollection_ui_state: {:?}\nauth_kind: {:?}\nauth_draft: {:?}\nediting_auth_field: {:?}\nactive_input.value: {:?}\nactive_input.cursor: {:?}\nvariables_table_selected: {:?}\nedit_context: {:?}",
        app.focus,
        app.active_tab,
        app.input_mode,
        app.response,
        app.selected_header_row,
        app.editing_header_field.as_ref(),
        app.headers_table_state.selected(),
        app.collection_ui_state,
        app.auth_kind,
        app.auth_draft,
        app.editing_auth_field.as_ref(),
        app.active_input.value,
        app.active_input.cursor,
        app.variables_table_state.selected(),
        app.edit_context
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
