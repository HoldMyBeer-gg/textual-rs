# textual-rs

A Rust TUI framework inspired by [Python Textual](https://textual.textualize.io), delivering modern terminal interfaces with CSS styling, reactive state, and rich widgets.

## Features

### Widgets (30+)

| Widget | Description |
|--------|-------------|
| `Button` | Clickable with 3D depth, press animation |
| `Input` | Single-line text field with clipboard (Ctrl+C/V), text selection, validation states |
| `MaskedInput` | Password/secret input with configurable mask character |
| `TextArea` | Multi-line editor with selection, clipboard, scroll |
| `Checkbox` | Toggle with checked/unchecked/indeterminate states |
| `Switch` | Animated pill-shaped sliding toggle |
| `RadioButton` / `RadioSet` | Mutually exclusive option groups |
| `Select` | Dropdown overlay with search filtering |
| `ListView` | Scrollable item list with keyboard navigation |
| `DataTable` | Columns, zebra striping, sortable headers, cursor row |
| `Tree` / `TreeView` | Hierarchical collapsible nodes |
| `DirectoryTree` | File-system tree browser with lazy loading |
| `Tabs` / `TabbedContent` | Tab bar with animated underline indicator |
| `Markdown` | Rendered markdown with headings, lists, code blocks |
| `Log` | Scrollable append-only message log |
| `RichLog` | Scrollable rich-text log with styled lines |
| `ProgressBar` | Determinate progress with half-block fill |
| `Sparkline` | Braille-character mini charts |
| `Collapsible` | Expandable/collapsible sections |
| `Placeholder` | Quadrant cross-hatch placeholder |
| `ScrollView` | Scrollable container with eighth-block scrollbar |
| `Header` / `Footer` | App chrome with key badges |
| `Horizontal` / `Vertical` | Layout containers |
| `Label` | Static text display |
| `LoadingIndicator` | Animated spinner overlay on any widget |
| `Toast` | Stacked transient notifications with severity levels |
| `ContextMenu` | Right-click floating overlay menus |
| `ModalScreen` | Screen overlay for modal dialogs with dismiss result |
| `CommandPalette` | Ctrl+P command discovery overlay |

### Styling

- **CSS/TCSS engine** -- type, class, and ID selectors with cascade and specificity
- **Theme variables** -- `$primary`, `$surface`, `$accent`, with shade generation (`$primary-lighten-2`, `$accent-darken-1`)
- **7 built-in themes** -- textual-dark, textual-light, tokyo-night, nord, gruvbox, dracula, catppuccin
- **Runtime theme switching** -- Ctrl+T to cycle themes
- **8 border styles** -- ascii, blank, double, heavy, inner, outer, round, tall (McGugan Box)
- **Box model** -- padding, margin, min/max width/height, flex grow
- **Pseudo-classes** -- `:focus`, `:hover` with automatic visual feedback
- **Custom themes** -- override any variable via user CSS

### Layout

- **Flexbox** via Taffy -- `layout-direction: vertical | horizontal`, `flex-grow`, `align-items`, `justify-content`
- **Dock** -- `dock: top | bottom | left | right` for fixed chrome
- **Responsive** -- recomputes on terminal resize

### Screens

- **Screen stack** -- push/pop screens with `push_screen()` and `pop_screen()`
- **Modal screens** -- `ModalScreen` overlays that return a result via `push_screen_wait()`
- **Screen lifecycle** -- mount/unmount callbacks on screen transitions

### Interaction

- **Mouse** -- click, hover tracking, scroll wheel, right-click context menus
- **Mouse capture stack** -- push/pop mouse-enabled state; Shift+click bypasses capture for native text selection
- **Keyboard** -- key bindings, Tab/Shift+Tab focus cycling, Ctrl+C/X/V clipboard
- **Focus system** -- visible focus indicators with border color changes
- **Hyperlinks** -- clickable OSC 8 terminal hyperlinks via `LinkedLine` / `LinkedSpan`

### Reactive State

- **`Reactive<T>`** -- property wrapper that triggers re-renders on change
- **Signals** -- built on `reactive_graph` (from Leptos); `ArcRwSignal` / `ArcMemo`
- **Message passing** -- typed async messages bubble up the widget tree

### Background Work

- **Worker API** -- `run_worker()` and `run_worker_with_progress()` for async tasks off the UI thread
- **Command palette** -- register commands via `CommandRegistry`, invoke with Ctrl+P

### Animation

- **Tween system** -- easing functions (linear, ease-in-out-cubic, ease-out-cubic)
- **Built-in animations** -- Switch toggle slide, Tab underline slide
- **30fps render tick** -- ratatui diff makes unchanged frames nearly free

### Rendering

- **Sub-cell graphics** -- half-block, eighth-block, quadrant, and braille characters
- **McGugan Box borders** -- thin tall borders using Unicode block elements
- **Terminal capability detection** -- color depth, Unicode support auto-detected

### Testing

- **`TestApp`** -- headless application harness, no terminal needed
- **`Pilot`** -- simulated keyboard/mouse input for integration tests
- **Snapshot testing** -- insta-based plain-text buffer snapshots
- **21 visual regression tests** included

### Cross-platform

- Windows 10+, macOS, Linux
- ratatui 0.30 + crossterm 0.29 backend
- Stable Rust (no nightly required), MSRV 1.88

## Quick Start

```bash
cargo add textual-rs
```

Or add manually to `Cargo.toml`:

```toml
[dependencies]
textual-rs = "0.3"
```

```rust
use textual_rs::{App, Widget, Label, Button, Header, Footer};
use textual_rs::widget::context::AppContext;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct MyScreen;

impl Widget for MyScreen {
    fn widget_type_name(&self) -> &'static str { "MyScreen" }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("My App")),
            Box::new(Label::new("Hello, textual-rs!")),
            Box::new(Button::new("Click Me")),
            Box::new(Footer),
        ]
    }
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(MyScreen))
        .with_css("MyScreen { layout-direction: vertical; background: $background; color: $foreground; }");
    app.run()
}
```

## Styling with CSS

```css
Screen {
    background: $background;
    color: $foreground;
}

Button {
    border: inner;
    min-width: 16;
    height: 3;
    background: $surface;
    color: $foreground;
}

Button.primary {
    background: $primary;
    color: #ffffff;
}

*:focus {
    border: tall $accent;
}
```

## Examples

```bash
cargo run --example demo                  # Widget showcase (4 tabs)
cargo run --example irc_demo              # IRC client demo
cargo run --example screen_stack          # Screen push/pop/modal demo
cargo run --example tutorial_01_hello     # Hello world
cargo run --example tutorial_02_layout    # Layout system
cargo run --example tutorial_03_events    # Event handling
cargo run --example tutorial_04_reactive  # Reactive state
cargo run --example tutorial_05_workers   # Background workers
cargo run --example tutorial_06_screens   # Screen stack and modals
```

## Documentation

- [User Guide](https://github.com/jabberwock/textual-rs/blob/master/docs/guide.md) -- complete walkthrough
- [CSS Reference](https://github.com/jabberwock/textual-rs/blob/master/docs/css-reference.md) -- all properties, selectors, theme variables

## License

MIT
