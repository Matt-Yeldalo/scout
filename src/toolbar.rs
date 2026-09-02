use crate::app::{App, CollectionState, Focus, InputMode, RequestTab};
static GAP: &str = "  ";
pub fn toolbar_hints(app: &App) -> String {
    let parts = match app.input_mode {
        InputMode::Normal => {
            match app.focus {
                Focus::Collections => match app.collection_ui_state {
                    CollectionState::CollectionList => {
                        vec![
                            "[tab] cycle focus",
                            "[k/j] up/down",
                            "[i] edit collection",
                            "[s] save",
                            "[q] quit",
                            "[?] toggle help",
                        ]
                    }
                    CollectionState::RequestList(_) => {
                        // arrow up: ↑, arrow down: ↓
                        // arrow left: ←, arrow right: →
                        // "  [tab] cycle focus   [h/j] up/down [←] collections   [i] edit request     [s] save   [q] quit".to_string()
                        vec![
                            "[tab] cycle focus",
                            "[k/j] up/down",
                            "[←] collections",
                            "[i] edit request",
                            "[s] save",
                            "[q] quit",
                            "[?] toggle help",
                        ]
                    }
                },
                Focus::RequestBuilder => match app.active_tab {
                    RequestTab::Auth => {
                        vec![
                            "[tab] cycle focus",
                            "[h/l] switch tab",
                            "[←/→] switch auth type",
                            "[↓/↑] switch auth field",
                            "[i] insert",
                            "[m] method",
                            "[s] send",
                            "[q] quit",
                            "[?] toggle help",
                        ]
                    }
                    _ => {
                        vec![
                            "[tab] cycle focus",
                            "[h/l] switch tab",
                            "[i] insert",
                            "[m] method",
                            "[s] send",
                            "[q] quit",
                            "[?] toggle help",
                        ]
                    }
                },
                Focus::Response => {
                    vec![
                        "[tab] cycle focus",
                        "[k/j] up/down",
                        "[y] copy response",
                        "[s] send",
                        "[q] quit",
                        "[?] toggle help",
                    ]
                }
                Focus::Variables => {
                    // let selected_field = app.variables_table_state.selected();
                    // let selected_field = match selected_field {
                    //     Some(0) => app
                    //     None => String::new(),
                    // };

                    vec![
                        "[tab] cycle focus",
                        "[k/j] up/down",
                        "[i] edit variable",
                        "[a] add row",
                        "[s] save",
                        "[q] quit",
                        "[?] toggle help",
                    ]
                }
            }
        }
        InputMode::Insert => vec!["[esc] back to normal"],
    };
    build_toolbar(&parts)
}

fn build_toolbar(parts: &Vec<&str>) -> String {
    let mut toolbar = String::new();
    for part in parts {
        toolbar.push_str(GAP);
        toolbar.push_str(&part);
    }
    toolbar
}
