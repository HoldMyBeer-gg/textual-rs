use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use insta::assert_snapshot;
use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::testing::TestApp;
use textual_rs::testing::assertions::assert_buffer_lines;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::Widget;
use textual_rs::{Button, Checkbox, Label, Switch};
use textual_rs::widget::button::messages::Pressed as ButtonPressed;

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_label_default() {
    let test_app = TestApp::new(20, 3, || Box::new(Label::new("Hello")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_button_default() {
    let test_app = TestApp::new(20, 3, || Box::new(Button::new("OK")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_checkbox_checked() {
    let test_app = TestApp::new(20, 3, || Box::new(Checkbox::new("Option", true)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_switch_on() {
    let test_app = TestApp::new(20, 3, || Box::new(Switch::new(true)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

// ---------------------------------------------------------------------------
// Label render tests
// ---------------------------------------------------------------------------

#[test]
fn label_renders_text_at_origin() {
    let test_app = TestApp::new(20, 3, || Box::new(Label::new("Hello")));
    assert_buffer_lines(test_app.buffer(), &["Hello"]);
}

#[test]
fn label_truncates_long_text() {
    let test_app = TestApp::new(5, 1, || Box::new(Label::new("Hello World")));
    assert_buffer_lines(test_app.buffer(), &["Hello"]);
}

// ---------------------------------------------------------------------------
// Button render tests
// ---------------------------------------------------------------------------

#[test]
fn button_renders_label_in_row() {
    let test_app = TestApp::new(20, 1, || Box::new(Button::new("OK")));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("OK"),
        "button label 'OK' should appear in buffer row 0, got: {:?}",
        row.trim()
    );
}

// ---------------------------------------------------------------------------
// Checkbox render tests
// ---------------------------------------------------------------------------

#[test]
fn checkbox_renders_checked_indicator() {
    let test_app = TestApp::new(20, 3, || Box::new(Checkbox::new("Test", true)));
    assert_buffer_lines(test_app.buffer(), &["[X] Test"]);
}

#[test]
fn checkbox_renders_unchecked_indicator() {
    let test_app = TestApp::new(20, 3, || Box::new(Checkbox::new("Test", false)));
    assert_buffer_lines(test_app.buffer(), &["[ ] Test"]);
}

// ---------------------------------------------------------------------------
// Switch render tests
// ---------------------------------------------------------------------------

#[test]
fn switch_renders_on_indicator() {
    let test_app = TestApp::new(10, 3, || Box::new(Switch::new(true)));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    let trimmed = row.trim_end();
    assert!(
        trimmed.contains("━━━◉"),
        "switch ON should render '━━━◉', got: {:?}",
        trimmed
    );
}

#[test]
fn switch_renders_off_indicator() {
    let test_app = TestApp::new(10, 3, || Box::new(Switch::new(false)));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    let trimmed = row.trim_end();
    assert!(
        trimmed.contains("◉━━━"),
        "switch OFF should render '◉━━━', got: {:?}",
        trimmed
    );
}

// ---------------------------------------------------------------------------
// Button press message verification
// ---------------------------------------------------------------------------

/// Screen that wraps a Button child and captures Pressed messages bubbling up.
struct ButtonCaptureScreen;

impl Widget for ButtonCaptureScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "ButtonCaptureScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Button::new("Click Me"))]
    }
}

/// We verify Pressed was posted to the message queue before drain by injecting
/// the key event directly via handle_key_event and inspecting the queue.
#[tokio::test]
async fn button_press_enter_emits_pressed_message() {
    let mut test_app = TestApp::new(40, 10, || Box::new(ButtonCaptureScreen));

    // Tab to focus the button child
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Button should have focus after Tab"
    );

    // Inject Enter key event without draining the message queue,
    // so we can inspect what was posted before bubbling.
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Verify ButtonPressed is in the message queue
    let has_pressed = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<ButtonPressed>().is_some());
    assert!(
        has_pressed,
        "Expected ButtonPressed in message queue after Enter on focused Button"
    );
}

// ---------------------------------------------------------------------------
// Checkbox toggle interaction tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn checkbox_toggle_space_changes_state() {
    let mut test_app = TestApp::new(40, 10, || Box::new(Checkbox::new("Opt", false)));

    // Focus the checkbox
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(test_app.ctx().focused_widget.is_some(), "Checkbox should have focus");

    // Verify initial render shows unchecked
    assert_buffer_lines(test_app.buffer(), &["[ ] Opt"]);

    // Press Space to toggle
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Char(' ')).await;
    }

    // Verify checkbox is now checked
    assert_buffer_lines(test_app.buffer(), &["[X] Opt"]);
}

#[tokio::test]
async fn checkbox_toggle_enter_also_works() {
    let mut test_app = TestApp::new(40, 10, || Box::new(Checkbox::new("Go", false)));

    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Enter (also bound to "toggle")
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Enter).await;
    }

    assert_buffer_lines(test_app.buffer(), &["[X] Go"]);
}

// ---------------------------------------------------------------------------
// Switch toggle interaction tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn switch_toggle_enter_changes_state() {
    let mut test_app = TestApp::new(40, 10, || Box::new(Switch::new(false)));

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(test_app.ctx().focused_widget.is_some(), "Switch should have focus");

    // Verify initial render shows OFF indicator
    {
        let buf = test_app.buffer();
        let row: String = (0..buf.area.width)
            .map(|col| buf[(col, 0)].symbol().to_string())
            .collect();
        assert!(
            row.contains("◉━━━"),
            "Initial OFF state should show '◉━━━', got: {:?}",
            row.trim_end()
        );
    }

    // Press Enter to toggle from OFF to ON
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Enter).await;
    }

    // Verify switch is now ON
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("━━━◉"),
        "Switch ON indicator expected after toggle, got: {:?}",
        row.trim_end()
    );
}

#[tokio::test]
async fn switch_toggle_space_also_works() {
    let mut test_app = TestApp::new(40, 10, || Box::new(Switch::new(true)));

    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Space to toggle from ON to OFF
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Char(' ')).await;
    }

    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("◉━━━"),
        "Switch OFF indicator expected after toggle from ON, got: {:?}",
        row.trim_end()
    );
}
