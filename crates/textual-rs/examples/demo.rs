use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::widget::context::AppContext;
use textual_rs::{App, Widget};

/// Minimal screen widget for the default demo.
struct DemoScreen;

impl Widget for DemoScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Phase 1 skeleton — no content rendered in the demo screen itself.
        // The TUI shows a blank terminal; Phase 3 will add real widgets.
    }

    fn widget_type_name(&self) -> &'static str {
        "DemoScreen"
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(DemoScreen));
    app.run()
}
