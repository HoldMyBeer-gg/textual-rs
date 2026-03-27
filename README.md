# textual-rs

A Rust TUI framework inspired by [Python Textual](https://textual.textualize.io), delivering modern terminal interfaces with CSS styling, reactive state, and rich widgets.

## Features

- **22+ built-in widgets** -- Button, Input, TextArea, Checkbox, Switch, RadioSet, Select, ListView, DataTable, Tree, Tabs, Markdown, and more
- **CSS/TCSS styling** -- type/class/ID selectors, cascade, theme variables (`$primary`, `$accent-lighten-2`), 8 border styles including McGugan Boxes
- **Sub-cell rendering** -- half-block, eighth-block, quadrant, and braille characters for high-resolution TUI graphics
- **Reactive state** -- `Reactive<T>` properties trigger automatic re-renders
- **Mouse support** -- click, hover, scroll wheel, right-click context menus
- **Keyboard** -- key bindings, Tab focus cycling, Ctrl+C/X/V clipboard
- **7 built-in themes** -- textual-dark, textual-light, tokyo-night, nord, gruvbox, dracula, catppuccin (Ctrl+T to cycle)
- **Animation** -- smooth tween transitions on Switch toggle and Tab switching
- **Testing** -- `TestApp` headless harness, `Pilot` for simulated input, snapshot testing with insta
- **Cross-platform** -- Windows 10+, macOS, Linux via ratatui + crossterm

## Quick Start

```toml
[dependencies]
textual-rs = "0.2"
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
cargo run --example demo          # Widget showcase
cargo run --example irc_demo      # IRC client demo
cargo run --example tutorial_01_hello
```

## Documentation

- [User Guide](docs/guide.md) -- complete walkthrough
- [CSS Reference](docs/css-reference.md) -- all properties, selectors, theme variables
- [API Docs](https://docs.rs/textual-rs) -- rustdoc

## License

MIT
