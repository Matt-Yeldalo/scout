# Scout — Design Document

## Overview

Scout is a lightweight terminal UI for quickly firing HTTP requests and inspecting
responses. The primary use case is testing third-party API integrations at work without
leaving the terminal.

**Goals**
- Fast to open and use
- Keyboard-driven, vim-inspired (Normal / Insert modes)
- Save and load named requests grouped into collections
- GET and POST support with headers, body, and auth (Bearer / Basic / API key)
- Pretty-printed response with status, headers, and body

**Non-goals**
- Not a full Postman replacement
- No scripting, environments, or test assertions
- No mouse support

---

## Architecture

Ratatui apps follow a simple render → event → update loop. There is no framework magic —
you own the loop entirely.

```
loop {
    terminal.draw(|frame| ui::render(frame, &app))?;  // pure read of state → screen
    let event = event::next(&mut response_rx)?;        // block until keyboard or HTTP response
    app.update(event);                                 // mutate state
    if app.should_quit { break; }
}
```

All mutable state lives in one `App` struct. The render function never mutates anything —
it is a pure mapping from `&App` to terminal output.

HTTP requests run on a Tokio background task and send their result back to the event loop
through an `mpsc` channel. This keeps the render loop responsive while waiting for slow
APIs.

```
[event loop] ──tokio::spawn──→ [http::send(request)]
[event loop] ←──AppEvent::HttpResponse── [task sends result]
```

---

## Module structure

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Terminal setup/teardown, run the event loop |
| `app.rs` | `App` struct — all state + `update(AppEvent)` dispatcher |
| `ui.rs` | Pure render functions — read `&App`, draw to `Frame` |
| `request.rs` | `Request`, `Collection`, `Auth`, `Response` types + serde |
| `http.rs` | Fire an HTTP request; return a `Response` |
| `event.rs` | `AppEvent` enum, poll keyboard + response channel |

---

## UI layout

```
┌─ Collections ──┬─ [URL] [Headers] [Body] [Auth] ─────────┐
│                │  POST  https://api.example.com/login     │
│  ▶ Auth API    │                                          │
│      login     ├──────────────────────────────────────────┤
│      get-user  │  200 OK  ·  142ms                        │
│                │  content-type: application/json          │
│  ▶ Billing API │                                          │
│      charges   │  {                                       │
│                │    "token": "abc123",                    │
│                │    "expires_in": 3600                    │
│                │  }                                       │
└────────────────┴──────────────────────────────────────────┘
 [n]ew  [s]end  [S]ave  [d]elete  [q]uit          NORMAL
```

Three focusable regions (`Tab` to cycle):
1. **Collections sidebar** — navigate saved requests with `j`/`k`
2. **Request builder** — tabs: URL · Headers · Body · Auth
3. **Response pane** — status line, response headers, pretty-printed body

---

## Input modes

| Mode | Enter | Exit | Purpose |
|------|-------|------|---------|
| Normal | `Esc` | — | Navigate, trigger actions with single keys |
| Insert | `i` or `Enter` on a field | `Esc` | Type into text fields |

In Normal mode all keypresses are action bindings. In Insert mode all keypresses go to
the focused text field. This mirrors Vim and is the most common pattern in ratatui apps.

### Normal mode key bindings

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus between panels |
| `j` / `k` | Move up/down in lists |
| `h` / `l` | Switch request builder tabs |
| `i` | Enter Insert mode |
| `s` | Send request |
| `n` | New request |
| `S` | Save current request to collection |
| `d` | Delete selected item |
| `q` | Quit |

---

## Data types

```
Collection
  name: String
  requests: Vec<Request>

Request
  name: String
  method: HttpMethod  (Get | Post)
  url: String
  headers: HashMap<String, String>
  body: Option<String>
  auth: Auth

Auth
  None
  Bearer(token: String)
  Basic { username, password }
  ApiKey { header, value }

Response
  status: u16
  status_text: String
  headers: HashMap<String, String>
  body: String
  duration_ms: u128
```

---

## Persistence

Collections are stored at `~/.config/scout/collections.json`. The format is a JSON array
of collections, each containing a named list of requests. Load on startup; save whenever
the user presses `S`. `serde` + `serde_json` handle serialisation automatically via
`#[derive(Serialize, Deserialize)]`.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI rendering (widgets, layout, frame) |
| `crossterm` | Terminal backend, raw mode, keyboard events |
| `tokio` | Async runtime for non-blocking HTTP |
| `reqwest` | HTTP client |
| `serde` + `serde_json` | Serialise / deserialise collections |
| `anyhow` | Ergonomic error handling throughout |

---

## Issues

Work through these in order. Each is self-contained.

### #1 — Terminal setup and teardown
Set up and restore the terminal correctly using `crossterm`. Enable raw mode, switch to
the alternate screen on startup, and restore both on exit — including on panic (use a
panic hook). Confirm you can enter and exit without corrupting the terminal.  
_Touches: `main.rs`_

### #2 — Static layout
Draw the three-panel layout with placeholder content using ratatui `Layout`,
`Block`, and `Paragraph` widgets. No interaction yet — just confirm the proportions
look right in your terminal.  
_Touches: `ui.rs`_

### #3 — Input mode and focus switching
Implement `Tab` to cycle focus between panels, `i` to enter Insert mode, `Esc` to
return to Normal. Highlight the focused panel's border. Show the current mode in the
status bar.  
_Touches: `app.rs`, `event.rs`, `ui.rs`_

### #4 — URL and method editing
Make the URL field editable in Insert mode (basic left/right/backspace/delete cursor).
Toggle method between GET and POST with `m` in Normal mode.  
_Touches: `app.rs`, `ui.rs`, `event.rs`_

### #5 — Send a request and display raw response
Wire up `http::send` using reqwest inside a `tokio::spawn` task. Send results back via
`tokio::sync::mpsc`. Display status code, duration, and raw body in the response pane.
Handle errors (timeout, connection refused) gracefully.  
_Touches: `http.rs`, `app.rs`, `event.rs`, `main.rs`_

### #6 — Headers tab
Editable key/value list for request headers. Navigate rows with `j`/`k`, add a new row
with `a`, delete selected row with `d`, edit values in Insert mode.  
_Touches: `app.rs`, `ui.rs`, `event.rs`_

### #7 — Body tab
A scrollable multiline text area for the POST body. Basic editing in Insert mode.  
_Touches: `app.rs`, `ui.rs`, `event.rs`_

### #8 — Auth tab
Choose auth type with `j`/`k` (None / Bearer / Basic / API key) and fill in the
relevant fields. Apply the selected auth to outgoing requests in `http::send`.  
_Touches: `request.rs`, `http.rs`, `ui.rs`, `app.rs`_

### #9 — Collections sidebar
Display collections with expandable request lists. Navigate with `j`/`k`,
expand/collapse a collection with `Enter`, load a request into the builder with `Enter`
on a request row.  
_Touches: `app.rs`, `ui.rs`, `event.rs`_

### #10 — Save and load collections
Persist collections to `~/.config/scout/collections.json`. Load on startup; save on `S`.
Create the config directory if it doesn't exist.  
_Touches: `request.rs`, `app.rs`, `main.rs`_

### #11 — Pretty-print JSON responses
Detect `content-type: application/json` in the response and pretty-print the body with
indentation using `serde_json::to_string_pretty`.  
_Touches: `ui.rs` or `app.rs`_

### #12 — Scrollable response body
Allow scrolling through long response bodies with `j`/`k` when the response pane is
focused.  
_Touches: `ui.rs`, `app.rs`, `event.rs`_
