// Tutorial 02: Layout — composing widgets with CSS styling
//
// This tutorial shows how to:
//   1. Use compose() to declare child widgets
//   2. Apply CSS styling via with_css()
//   3. Use flex layout with fixed-height Header/Footer
//   4. Use built-in container widgets (Header, Footer, Label)
//
// Run with: cargo run --example tutorial_02_layout
// Quit with: q or Ctrl+C

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use textual_rs::{App, Widget, Header, Footer, Label};
use textual_rs::widget::context::AppContext;

// ---------------------------------------------------------------------------
// Inline TCSS stylesheet.
//
// TCSS is textual-rs's CSS-like styling language. Selectors are widget type
// names. Properties control layout, colors, borders, and sizing.
//
// Key properties used here:
//   background: #rrggbb        — background color
//   color: #rrggbb             — foreground text color
//   height: N                  — fixed height in rows
//   layout-direction: vertical — stack children top-to-bottom (default)
//   flex-grow: 1               — expand to fill remaining space
// ---------------------------------------------------------------------------
const CSS: &str = r#"
LayoutScreen {
    background: $background;
    color: $foreground;
    layout-direction: vertical;
}
Header {
    height: 1;
    background: $panel;
    color: $primary;
}
Footer {
    height: 1;
    background: $panel;
    color: $text;
}
ContentArea {
    flex-grow: 1;
    layout-direction: vertical;
    padding: 1;
    border: tall $primary;
}
Label {
    height: 1;
    color: $foreground;
}
"#;

// ---------------------------------------------------------------------------
// ContentArea — a vertical container with two label children.
//
// compose() returns the list of child widgets. The framework mounts them
// into the widget tree and lays them out according to the CSS rules.
// ---------------------------------------------------------------------------
struct ContentArea;

impl Widget for ContentArea {
    fn widget_type_name(&self) -> &'static str {
        "ContentArea"
    }

    // compose() declares this widget's children.
    // The framework calls this once at mount time to build the widget tree.
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            // Built-in Label widget renders a single line of text.
            Box::new(Label::new("Welcome to textual-rs!")),
            Box::new(Label::new("This is the layout tutorial.")),
            Box::new(Label::new("")),
            Box::new(Label::new("compose() declares children.")),
            Box::new(Label::new("CSS controls size, color, and position.")),
            Box::new(Label::new("Tab cycles focus. q quits.")),
        ]
    }

    // Container widgets usually have an empty render() — children render themselves.
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---------------------------------------------------------------------------
// LayoutScreen — the root screen.
//
// It composes Header + ContentArea + Footer.
// Header (height:1) sits at top, Footer (height:1) at bottom.
// ContentArea fills the remaining space (flex-grow: 1).
// ---------------------------------------------------------------------------
struct LayoutScreen;

impl Widget for LayoutScreen {
    fn widget_type_name(&self) -> &'static str {
        "LayoutScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            // Header is a built-in widget. with_subtitle() adds a subtitle.
            Box::new(Header::new("Tutorial 02: Layout").with_subtitle("compose + CSS")),
            // ContentArea fills the middle space.
            Box::new(ContentArea),
            // Footer shows key bindings from the focused widget.
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    // with_css() parses and applies the inline TCSS stylesheet.
    // CSS errors are logged to stderr; the app still runs with defaults.
    let mut app = App::new(|| Box::new(LayoutScreen))
        .with_css(CSS);
    app.run()
}
