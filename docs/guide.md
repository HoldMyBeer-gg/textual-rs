# textual-rs User Guide

textual-rs is a Rust port of the [Textual](https://textual.textualize.io) Python TUI framework. It lets you build beautiful terminal UIs by declaring widgets, styling them with CSS, and reacting to events -- all in safe Rust.

## Getting Started

Add textual-rs to your `Cargo.toml`:

```toml
[dependencies]
textual-rs = "0.2"
```

### Minimal Application

Every textual-rs application needs three things: a root screen widget, an `App`, and a call to `run()`.

```rust
use textual_rs::{App, Widget, Header, Footer, Label};
use textual_rs::widget::context::AppContext;
use ratatui::{buffer::Buffer, layout::Rect};

struct MyScreen;

impl Widget for MyScreen {
    fn widget_type_name(&self) -> &'static str { "MyScreen" }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("My App")),
            Box::new(Label::new("Hello, world!")),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    App::new(|| Box::new(MyScreen)).run()
}
```

`App::new` takes a factory closure that returns the root screen. The framework calls it once during startup.

### Adding CSS

Style your application with TCSS (Textual CSS):

```rust
let mut app = App::new(|| Box::new(MyScreen))
    .with_css("MyScreen { background: #0a0a0f; color: #e0e0e0; }");
app.run()?;
```

Or load from a file with hot-reload support (re-parsed every 2 seconds during development):

```rust
let mut app = App::new(|| Box::new(MyScreen))
    .with_css_file("styles/app.tcss");
app.run()?;
```

You can also embed CSS at compile time:

```rust
let mut app = App::new(|| Box::new(MyScreen))
    .with_css(include_str!("app.tcss"));
```

---

## Widget System

Widgets are the building blocks of every textual-rs UI. They form a tree: `App > Screen > Widget hierarchy`.

### The Widget Trait

Every widget implements the `Widget` trait:

```rust
pub trait Widget: 'static {
    /// Paint this widget into the terminal buffer.
    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer);

    /// CSS type selector name (e.g. "Button", "MyWidget").
    fn widget_type_name(&self) -> &'static str;

    /// Declare child widgets. Called once at mount time.
    fn compose(&self) -> Vec<Box<dyn Widget>> { vec![] }

    /// Called when inserted into the widget tree.
    fn on_mount(&self, _id: WidgetId) {}

    /// Whether this widget participates in Tab focus cycling.
    fn can_focus(&self) -> bool { false }

    /// CSS class names (e.g. &["primary", "active"]).
    fn classes(&self) -> &[&str] { &[] }

    /// Element ID for #id CSS selectors.
    fn id(&self) -> Option<&str> { None }

    /// Handle a dispatched event/message.
    fn on_event(&self, _event: &dyn std::any::Any, _ctx: &AppContext)
        -> EventPropagation { EventPropagation::Continue }

    /// Declare key bindings for this widget.
    fn key_bindings(&self) -> &[KeyBinding] { &[] }

    /// Handle a key binding action.
    fn on_action(&self, _action: &str, _ctx: &AppContext) {}

    /// Context menu items for right-click.
    fn context_menu_items(&self) -> Vec<ContextMenuItem> { Vec::new() }

    /// Action triggered on mouse click.
    fn click_action(&self) -> Option<&str> { None }
}
```

### compose() -- Building the Widget Tree

Container widgets return their children from `compose()`. The framework calls this once at mount time and inserts the children into the arena.

```rust
struct Dashboard;

impl Widget for Dashboard {
    fn widget_type_name(&self) -> &'static str { "Dashboard" }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Dashboard")),
            Box::new(Horizontal::with_children(vec![
                Box::new(sidebar()),
                Box::new(main_content()),
            ])),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}
```

### render() -- Custom Painting

Leaf widgets paint their content in `render()`. The `area` parameter is pre-clipped to the widget's computed layout rectangle. Use `ctx.text_style(id)` to get the CSS-computed foreground/background style.

```rust
fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
    let style = ctx.text_style(self.my_id.get());
    buf.set_string(area.x, area.y, "Hello!", style);
}
```

**Important:** Use `get_untracked()` on reactive values inside `render()` to avoid creating tracking dependencies that would cause infinite re-render loops.

### Derive Macro

For simple widgets, use `#[derive(Widget)]` to scaffold the trait:

```rust
use textual_rs::Widget;

#[derive(Widget)]
struct MyWidget;
```

---

## CSS / TCSS Styling

textual-rs uses TCSS (Textual CSS), a subset of CSS designed for terminal UIs. Stylesheets are parsed and applied via a specificity-based cascade, just like web CSS.

### Selectors

| Selector | Syntax | Example |
|----------|--------|---------|
| Type | `WidgetTypeName` | `Button { ... }` |
| Class | `.classname` | `.primary { ... }` |
| ID | `#id` | `#sidebar { ... }` |
| Universal | `*` | `* { color: white; }` |
| Pseudo-class | `:pseudo` | `Button:focus { ... }` |

Supported pseudo-classes: `:focus`, `:hover`, `:disabled`.

### Example Stylesheet

```css
Screen {
    background: $background;
    color: $foreground;
}

Header {
    background: $panel;
    color: $primary;
    height: 1;
}

*:focus {
    border: tall #00ffa3;
}

Button {
    border: inner;
    min-width: 16;
    height: 3;
    color: $foreground;
    background: $surface;
}

Button.primary {
    background: $primary;
    color: #ffffff;
}

Input {
    border: tall #4a4a5a;
    height: 3;
    background: $panel;
}
```

### Specificity

Rules are applied in specificity order (lowest to highest):

1. Built-in widget defaults (framework-provided)
2. User stylesheets (`.with_css()` / `.with_css_file()`)
3. Pseudo-class rules (`:focus`, `:hover`) override base rules

### Theme Variables

Use `$variable` syntax to reference the active theme's color palette. Variables resolve to RGB values at cascade time.

```css
MyWidget {
    background: $primary;
    color: $foreground;
    border: tall $accent;
}
```

See the [CSS Reference](css-reference.md) for all available variables and properties.

---

## Layout System

textual-rs uses [Taffy](https://github.com/DioxusLabs/taffy) for flexbox layout.

### Layout Direction

By default, widgets stack vertically. Set `layout-direction: horizontal` for side-by-side layout:

```css
MainRegion {
    layout-direction: horizontal;
    flex-grow: 1;
}
```

Or use the built-in container widgets:

```rust
// Vertical stacking (default)
Vertical::with_children(vec![...])

// Horizontal side-by-side
Horizontal::with_children(vec![...])
```

### Flex Grow

Distribute remaining space proportionally with `flex-grow`:

```css
ChatLog { flex-grow: 1; }    /* Takes all remaining space */
Sidebar { width: 20; }       /* Fixed 20-column sidebar */
```

### Sizing

```css
MyWidget {
    width: 40;          /* Fixed columns */
    height: 10;         /* Fixed rows */
    min-width: 20;      /* Minimum size */
    max-height: 50%;    /* Percentage of parent */
}
```

Dimension values: plain numbers (columns/rows), percentages (`50%`), fractional units (`1fr`), or `auto`.

### Padding and Margin

```css
MyWidget {
    padding: 1;         /* All sides */
    padding: 1 2;       /* Vertical Horizontal */
    padding: 1 2 3 4;   /* Top Right Bottom Left */
    margin: 1;
}
```

### Dock Layout

Pin widgets to edges of their parent:

```css
Header { dock: top; }
Footer { dock: bottom; }
Sidebar { dock: left; }
```

### Grid Layout

```css
GridContainer {
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    grid-template-rows: auto 1fr;
    keyline: $primary;       /* Separator lines between cells */
}
```

---

## Events and Messages

### Key Bindings

Declare keyboard shortcuts by implementing `key_bindings()`:

```rust
use crossterm::event::{KeyCode, KeyModifiers};
use textual_rs::event::keybinding::KeyBinding;

fn key_bindings(&self) -> &[KeyBinding] {
    &[
        KeyBinding {
            key: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            action: "save",
            description: "Save",
            show: true,     // Show in Footer
        },
        KeyBinding {
            key: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            action: "submit",
            description: "Submit",
            show: false,
        },
    ]
}

fn on_action(&self, action: &str, ctx: &AppContext) {
    match action {
        "save" => { /* handle save */ }
        "submit" => { /* handle submit */ }
        _ => {}
    }
}
```

Bindings with `show: true` appear in the `Footer` widget and the command palette.

### Event Bubbling

Events bubble up the widget tree from child to parent. Return `EventPropagation::Stop` to consume an event:

```rust
fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
    if let Some(msg) = event.downcast_ref::<MyMessage>() {
        // Handle the message
        return EventPropagation::Stop;
    }
    EventPropagation::Continue
}
```

### Posting Messages

Send typed messages between widgets:

```rust
// From on_action or on_event (takes &self):
ctx.post_message(my_id, MyCustomMessage { value: 42 });

// Alias:
ctx.notify(my_id, MyCustomMessage { value: 42 });
```

Messages bubble up from the source widget, and parent widgets can handle them in `on_event()`.

### Mouse Events

Widgets that respond to clicks implement `click_action()`:

```rust
fn click_action(&self) -> Option<&str> {
    Some("toggle")  // Same action as Space/Enter key binding
}
```

The framework handles click-to-focus automatically. Right-click triggers context menus (see below).

### Context Menus

Provide right-click menu items:

```rust
fn context_menu_items(&self) -> Vec<ContextMenuItem> {
    vec![
        ContextMenuItem::new("Copy", "copy").with_shortcut("Ctrl+C"),
        ContextMenuItem::new("Paste", "paste").with_shortcut("Ctrl+V"),
        ContextMenuItem::new("Delete", "delete"),
    ]
}
```

---

## Reactive State

`Reactive<T>` wraps a value that triggers automatic re-renders when changed. It is built on `reactive_graph` signals.

```rust
use textual_rs::reactive::Reactive;

struct Counter {
    count: Reactive<i32>,
}

impl Counter {
    fn new() -> Self {
        Self { count: Reactive::new(0) }
    }
}

impl Widget for Counter {
    // ...

    fn on_action(&self, action: &str, _ctx: &AppContext) {
        match action {
            "increment" => self.count.update(|v| *v += 1),
            "decrement" => self.count.update(|v| *v -= 1),
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // IMPORTANT: use get_untracked() in render() to avoid tracking loops
        let value = self.count.get_untracked();
        let text = format!("Count: {}", value);
        buf.set_string(area.x, area.y, &text, Style::default());
    }
}
```

### API

| Method | Description |
|--------|-------------|
| `Reactive::new(value)` | Create a new reactive value |
| `.get()` | Read with tracking (creates dependency) |
| `.get_untracked()` | Read without tracking (use in render!) |
| `.set(value)` | Set new value, notify dependents |
| `.update(\|v\| ...)` | Mutate in-place, notify dependents |
| `.signal()` | Get inner `ArcRwSignal<T>` for use in closures |

### Computed Values

Derive values from reactive sources:

```rust
use textual_rs::reactive::{Reactive, ComputedReactive};

let count = Reactive::new(3);
let sig = count.signal();
let doubled = ComputedReactive::new(move |_| sig.get() * 2);
assert_eq!(doubled.get_untracked(), 6);
```

---

## Screen Stack

textual-rs supports a screen stack for modal flows (settings panels, dialogs, etc.):

```rust
// Push a new screen (from on_action with &self):
ctx.push_screen_deferred(Box::new(SettingsScreen));

// Pop back to previous screen:
ctx.pop_screen_deferred();
```

---

## Testing

textual-rs includes a headless testing harness for automated UI tests.

### TestApp

`TestApp` creates an `App` with a `TestBackend` (no real terminal needed):

```rust
use textual_rs::TestApp;

#[test]
fn test_my_widget() {
    let test_app = TestApp::new(80, 24, || Box::new(MyScreen));
    // Assert on the rendered buffer
}
```

For tests that need proper CSS styling:

```rust
let test_app = TestApp::new_styled(80, 24, "Button { height: 3; }", || Box::new(MyScreen));
```

### Pilot -- Simulating Input

`Pilot` lets you simulate user interaction:

```rust
#[tokio::test]
async fn test_typing() {
    let mut test_app = TestApp::new(80, 24, || Box::new(MyScreen));
    let mut pilot = test_app.pilot();

    pilot.press(KeyCode::Tab).await;          // Press a key
    pilot.type_text("hello").await;           // Type a string
    pilot.click(10, 5).await;                 // Click at (col, row)
    pilot.press_with_modifiers(               // Key + modifiers
        KeyCode::Char('s'),
        KeyModifiers::CONTROL,
    ).await;
    pilot.settle().await;                     // Wait for quiescence
}
```

### Buffer Assertions

Assert on rendered terminal content:

```rust
use textual_rs::testing::assertions::{assert_buffer_lines, assert_cell};

// Check entire rows
assert_buffer_lines(test_app.buffer(), &[
    "Hello",
    "World",
]);

// Check a single cell
assert_cell(test_app.buffer(), 0, 0, "H");
```

### Snapshot Testing

Use the `TestBackend`'s `Display` implementation with [insta](https://insta.rs):

```rust
insta::assert_display_snapshot!(test_app.backend());
```

---

## Themes

textual-rs ships with 7 built-in themes. Press **Ctrl+T** at runtime to cycle through them.

### Built-in Themes

| Name | Style |
|------|-------|
| `textual-dark` | Default dark theme (Textual's signature palette) |
| `textual-light` | Light variant |
| `tokyo-night` | Clean dark, inspired by Tokyo Night |
| `nord` | Arctic, north-bluish palette |
| `gruvbox` | Retro groove with warm earth tones |
| `dracula` | Dark theme with vibrant colors |
| `catppuccin` | Soothing pastel (Mocha variant) |

### Theme Variables

All themes provide the same set of semantic color variables:

| Variable | Default (textual-dark) |
|----------|----------------------|
| `$primary` | `#0178d4` |
| `$secondary` | `#004578` |
| `$accent` | `#ffa62b` |
| `$surface` | `#1e1e1e` |
| `$panel` | blended surface+primary |
| `$background` | `#121212` |
| `$foreground` | `#e0e0e0` |
| `$success` | `#4ebf71` |
| `$warning` | `#ffa62b` |
| `$error` | `#ba3c5b` |

Each variable supports lighten/darken modifiers: `$primary-lighten-1` through `$primary-lighten-3`, `$primary-darken-1` through `$primary-darken-3`.

### Custom Themes

Set a custom theme on the `AppContext`:

```rust
use textual_rs::css::theme::Theme;

let mut theme = Theme {
    name: "my-theme".to_string(),
    primary: (0, 200, 100),
    secondary: (50, 100, 150),
    accent: (255, 100, 50),
    surface: (30, 30, 30),
    panel: (40, 40, 50),
    background: (20, 20, 20),
    foreground: (230, 230, 230),
    success: (0, 200, 100),
    warning: (255, 200, 0),
    error: (255, 50, 50),
    dark: true,
    luminosity_spread: 0.15,
    variables: HashMap::new(),
};

// Override individual variables:
theme.variables.insert("primary".to_string(), TcssColor::Rgb(100, 200, 255));
```

---

## Workers

Run background tasks without blocking the UI. Workers execute on the Tokio runtime and deliver results as messages.

### Basic Worker

```rust
fn on_mount(&self, id: WidgetId) {
    self.my_id.set(id);
}

fn on_action(&self, action: &str, ctx: &AppContext) {
    if action == "fetch" {
        ctx.run_worker(self.my_id.get(), async {
            // Do async work...
            tokio::time::sleep(Duration::from_secs(2)).await;
            "result data".to_string()
        });
    }
}

fn on_event(&self, event: &dyn Any, _ctx: &AppContext) -> EventPropagation {
    if let Some(result) = event.downcast_ref::<WorkerResult<String>>() {
        // Handle the result
        self.data.set(result.value.clone());
        return EventPropagation::Stop;
    }
    EventPropagation::Continue
}
```

### Worker with Progress

Stream incremental updates during long-running tasks:

```rust
ctx.run_worker_with_progress(my_id, |progress_tx| {
    Box::pin(async move {
        for i in 0..100 {
            let _ = progress_tx.send(i as f32 / 100.0);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        "complete"
    })
});

// Handle progress in on_event:
if let Some(progress) = event.downcast_ref::<WorkerProgress<f32>>() {
    self.progress.set(progress.progress);
}
```

Workers are automatically cancelled when their owning widget is unmounted.

---

## Command Palette

Press **Ctrl+P** to open the command palette -- a fuzzy-search overlay that lists all visible key bindings and registered commands.

Register app-level commands:

```rust
let mut app = App::new(|| Box::new(MyScreen));
app.register_command("Reload Config", "reload-config");
app.register_command("Toggle Debug", "toggle-debug");
```

---

## ScrollRegion — Direct Buffer Scrolling

`ScrollView` is for composed child widgets. When your widget does its own buffer writes (custom styling, per-cell colors), use `ScrollRegion` + `ScrollContent` instead:

```rust
use textual_rs::{ScrollContent, ScrollRegion};
use textual_rs::widget::context::AppContext;
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style}};

struct MessagePane {
    lines: Vec<(String, Color)>,
}

impl ScrollContent for MessagePane {
    fn content_height(&self) -> usize {
        self.lines.len()
    }

    fn render_content(&self, _ctx: &AppContext, buf: &mut Buffer, area: Rect, scroll_offset: usize) {
        for (i, (text, color)) in self.lines.iter().enumerate().skip(scroll_offset).take(area.height as usize) {
            let row = (i - scroll_offset) as u16;
            let style = Style::default().fg(*color);
            buf.set_string(area.x, area.y + row, text, style);
        }
    }
}

// In compose():
Box::new(ScrollRegion::new(Box::new(MessagePane { lines: vec![] })))
```

`ScrollRegion` handles Up/Down/PageUp/PageDown/Home/End key bindings, mouse wheel, and draws a sub-cell vertical scrollbar automatically. The `render_content` callback receives the already-clipped viewport rect and the scroll offset — draw only what's visible.

---

## Built-in Widgets

textual-rs ships with 22+ ready-to-use widgets:

### Layout

| Widget | Description |
|--------|-------------|
| `Vertical` | Stacks children vertically (default layout direction) |
| `Horizontal` | Arranges children side-by-side horizontally |
| `ScrollView` | Scrollable container for composed child widgets |
| `ScrollRegion` | Scrollable viewport for direct buffer rendering — implement `ScrollContent` |

### Display

| Widget | Description |
|--------|-------------|
| `Label` | Static text display |
| `Placeholder` | Bordered placeholder with optional label |
| `ProgressBar` | Determinate progress indicator (0.0 to 1.0) |
| `Sparkline` | Inline sparkline chart from a Vec of f64 values |
| `Markdown` | Inline Markdown renderer (headers, lists, code blocks, emphasis) |
| `Log` | Append-only scrolling log (push_line to add text) |

### Form Controls

| Widget | Description |
|--------|-------------|
| `Button` | Clickable button with variants (Default, Primary, Success, Warning, Error) |
| `Input` | Single-line text input with optional validation |
| `TextArea` | Multi-line text editor |
| `Checkbox` | Toggle checkbox with label |
| `Switch` | On/off toggle switch |
| `RadioButton` | Single radio option (usually used inside RadioSet) |
| `RadioSet` | Group of mutually exclusive radio buttons |
| `Select` | Dropdown selection with popup overlay |

### Data Display

| Widget | Description |
|--------|-------------|
| `DataTable` | Sortable, scrollable data table with column definitions |
| `ListView` | Scrollable list of selectable items |
| `Tree` | Hierarchical tree with expand/collapse |

### Navigation

| Widget | Description |
|--------|-------------|
| `Header` | Application header bar with title and optional subtitle |
| `Footer` | Shows key bindings from focused widget |
| `Tabs` / `TabbedContent` | Tab bar with switchable panes |
| `Collapsible` | Expandable/collapsible content section |

### System

| Widget | Description |
|--------|-------------|
| `CommandPalette` | Fuzzy-search command overlay (Ctrl+P) |

---

## Global Key Bindings

These work everywhere in a textual-rs application:

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus to next focusable widget |
| `Shift+Tab` | Cycle focus backward |
| `Ctrl+C` (twice) | Quit the application |
| `Ctrl+T` | Cycle through built-in themes |
| `Ctrl+P` | Open command palette |
| Right-click | Open context menu (if widget provides items) |

### Quitting programmatically

Call `ctx.quit()` from any `on_action` handler to exit cleanly:

```rust
KeyBinding { key: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, action: "quit", description: "Quit", show: true },

fn on_action(&self, action: &str, ctx: &AppContext) {
    if action == "quit" {
        ctx.quit();
    }
}
```

There is no global `q` binding — quit behaviour is the application's responsibility.

---

## Debugging

### Dispatch tracing

Set `TEXTUAL_RS_DEBUG=1` to log every message dispatch to stderr:

```
TEXTUAL_RS_DEBUG=1 cargo run
```

Output shows the message type, the full widget bubble chain, where `Stop` was returned, or a dead-letter warning when the message reaches the root unhandled:

```
[textual-rs dispatch] myapp::workers::RefreshResult → [DataTable, Screen]
[textual-rs dispatch]   no handler matched — message bubbled to root unhandled
```

This is particularly useful when worker results go silent — the most common cause is a type mismatch in `downcast_ref` inside `on_event`.

---

## Full Example: IRC Client Layout

Here is a complete example showing a weechat-style IRC client with sidebars, chat log, and input bar:

```rust
use textual_rs::{App, Widget, Header, Footer, ListView, Log, Input};
use textual_rs::widget::context::AppContext;
use ratatui::{buffer::Buffer, layout::Rect};

struct ChannelPane;
impl Widget for ChannelPane {
    fn widget_type_name(&self) -> &'static str { "ChannelPane" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec![
            "#general".into(), "#rust".into(), "#help".into(),
        ]))]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

struct ChatLog;
impl Widget for ChatLog {
    fn widget_type_name(&self) -> &'static str { "ChatLog" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let log = Log::new();
        log.push_line("[12:01] <alice> hello everyone".into());
        log.push_line("[12:02] <bob> hey alice!".into());
        vec![Box::new(log)]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

struct MainRegion;
impl Widget for MainRegion {
    fn widget_type_name(&self) -> &'static str { "MainRegion" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ChannelPane), Box::new(ChatLog)]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

struct IrcScreen;
impl Widget for IrcScreen {
    fn widget_type_name(&self) -> &'static str { "IrcScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("IRC Client")),
            Box::new(MainRegion),
            Box::new(Input::new("Type a message...")),
            Box::new(Footer),
        ]
    }
    fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    App::new(|| Box::new(IrcScreen))
        .with_css("
            MainRegion { layout-direction: horizontal; flex-grow: 1; }
            ChannelPane { width: 20; border: tall #4a4a5a; background: $panel; }
            ChatLog { flex-grow: 1; background: $background; color: $success; }
        ")
        .run()
}
```
