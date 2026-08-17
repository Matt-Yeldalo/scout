use crate::app::{App, CollectionState, Focus, InputMode};
static GAP: &str = "  ";
pub fn toolbar_hints(app: &App) -> String {
    let parts = match app.input_mode {
        InputMode::Normal => {
            match app.focus {
                Focus::Collections => match app.collection_ui_state {
                    CollectionState::CollectionList => {
                        // "  [tab] cycle focus   [h/j] up/down   [i] edit collection     [s] save   [q] quit".to_string()
                        vec![
                            "[tab] cycle focus",
                            "[h/j] up/down",
                            "[i] edit collection",
                            "[s] save",
                            "[q] quit",
                        ]
                    }
                    CollectionState::RequestList(_) => {
                        // arrow up: ↑, arrow down: ↓
                        // arrow left: ←, arrow right: →
                        // "  [tab] cycle focus   [h/j] up/down [←] collections   [i] edit request     [s] save   [q] quit".to_string()
                        vec![
                            "[tab] cycle focus",
                            "[h/j] up/down",
                            "[←] collections",
                            "[i] edit request",
                            "[s] save",
                            "[q] quit",
                        ]
                    }
                },
                Focus::RequestBuilder => {
                    // "  [tab] cycle focus   [h/l] switch tab   [i] insert   [m] method   [s] send   [q] quit".to_string()
                    vec![
                        "[tab] cycle focus",
                        "[h/l] switch tab",
                        "[i] insert",
                        "[m] method",
                        "[s] send",
                        "[q] quit",
                    ]
                }
                Focus::Response => {
                    // "  [tab] cycle focus   [h/l] switch tab   [s] send   [q] quit".to_string()
                    vec![
                        "[tab] cycle focus",
                        "[h/l] switch tab",
                        "[s] send",
                        "[q] quit",
                    ]
                } // _ => {
                  //     "  [tab] cycle focus   [h/l] switch tab   [q] quit".to_string()
                  // }
            }
        }
        // InputMode::Insert => "  [esc] back to normal".to_string(),
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
