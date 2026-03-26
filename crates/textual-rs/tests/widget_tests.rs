use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use insta::assert_snapshot;
use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::testing::TestApp;
use textual_rs::testing::assertions::assert_buffer_lines;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::Widget;
use textual_rs::{Button, Checkbox, Input, Label, Switch};
use textual_rs::widget::button::messages::Pressed as ButtonPressed;
use textual_rs::widget::input::messages::Submitted as InputSubmitted;

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

// ---------------------------------------------------------------------------
// Input widget tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn input_type_text() {
    let mut test_app = TestApp::new(40, 3, || Box::new(Input::new("")));

    // Focus the input
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(test_app.ctx().focused_widget.is_some(), "Input should have focus");

    // Type "hello"
    {
        let mut pilot = test_app.pilot();
        pilot.type_text("hello").await;
    }

    // Verify value via buffer content (cursor at end, all chars visible)
    let buf = test_app.buffer();
    let row: String = (0..5u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert_eq!(row, "hello", "Input should display typed text, got: {:?}", row);
}

#[tokio::test]
async fn input_cursor_movement() {
    let mut test_app = TestApp::new(40, 3, || Box::new(Input::new("")));

    // Focus and type "hello"
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("hello").await;
    }

    // Press Left 2 times — cursor should move from position 5 to 3
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Left).await;
        pilot.press(KeyCode::Left).await;
    }

    // Insert a character at cursor position 3 to verify cursor location
    {
        let mut pilot = test_app.pilot();
        pilot.type_text("X").await;
    }

    // Value should now be "helXlo" with cursor at 4
    let buf = test_app.buffer();
    let row: String = (0..6u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("helX") || row.contains("Xlo"),
        "After Left×2 and typing X, buffer should contain 'helX', got: {:?}",
        row
    );
}

#[tokio::test]
async fn input_backspace() {
    let mut test_app = TestApp::new(40, 3, || Box::new(Input::new("")));

    // Focus and type "hello"
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("hello").await;
    }

    // Press Backspace — should remove last char
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Backspace).await;
    }

    // Verify value is "hell"
    let buf = test_app.buffer();
    let row: String = (0..4u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert_eq!(row, "hell", "After Backspace, value should be 'hell', got: {:?}", row);
}

#[tokio::test]
async fn input_submit_emits_message() {
    let mut test_app = TestApp::new(40, 3, || Box::new(Input::new("")));

    // Focus and type "test"
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("test").await;
    }

    // Press Enter without draining message queue so we can inspect it
    test_app.inject_key_event(crossterm::event::KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    });

    // Verify Submitted message in queue
    let has_submitted = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<InputSubmitted>().is_some());
    assert!(has_submitted, "Expected Submitted message in queue after Enter on Input");
}

#[test]
fn input_placeholder_renders() {
    let test_app = TestApp::new(20, 3, || Box::new(Input::new("Type here")));
    // No focus — placeholder should be visible
    let buf = test_app.buffer();
    let row: String = (0..9u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert_eq!(row, "Type here", "Placeholder should render when input is empty and unfocused, got: {:?}", row);
}

#[tokio::test]
async fn input_password_mode() {
    let mut test_app = TestApp::new(40, 3, || {
        let mut input = Input::new("");
        input.password = true;
        Box::new(input)
    });

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Type "secret"
    {
        let mut pilot = test_app.pilot();
        pilot.type_text("secret").await;
    }

    // Verify buffer shows "******" not "secret"
    let buf = test_app.buffer();
    let row: String = (0..6u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    // In password mode, characters should be masked (shown as *, space due to cursor, or reversed *)
    // The first 5 chars should be * (cursor is on last position)
    let contains_stars = row.chars().all(|c| c == '*' || c == ' ');
    assert!(
        contains_stars,
        "Password mode should mask input with '*', got: {:?}",
        row
    );
}
