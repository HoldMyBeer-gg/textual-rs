use insta::assert_snapshot;
use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::testing::TestApp;
use textual_rs::widget::context::AppContext;
use textual_rs::Widget;

// ---------------------------------------------------------------------------
// Test widgets for snapshot tests
// ---------------------------------------------------------------------------

struct GreetingWidget;

impl Widget for GreetingWidget {
    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, "Hello, World!", ratatui::style::Style::default());
    }
    fn widget_type_name(&self) -> &'static str {
        "GreetingWidget"
    }
}

struct EmptyWidget;

impl Widget for EmptyWidget {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "EmptyWidget"
    }
}

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_greeting_widget() {
    let test_app = TestApp::new(20, 3, || Box::new(GreetingWidget));
    // TestBackend implements Display — insta captures the rendered output.
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_empty_widget() {
    let test_app = TestApp::new(10, 3, || Box::new(EmptyWidget));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_different_sizes() {
    let small = TestApp::new(10, 2, || Box::new(GreetingWidget));
    let large = TestApp::new(40, 5, || Box::new(GreetingWidget));
    assert_snapshot!("small_terminal", format!("{}", small.backend()));
    assert_snapshot!("large_terminal", format!("{}", large.backend()));
}
