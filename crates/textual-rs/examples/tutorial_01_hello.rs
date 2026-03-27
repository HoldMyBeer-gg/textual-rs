// Tutorial 01: Hello, textual-rs
//
// This is the simplest possible textual-rs application.
// It shows how to:
//   1. Define a Widget (the root "screen" widget)
//   2. Implement the required render() method
//   3. Create an App and run it
//
// Run with: cargo run --example tutorial_01_hello
// Quit with: q or Ctrl+C

// Import ratatui types used in the Widget trait's render() method.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

// Import the App (application runner) and Widget (trait for all UI nodes).
use textual_rs::{App, Widget};

// AppContext is passed to render() — it holds the widget arena, focus state, etc.
use textual_rs::widget::context::AppContext;

// ---------------------------------------------------------------------------
// Define our root screen widget.
//
// In textual-rs, every piece of UI is a Widget. The top-level widget is called
// a "screen" — it fills the entire terminal window.
// ---------------------------------------------------------------------------
struct HelloScreen;

// Every widget must implement the Widget trait.
impl Widget for HelloScreen {
    // widget_type_name() returns the CSS selector name for this widget type.
    // Used by the CSS engine to apply styles like: HelloScreen { background: blue; }
    fn widget_type_name(&self) -> &'static str {
        "HelloScreen"
    }

    // render() is called every frame to paint this widget into the terminal buffer.
    //
    // Parameters:
    //   ctx  - AppContext: access widget arena, focus state, computed styles
    //   area - Rect: the pixel-area (in terminal cells) this widget occupies
    //   buf  - Buffer: the ratatui buffer to draw into (set_string, set_style, etc.)
    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // Don't draw in zero-size areas (can happen during terminal resize).
        if area.width == 0 || area.height == 0 {
            return;
        }

        // The message we want to display.
        let msg = "Hello, textual-rs!";

        // Center the message horizontally.
        let x = if area.width as usize > msg.len() {
            area.x + (area.width - msg.len() as u16) / 2
        } else {
            area.x
        };

        // Center vertically.
        let y = area.y + area.height / 2;

        // Inherit the current style from the buffer cell at our position.
        // This picks up any background/foreground color set by the CSS engine.
        let style = buf
            .cell((area.x, area.y))
            .map(|c| c.style())
            .unwrap_or_default();

        // Write the string into the buffer.
        buf.set_string(x, y, msg, style);
    }
}

// ---------------------------------------------------------------------------
// main() — create the App and run it.
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    // App::new() takes a factory closure that creates the root screen widget.
    // The factory is called once when the app starts.
    let mut app = App::new(|| Box::new(HelloScreen));

    // app.run() takes over the terminal, starts the event loop, and blocks
    // until the user quits (q or Ctrl+C).
    app.run()
}
