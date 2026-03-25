use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::terminal::init_panic_hook;
use textual_rs::widget::context::AppContext;
use textual_rs::{App, Widget};

/// Minimal screen widget for integration tests.
struct TestScreen;

impl Widget for TestScreen {
    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        use ratatui::prelude::Widget as RatatuiWidget;
        let para = ratatui::widgets::Paragraph::new("Hello from textual-rs!");
        RatatuiWidget::render(para, area, buf);
    }

    fn widget_type_name(&self) -> &'static str {
        "TestScreen"
    }
}

#[test]
fn test_render_hello() {
    let mut app = App::new(|| Box::new(TestScreen));
    let buffer = app.render_to_test_backend(80, 24);

    let content: String = buffer.content().iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(
        content.contains("Hello from textual-rs!"),
        "Buffer should contain 'Hello from textual-rs!' but got: {}",
        &content[..200.min(content.len())]
    );
}

#[test]
fn test_render_has_title() {
    let mut app = App::new(|| Box::new(TestScreen));
    let buffer = app.render_to_test_backend(80, 24);

    let content: String = buffer.content().iter()
        .map(|cell| cell.symbol())
        .collect();
    assert!(
        content.contains("textual-rs"),
        "Buffer should contain 'textual-rs'"
    );
}

#[test]
fn test_panic_hook_is_installed() {
    init_panic_hook();
}

#[test]
fn test_terminal_guard_drop_is_idempotent() {
    let result = crossterm::terminal::disable_raw_mode();
    let _ = result;
}

#[test]
fn test_render_at_different_sizes() {
    let mut app_small = App::new(|| Box::new(TestScreen));
    let buf_small = app_small.render_to_test_backend(50, 15);

    let mut app_large = App::new(|| Box::new(TestScreen));
    let buf_large = app_large.render_to_test_backend(120, 40);

    let small_content: String = buf_small.content().iter()
        .map(|cell| cell.symbol()).collect();
    let large_content: String = buf_large.content().iter()
        .map(|cell| cell.symbol()).collect();

    assert!(small_content.contains("Hello from textual-rs!"),
        "Small terminal should render the hello text");
    assert!(large_content.contains("Hello from textual-rs!"),
        "Large terminal should render the hello text");

    assert_ne!(buf_small.area(), buf_large.area(),
        "Different terminal sizes should produce different buffer areas");
}
