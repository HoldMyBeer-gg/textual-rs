use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::testing::assertions::{assert_buffer_lines, assert_cell};
use textual_rs::testing::TestApp;
use textual_rs::widget::context::AppContext;
use textual_rs::Widget;

// ---------------------------------------------------------------------------
// Test widgets
// ---------------------------------------------------------------------------

/// A simple widget that renders its text label.
struct TestLabel {
    text: &'static str,
    focusable: bool,
}

impl TestLabel {
    fn new(text: &'static str, focusable: bool) -> Self {
        TestLabel { text, focusable }
    }
}

impl Widget for TestLabel {
    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.width > 0 && area.height > 0 {
            buf.set_string(area.x, area.y, self.text, ratatui::style::Style::default());
        }
    }
    fn widget_type_name(&self) -> &'static str {
        "TestLabel"
    }
    fn can_focus(&self) -> bool {
        self.focusable
    }
}

/// A screen with two focusable TestLabel children.
struct TwoLabelScreen;

impl Widget for TwoLabelScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "TwoLabelScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(TestLabel::new("Label1", true)),
            Box::new(TestLabel::new("Label2", true)),
        ]
    }
}

/// A screen that renders a fixed string for snapshot/buffer assertion testing.
struct HelloScreen;

impl Widget for HelloScreen {
    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "Hello Test",
            ratatui::style::Style::default(),
        );
    }
    fn widget_type_name(&self) -> &'static str {
        "HelloScreen"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_app_creates_headless_app() {
    // TestApp should mount the screen without a real terminal.
    let test_app = TestApp::new(40, 10, || Box::new(TwoLabelScreen));
    // Screen is mounted — screen_stack should have one entry.
    assert!(
        !test_app.ctx().screen_stack.is_empty(),
        "screen_stack should not be empty after mount"
    );
}

#[tokio::test]
async fn test_app_buffer_has_correct_dimensions() {
    let test_app = TestApp::new(40, 10, || Box::new(HelloScreen));
    let buf = test_app.buffer();
    assert_eq!(buf.area.width, 40);
    assert_eq!(buf.area.height, 10);
}

#[tokio::test]
async fn pilot_press_tab_advances_focus() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TwoLabelScreen));

    // No focus initially.
    assert!(
        test_app.ctx().focused_widget.is_none(),
        "No focus before first Tab"
    );

    let mut pilot = test_app.pilot();
    pilot.press(KeyCode::Tab).await;

    // Focus should now be set.
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Focus should be set after pressing Tab"
    );
}

#[tokio::test]
async fn pilot_press_tab_twice_changes_focus() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TwoLabelScreen));

    // First Tab — advance to first focusable.
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    let first_focused = test_app.ctx().focused_widget;
    assert!(first_focused.is_some(), "first Tab should set focus");

    // Second Tab — advance to next focusable.
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    let second_focused = test_app.ctx().focused_widget;

    assert_ne!(
        first_focused, second_focused,
        "second Tab should move focus to a different widget"
    );
}

#[tokio::test]
async fn settle_makes_assertions_deterministic() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TwoLabelScreen));

    // Press Tab and settle — state should be stable afterwards.
    let mut pilot = test_app.pilot();
    pilot.press(KeyCode::Tab).await;
    // settle() is called inside press(); calling it again is a no-op.
    pilot.settle().await;

    assert!(
        test_app.ctx().focused_widget.is_some(),
        "focus state should be stable after settle"
    );
}

#[tokio::test]
async fn assert_buffer_lines_matches_rendered_text() {
    let test_app = TestApp::new(20, 5, || Box::new(HelloScreen));
    // "Hello Test" should be rendered at row 0.
    assert_buffer_lines(test_app.buffer(), &["Hello Test"]);
}

#[tokio::test]
async fn assert_cell_matches_character() {
    let test_app = TestApp::new(20, 5, || Box::new(HelloScreen));
    // Cell (0, 0) should be 'H'.
    assert_cell(test_app.buffer(), 0, 0, "H");
}

#[tokio::test]
async fn pilot_type_text_processes_each_char() {
    // HelloScreen ignores input but we verify no panics occur.
    let mut test_app = TestApp::new(20, 5, || Box::new(HelloScreen));
    let mut pilot = test_app.pilot();
    pilot.type_text("abc").await;
    // If we reach here without panic, the test passes.
}

#[tokio::test]
async fn pilot_click_processes_mouse_event() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TwoLabelScreen));
    let mut pilot = test_app.pilot();
    // Click somewhere on the screen — should not panic.
    pilot.click(5, 0).await;
}
