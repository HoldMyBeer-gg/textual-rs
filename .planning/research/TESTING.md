# Testing Strategies for Rust TUI Applications

**Domain:** Rust TUI framework (textual-rs) test infrastructure
**Researched:** 2026-03-24
**Overall confidence:** HIGH (all major claims verified against official documentation or well-maintained source repos)

---

## 1. Ratatui TestBackend and Buffer Assertions

### What TestBackend Is

`ratatui::backend::TestBackend` is ratatui's built-in in-memory backend. It renders into a `Buffer` (a 2D grid of `Cell` values) instead of writing escape sequences to a real terminal. It is the right primitive for any unit or integration test that does not need real PTY behavior.

**Create a test terminal:**
```rust
use ratatui::{Terminal, backend::TestBackend};

let backend = TestBackend::new(80, 24);
let mut terminal = Terminal::new(backend).unwrap();
```

### Buffer Assertion API (as of ratatui 0.29+)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `assert_buffer` | `(&self, expected: &Buffer)` | Full buffer equality check with rich diff on failure |
| `assert_buffer_lines` | `(&self, lines: impl IntoIterator<Item = impl Into<Line>>)` | Compare buffer rows as plain strings — the most ergonomic option |
| `assert_scrollback` | `(&self, expected: &Buffer)` | For content scrolled off-screen |
| `assert_scrollback_empty` | `(&self)` | Assert nothing was scrolled off |
| `assert_cursor_position` | `(&mut self, position: impl Into<Position>)` | Assert cursor coordinates |

`assert_buffer_eq!` macro was deprecated in 0.26.3 in favor of `assert_buffer_lines`.

### Buffer Direct API (for custom assertions)

`ratatui::buffer::Buffer` exposes:
- `Buffer::with_lines(lines)` — construct an expected buffer from string slices, useful for constructing expected values
- `buffer.cell((x, y))` — safe `Option<&Cell>` access
- `buffer[(x, y)]` — indexed `&Cell` access (panics on out-of-bounds)
- `buffer.diff(&other)` — minimal set of changed cells; useful for custom assertion messages
- `buffer.content()` — raw `&[Cell]` slice

**Current known limitation:** Cell style/color data is captured in the buffer but `assert_buffer_lines` compares only character content. Full style assertions require comparing `Cell` objects directly via the `buffer[(x,y)]` API.

### Canonical Unit Test Pattern for a Widget

```rust
#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
    use crate::widgets::MyWidget;

    #[test]
    fn renders_correctly() {
        let widget = MyWidget::new("hello");
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Approach 1: assert all lines at once
        buf.assert_buffer_lines([
            "hello               ",
            "                    ",
            "                    ",
        ]);

        // Approach 2: spot-check individual cells
        assert_eq!(buf[(0, 0)].symbol(), "h");
    }
}
```

**Source:** [TestBackend docs](https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html), [ratatui TDD discussion #78](https://github.com/ratatui/ratatui/discussions/78), [ratatui CONTRIBUTING.md](https://github.com/ratatui/ratatui/blob/main/CONTRIBUTING.md)

---

## 2. Insta — Snapshot Testing

### What Insta Is

`insta` (by Armin Ronacher / mitsuhiko) is the de-facto standard snapshot testing crate for Rust. It captures the `Display` (or serialized) output of any value and stores it as a `.snap` file. On subsequent runs it diffs current output against the stored snapshot and fails the test on divergence. The companion CLI `cargo-insta` provides an interactive review workflow.

**Install:**
```toml
# Cargo.toml
[dev-dependencies]
insta = "1"

# Shell
cargo install cargo-insta
```

### Core Macros

| Macro | Input Type | Best For |
|-------|-----------|---------|
| `assert_snapshot!` | `Display` | Terminal string output, rendered buffers |
| `assert_debug_snapshot!` | `Debug` | Parsed AST nodes, widget state structs |
| `assert_yaml_snapshot!` | `Serialize` | Complex structs with serde |
| `assert_json_snapshot!` | `Serialize` | JSON-shaped data |

### Inline vs File Snapshots

```rust
// External file (stored in snapshots/ directory)
insta::assert_snapshot!(rendered_output);

// Inline (stored in the source file itself — great for small outputs)
insta::assert_snapshot!(rendered_output, @r###"
┌──────────────────────────────┐
│ Hello, World!                │
└──────────────────────────────┘
"###);
```

Inline snapshots are recommended for TUI widgets because the visual output is right next to the test code, making regressions obvious during code review.

### Workflow

1. First run: test "fails" but writes `.snap.new` file
2. Run `cargo insta review` — interactive accept/reject per snapshot
3. Accept: snapshot is written to `.snap` (or inlined)
4. CI: committed snapshots are reference values; divergence = test failure
5. Intentional UI change: update via `cargo insta review` and commit the diff

### Ratatui + Insta: The Recommended Integration

```rust
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};
use crate::App;

#[test]
fn snapshot_app_initial_state() {
    let app = App::default();
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    terminal
        .draw(|frame| frame.render_widget(&app, frame.area()))
        .unwrap();
    // TestBackend implements Display — renders each row as a text line
    assert_snapshot!(terminal.backend());
}
```

**Consistency rule:** Always use a fixed terminal size (e.g., 80x20). Any test with a different size will produce a different snapshot. Keep size as a named constant.

**Color limitation (MEDIUM confidence):** As of early 2026, `assert_snapshot!` via `terminal.backend()` captures character content only, not ANSI color attributes. Style regressions require direct `Cell` inspection.

**Source:** [Ratatui snapshot guide](https://ratatui.rs/recipes/testing/snapshots/), [insta.rs](https://insta.rs/), [insta GitHub](https://github.com/mitsuhiko/insta)

---

## 3. Implementing a "Pilot" Concept in Rust

### What Textual's Pilot Does (Reference Model)

Python's Textual framework exposes a `Pilot` object from `app.run_test()`. It lets tests simulate:
- `pilot.press("ctrl+c")` — key events
- `pilot.click("#button", offset=(5, 0))` — mouse click at CSS-selected widget
- `pilot.pause()` — drain the message queue before asserting

The key design principle: the app runs in headless mode (no real terminal), and the pilot drives it through the same event channel the real app uses. There is no "special test mode" — just an injected event source.

**Source:** [Textual testing guide](https://textual.textualize.io/guide/testing/), [Textual Pilot API](https://textual.textualize.io/api/pilot/)

### Implementing Pilot in Rust: The Channel Injection Pattern

The Rust equivalent requires designing the event loop to accept events from an injectable source rather than always reading from a real terminal. The recommended architecture:

```rust
// Event type for the whole application
#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    Quit,
}

// The app's event loop accepts a Receiver — the source is swappable
pub struct App {
    event_rx: mpsc::Receiver<AppEvent>,
    // ... other state
}

impl App {
    pub fn new(event_rx: mpsc::Receiver<AppEvent>) -> Self { ... }

    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            match self.event_rx.recv().await {
                Some(AppEvent::Quit) => break,
                Some(event) => self.handle_event(event),
                None => break,
            }
        }
        Ok(())
    }
}
```

### The TestPilot Harness

```rust
pub struct TestPilot {
    event_tx: mpsc::Sender<AppEvent>,
    terminal: Terminal<TestBackend>,
}

impl TestPilot {
    pub fn new(width: u16, height: u16) -> (Self, mpsc::Receiver<AppEvent>) {
        let (tx, rx) = mpsc::channel(64);
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).unwrap();
        (Self { event_tx: tx, terminal }, rx)
    }

    /// Simulate a key press
    pub async fn press(&self, key: KeyCode, modifiers: KeyModifiers) {
        self.event_tx
            .send(AppEvent::Key(KeyEvent::new(key, modifiers)))
            .await
            .unwrap();
    }

    /// Simulate character input (convenience wrapper)
    pub async fn type_text(&self, text: &str) {
        for ch in text.chars() {
            self.press(KeyCode::Char(ch), KeyModifiers::NONE).await;
        }
    }

    /// Drain the event queue and re-render (equivalent to Textual's pilot.pause())
    pub async fn pause(&mut self) {
        tokio::task::yield_now().await;
    }

    /// Assert current rendered output
    pub fn assert_lines(&self, expected: impl IntoIterator<Item = impl Into<Line>>) {
        self.terminal.backend().assert_buffer_lines(expected);
    }

    /// Take a snapshot (requires insta)
    pub fn assert_snapshot(&self, name: &str) {
        insta::assert_snapshot!(name, self.terminal.backend());
    }
}
```

**Usage in a test:**
```rust
#[tokio::test]
async fn test_text_input_widget() {
    let (mut pilot, event_rx) = TestPilot::new(40, 5);
    let mut app = App::new(event_rx);

    // Spawn app on a background task
    let app_handle = tokio::spawn(async move {
        app.run(&mut pilot.terminal).await
    });

    pilot.type_text("hello").await;
    pilot.pause().await;

    pilot.assert_lines([
        "Input: hello                           ",
        "                                        ",
    ]);

    pilot.press(KeyCode::Char('c'), KeyModifiers::CONTROL).await;
    app_handle.await.unwrap().unwrap();
}
```

### Mouse Simulation

For mouse events, extend `AppEvent` with `MouseEvent` from crossterm and add `TestPilot::click(col, row)`:

```rust
pub async fn click(&self, col: u16, row: u16) {
    use crossterm::event::{MouseButton, MouseEventKind};
    self.event_tx
        .send(AppEvent::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }))
        .await
        .unwrap();
}
```

**Source:** [Tokio unit testing guide](https://tokio.rs/tokio/topics/testing), [ratatui async event stream](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/), [Rust forum — simulating key presses](https://users.rust-lang.org/t/simulate-key-presses-to-test-the-game-loop/105428)

---

## 4. Testing Async TUI Event Loops Without a Real Terminal

### Core Principle

Separate the event *source* from the event *loop*. In production, the source reads from crossterm; in tests, the source is a `tokio::mpsc::Sender` controlled by the test. The loop itself is identical in both environments.

### Tokio Test Utilities

**`#[tokio::test]`** — standard async test entry point. Single-threaded by default (deterministic).

**`tokio::time::pause()` / `#[tokio::test(start_paused = true)]`** — makes all timer-based waits resolve immediately. Critical for testing debounce, animation timers, or auto-scroll without `sleep()` calls in tests. Requires the `test-util` feature in Cargo.toml:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util", "macros", "rt"] }
```

```rust
#[tokio::test(start_paused = true)]
async fn test_debounce_search() {
    // tokio::time::sleep(Duration::from_millis(300)) completes instantly
    let (mut pilot, rx) = TestPilot::new(80, 24);
    // ... inject events, advance time, assert
}
```

**`tokio::sync::mpsc` channels** — inject events at any point in a test; bounded channels backpressure naturally without spin-looping.

**`tokio::task::yield_now()`** — yield execution to let spawned tasks run before asserting. This is the async equivalent of Textual's `pilot.pause()`.

### Deterministic Event Sequencing

The pattern for deterministic tests is:
1. Send event via `pilot.event_tx.send(...)`.
2. Call `pilot.pause().await` (which calls `yield_now().await`) to let the app process it.
3. Assert state.

Do not use `tokio::time::sleep` for synchronization — it makes tests slow and flaky. Use `yield_now` or a bounded channel that blocks until the app drains it.

### Testing Without a Real Terminal: The TestBackend Contract

`TestBackend` implements the same `Backend` trait as the real crossterm backend. The `Terminal<TestBackend>` can call `.draw()`, compute diffs, and flush to the in-memory buffer without any OS terminal interaction. This means:

- Tests run on any platform, including Windows, macOS, Linux, and CI.
- No PTY, no `isatty` check, no ANSI escape sequences.
- Tests are deterministic (no race with terminal's buffering).

**Source:** [Tokio testing docs](https://tokio.rs/tokio/topics/testing), [crossterm async event stream](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/)

---

## 5. Testing Visual Output — The Assertion API

### Layer 1: String Content Assertions (Primary)

Use `assert_buffer_lines` for character-level rendering assertions. These are the most readable:

```rust
buf.assert_buffer_lines([
    "┌ Title ────────────────────────────────────────────────────────────────────────┐",
    "│ Content line 1                                                                │",
    "│ Content line 2                                                                │",
    "└───────────────────────────────────────────────────────────────────────────────┘",
]);
```

### Layer 2: Snapshot Assertions (Regression Detection)

Use `insta::assert_snapshot!` for visual regression tests. The snapshot captures exact character content and is stored in `tests/snapshots/`. These tests are "golden file" tests — they fail only when output changes, not when it changes in a specific way:

```rust
#[test]
fn snapshot_dialog_widget() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
    let dialog = Dialog::new("Are you sure?").with_buttons(["Yes", "No"]);
    dialog.render(buf.area, &mut buf);

    // buf implements Display for ratatui buffers
    insta::assert_snapshot!(format!("{}", buf));
}
```

### Layer 3: Style/Cell Assertions (Color and Decoration)

When color or bold/italic decoration matters, inspect cells directly:

```rust
use ratatui::style::{Color, Modifier};

let cell = &buf[(5, 0)];
assert_eq!(cell.symbol(), "E");  // character
assert_eq!(cell.style().fg, Some(Color::Red));
assert!(cell.style().add_modifier.contains(Modifier::BOLD));
```

This is the only way to assert style until ratatui adds color-aware snapshot support.

### Good Assertion API Design (for textual-rs)

Recommended layered API for textual-rs's own test helpers:

```rust
pub trait BufferAssertExt {
    /// Assert exact character content of every row
    fn assert_text(&self, lines: &[&str]);

    /// Assert a specific cell's character
    fn assert_char(&self, x: u16, y: u16, expected: char);

    /// Assert a specific cell's foreground color
    fn assert_fg(&self, x: u16, y: u16, color: Color);

    /// Assert a specific cell's background color
    fn assert_bg(&self, x: u16, y: u16, color: Color);

    /// Assert a region contains a substring somewhere on the given row
    fn assert_row_contains(&self, row: u16, substring: &str);

    /// Snapshot the entire buffer using insta
    fn assert_snapshot(&self, name: &str);
}
```

**Source:** [Buffer docs](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Buffer.html), [ratatui snapshot guide](https://ratatui.rs/recipes/testing/snapshots/)

---

## 6. Existing Rust TUI Testing Frameworks and Utilities

### ratatui-testlib (PTY-Based Integration Testing)

**Crate:** `ratatui-testlib` by raibid-labs
**Purpose:** Full integration tests in a real pseudo-terminal — the "next level up" from TestBackend.

```rust
use ratatui_testlib::TuiTestHarness;

#[tokio::test]
async fn test_full_app() {
    let harness = TuiTestHarness::new()
        .spawn(CommandBuilder::new("./my-tui-app"))
        .await
        .unwrap();

    harness.wait_for_text("Welcome").await.unwrap();
    harness.send_key(KeyCode::Down).await;
    harness.wait_for_text("Item 2 selected").await.unwrap();
}
```

**When to use:** When you need to test escape sequence handling, real terminal dimensions, signals, or Sixel graphics. Overkill for widget unit tests. Use TestBackend for those.

**Verdict:** Use for end-to-end smoke tests of the full textual-rs runtime. Not for widget unit tests.

### expectrl (Expect-Style PTY Control)

**Crate:** `expectrl` by zhiburt
**What it is:** Rust port of the Unix `expect` utility — spawn a process in a PTY, send input, wait for output patterns.
**When to use:** When testing CLI applications that are not written against ratatui/crossterm at all. Less relevant for textual-rs internals.

### tui-realm (Component Isolation via MockComponent)

`tui-realm` is a framework (not a test library) built on ratatui with React/Elm-style components. It provides `#[derive(MockComponent)]` to create test doubles of components. The concept is applicable to textual-rs: a `MockWidget` that implements the same `Widget` trait but records calls, enabling assertion on render counts or received messages.

### similar-asserts (Better assert_eq! Output)

**Crate:** `similar-asserts` by mitsuhiko
Replaces `assert_eq!` with a diff-aware version that renders colorized side-by-side diffs. Drop-in replacement:

```rust
use similar_asserts::assert_eq;
```

Especially useful for comparing `Buffer` diffs and parse tree output.

### Summary Recommendation

| Test Level | Tool | When |
|-----------|------|------|
| Widget unit | `ratatui::TestBackend` + `Buffer::assert_buffer_lines` | Always, for all widget rendering |
| Visual regression | `insta::assert_snapshot!` | After stabilizing widget appearance |
| Async app logic | `tokio::test` + channel injection | Event handling, state transitions |
| CSS/layout unit | Rust `#[test]` + `assert_eq!` + `similar-asserts` | Pure logic, no rendering |
| Property/fuzz | `proptest` | Parser, layout constraint solver |
| End-to-end | `ratatui-testlib` | Smoke tests of the full runtime |
| Interactive debugging | `expectrl` | Ad-hoc; not for CI |

---

## 7. Unit Test Patterns by Component

### 7a. CSS Parsing Tests

CSS parsing has three testable layers: tokenizer, parser (token stream to AST), and resolver (CSS property → computed style).

**Unit tests — tokenizer:**
```rust
#[test]
fn tokenizes_hex_color() {
    let tokens = tokenize("#ff0000");
    assert_eq!(tokens, vec![Token::Hash("ff0000")]);
}

#[test]
fn tokenizes_dimension() {
    let tokens = tokenize("12px");
    assert_eq!(tokens, vec![Token::Dimension(12.0, "px")]);
}
```

**Parameterized tests with rstest:**
```rust
use rstest::rstest;

#[rstest]
#[case("red",        Color::Named(NamedColor::Red))]
#[case("#ff0000",    Color::Rgb(255, 0, 0))]
#[case("rgb(1,2,3)", Color::Rgb(1, 2, 3))]
fn parses_color(#[case] input: &str, #[case] expected: Color) {
    assert_eq!(parse_color(input), Ok(expected));
}
```

**Property-based tests with proptest (parser round-trip):**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn color_serialization_roundtrips(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let color = Color::Rgb(r, g, b);
        let serialized = color.to_css_string();
        let parsed = parse_color(&serialized).unwrap();
        prop_assert_eq!(color, parsed);
    }
}
```

**Error case tests:**
```rust
#[test]
fn invalid_property_returns_error() {
    assert!(parse_property("color", "not-a-color").is_err());
}
```

### 7b. Layout Engine Tests

The layout engine takes a tree of widgets with CSS-style constraints and produces a tree of `Rect` values. Key properties to test:

```rust
#[test]
fn horizontal_split_fills_parent() {
    let parent = Rect::new(0, 0, 80, 24);
    let children = layout_horizontal(parent, &[Constraint::Fill(1), Constraint::Fill(1)]);
    // Both halves together must equal the parent width
    let total_width: u16 = children.iter().map(|r| r.width).sum();
    assert_eq!(total_width, parent.width);
}

#[test]
fn fixed_constraint_honored() {
    let parent = Rect::new(0, 0, 80, 24);
    let children = layout_horizontal(parent, &[Constraint::Length(20), Constraint::Fill(1)]);
    assert_eq!(children[0].width, 20);
    assert_eq!(children[1].width, 60);
}

// Property-based: no layout should exceed parent bounds
proptest! {
    #[test]
    fn layout_never_overflows(width in 10u16..200, height in 5u16..100) {
        let parent = Rect::new(0, 0, width, height);
        let result = layout_vertical(parent, &[Constraint::Fill(1), Constraint::Fill(1), Constraint::Fill(1)]);
        for rect in &result {
            prop_assert!(rect.right() <= parent.right());
            prop_assert!(rect.bottom() <= parent.bottom());
        }
    }
}
```

### 7c. Widget Rendering Tests

Each widget should have a rendering test at its smallest valid size, at a representative standard size, and at edge-case sizes (zero width, extremely narrow).

```rust
#[test]
fn button_renders_label() {
    let btn = Button::new("OK");
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
    btn.render(buf.area, &mut buf);
    buf.assert_buffer_lines(["[ OK ]    "]);
}

#[test]
fn button_renders_focused_state() {
    let btn = Button::new("OK").focused(true);
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
    btn.render(buf.area, &mut buf);
    // Assert on cell style for focused indicator
    let cell = &buf[(1, 0)];
    assert_eq!(cell.style().fg, Some(Color::Yellow));
}

#[test]
fn button_truncates_at_narrow_width() {
    let btn = Button::new("Long Label");
    let mut buf = Buffer::empty(Rect::new(0, 0, 6, 1));
    btn.render(buf.area, &mut buf);
    // Should not panic, should truncate gracefully
    assert_eq!(buf[(0,0)].symbol(), "[");
}
```

**Snapshot test for complex widgets:**
```rust
#[test]
fn data_table_snapshot() {
    let table = DataTable::new(vec![
        vec!["Alice", "30", "Engineer"],
        vec!["Bob",   "25", "Designer"],
    ]).with_headers(["Name", "Age", "Role"]);

    let mut buf = Buffer::empty(Rect::new(0, 0, 50, 6));
    table.render(buf.area, &mut buf);

    insta::assert_snapshot!(format!("{buf}"));
}
```

### 7d. Event Dispatch Tests

Test that the application's event handler routes events to the correct widget and produces the expected state change:

```rust
#[tokio::test]
async fn tab_key_advances_focus() {
    let (tx, rx) = mpsc::channel(8);
    let mut app = App::new(rx);

    assert_eq!(app.focused_widget_id(), "input_1");
    tx.send(AppEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    app.process_pending_events().await;

    assert_eq!(app.focused_widget_id(), "input_2");
}

#[test]
fn escape_key_closes_modal() {
    let mut app = App::default();
    app.open_modal("Confirm");
    assert!(app.is_modal_open());

    app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!app.is_modal_open());
}
```

**Source:** [rstest crate](https://github.com/la10736/rstest), [proptest GitHub](https://github.com/proptest-rs/proptest), [ratatui layout docs](https://ratatui.rs/concepts/layout/)

---

## 8. The TestApp Harness — Recommended Design

The `TestApp` harness should be a single struct that owns all the pieces needed to drive and inspect a textual-rs application in tests. It should model Textual's `run_test()` / `Pilot` closely.

```rust
/// Full test harness for a textual-rs application.
/// Create this at the start of each test, use it to drive the app,
/// then drop it when done.
pub struct TestApp<A: Application> {
    app: A,
    terminal: Terminal<TestBackend>,
    event_tx: mpsc::Sender<AppEvent>,
    event_rx: mpsc::Receiver<AppEvent>,
    size: (u16, u16),
}

impl<A: Application> TestApp<A> {
    const DEFAULT_SIZE: (u16, u16) = (80, 24);

    pub fn new(app: A) -> Self {
        Self::with_size(app, Self::DEFAULT_SIZE)
    }

    pub fn with_size(app: A, size: (u16, u16)) -> Self {
        let (event_tx, event_rx) = mpsc::channel(256);
        let backend = TestBackend::new(size.0, size.1);
        let terminal = Terminal::new(backend).unwrap();
        Self { app, terminal, event_tx, event_rx, size }
    }

    // --- Event injection ---

    pub async fn press(&self, key: KeyCode) {
        self.press_with_modifiers(key, KeyModifiers::NONE).await;
    }

    pub async fn press_with_modifiers(&self, key: KeyCode, mods: KeyModifiers) {
        self.event_tx
            .send(AppEvent::Key(KeyEvent::new(key, mods)))
            .await
            .unwrap();
    }

    pub async fn type_text(&self, text: &str) {
        for ch in text.chars() {
            self.press(KeyCode::Char(ch)).await;
        }
    }

    pub async fn click(&self, col: u16, row: u16) { ... }

    /// Drain all pending events and re-render.
    /// Call this after every interaction before asserting.
    pub async fn settle(&mut self) {
        tokio::task::yield_now().await;
        // Process any queued events
        while let Ok(event) = self.event_rx.try_recv() {
            self.app.handle_event(event);
        }
        self.terminal.draw(|f| self.app.render(f)).unwrap();
    }

    // --- Assertions ---

    pub fn assert_text(&self, lines: &[&str]) {
        self.terminal.backend().assert_buffer_lines(lines.iter().copied());
    }

    pub fn assert_snapshot(&self, name: &str) {
        insta::assert_snapshot!(name, self.terminal.backend());
    }

    pub fn assert_snapshot_inline(&self, snapshot: &str) {
        // Used with insta's inline snapshot syntax
        insta::assert_snapshot!(self.terminal.backend(), @snapshot);
    }

    pub fn assert_cell_char(&self, x: u16, y: u16, expected: char) {
        let cell = &self.terminal.backend().buffer()[(x, y)];
        assert_eq!(cell.symbol(), expected.to_string().as_str());
    }

    pub fn assert_cell_fg(&self, x: u16, y: u16, expected: Color) {
        let cell = &self.terminal.backend().buffer()[(x, y)];
        assert_eq!(cell.style().fg, Some(expected));
    }

    pub fn buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    pub fn app(&self) -> &A {
        &self.app
    }
}

// Example usage
#[tokio::test]
async fn input_widget_accepts_text() {
    let mut t = TestApp::new(MyApp::default());
    t.settle().await;                              // initial render

    t.type_text("hello world").await;
    t.settle().await;

    t.assert_text(&[
        "Input: hello world                      ",
        "                                        ",
    ]);
}

#[tokio::test]
async fn modal_opens_on_ctrl_m() {
    let mut t = TestApp::new(MyApp::default());
    t.settle().await;

    assert!(!t.app().is_modal_open());
    t.press_with_modifiers(KeyCode::Char('m'), KeyModifiers::CONTROL).await;
    t.settle().await;

    assert!(t.app().is_modal_open());
    t.assert_snapshot("modal_open");
}
```

---

## 9. Crate Recommendations

### Essential (Include in textual-rs dev-dependencies)

| Crate | Version | Purpose | Confidence |
|-------|---------|---------|-----------|
| `insta` | `1` | Snapshot assertions for rendered output | HIGH |
| `similar-asserts` | `1` | Colorized diffs in `assert_eq!` failures | HIGH |
| `rstest` | `0.23` | Parameterized and fixture-based tests | HIGH |
| `proptest` | `1` | Property-based tests for parser and layout | HIGH |
| `tokio` (test-util feature) | `1` | Async test runtime + time control | HIGH |

### Recommended (Add when needed)

| Crate | Purpose | Confidence |
|-------|---------|-----------|
| `cargo-insta` (CLI) | Interactive snapshot review | HIGH |
| `cargo-nextest` (CLI) | Parallel test runner, better CI output | HIGH |
| `ratatui-testlib` | End-to-end PTY integration tests | MEDIUM |
| `cargo-llvm-cov` (CLI) | Coverage reporting (cross-platform) | HIGH |

### Avoid

| Crate | Reason |
|-------|--------|
| `cargo-tarpaulin` | Linux-only; `cargo-llvm-cov` is the better cross-platform option |
| `expectrl` | PTY process control; `ratatui-testlib` already wraps this pattern better |
| `mockers` / `mockall` | Trait-based mocking is rarely needed in well-designed TUI code; prefer dependency injection |

---

## 10. Project-Level Test Structure

```
textual-rs/
├── src/
│   ├── css/
│   │   ├── tokenizer.rs          # #[cfg(test)] mod tests { } inline
│   │   ├── parser.rs             # inline unit tests
│   │   └── resolver.rs           # inline unit tests
│   ├── layout/
│   │   ├── engine.rs             # inline unit + proptest
│   │   └── constraints.rs        # inline unit + proptest
│   ├── widgets/
│   │   ├── button.rs             # inline rendering tests
│   │   ├── input.rs              # inline rendering + behavior tests
│   │   └── table.rs              # inline rendering tests
│   └── app/
│       └── event_loop.rs         # inline async tests
├── tests/
│   ├── integration/
│   │   ├── harness.rs            # TestApp definition (shared helper)
│   │   ├── text_input_test.rs
│   │   ├── navigation_test.rs
│   │   └── css_full_pipeline_test.rs
│   └── snapshots/                # insta .snap files (committed to git)
│       ├── button_renders_label.snap
│       └── data_table_snapshot.snap
└── Cargo.toml
```

**Conventions:**
- Unit tests (pure logic, no rendering) live inline in `src/` modules using `#[cfg(test)]`.
- Integration tests using `TestApp` live in `tests/integration/`.
- All `insta` snapshot files are committed to git and reviewed via `cargo insta review`.
- All tests run with `cargo nextest run`.
- Terminal size constant: define `const TEST_TERM_SIZE: (u16, u16) = (80, 24)` in `tests/integration/harness.rs`.

---

## Sources

- [ratatui TestBackend docs](https://docs.rs/ratatui/latest/ratatui/backend/struct.TestBackend.html)
- [ratatui Buffer docs](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Buffer.html)
- [ratatui snapshot testing guide](https://ratatui.rs/recipes/testing/snapshots/)
- [ratatui TDD discussion #78](https://github.com/ratatui/ratatui/discussions/78)
- [ratatui CONTRIBUTING.md](https://github.com/ratatui/ratatui/blob/main/CONTRIBUTING.md)
- [insta.rs](https://insta.rs/)
- [insta GitHub](https://github.com/mitsuhiko/insta)
- [insta assert_snapshot docs](https://docs.rs/insta/latest/insta/macro.assert_snapshot.html)
- [similar-asserts crate](https://crates.io/crates/similar-asserts)
- [Textual testing guide](https://textual.textualize.io/guide/testing/)
- [Textual Pilot API](https://textual.textualize.io/api/pilot/)
- [Tokio unit testing docs](https://tokio.rs/tokio/topics/testing)
- [ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/)
- [ratatui-testlib GitHub](https://github.com/raibid-labs/ratatui-testlib)
- [ratatui-testlib docs](https://docs.rs/ratatui-testlib)
- [rstest crate](https://crates.io/crates/rstest)
- [proptest GitHub](https://github.com/proptest-rs/proptest)
- [cargo-nextest](https://nexte.st/)
- [cargo-llvm-cov GitHub](https://github.com/taiki-e/cargo-llvm-cov)
- [tui-realm GitHub](https://github.com/veeso/tui-realm)
- [expectrl GitHub](https://github.com/zhiburt/expectrl)
- [ratatui layout concepts](https://ratatui.rs/concepts/layout/)
- [Rust forum — testing TUI apps](https://users.rust-lang.org/t/how-to-test-tui-applications/78666)
