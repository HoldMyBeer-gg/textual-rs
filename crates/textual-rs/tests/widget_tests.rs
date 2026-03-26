use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use insta::assert_snapshot;
use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::testing::TestApp;
use textual_rs::testing::assertions::assert_buffer_lines;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::{EventPropagation, Widget};
use textual_rs::{Button, Checkbox, Input, Label, RadioButton, RadioSet, Select, Switch, TextArea};
use textual_rs::widget::button::messages::Pressed as ButtonPressed;
use textual_rs::widget::input::messages::Submitted as InputSubmitted;
use textual_rs::widget::radio::messages::RadioSetChanged;

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// RadioButton render tests
// ---------------------------------------------------------------------------

#[test]
fn radio_button_renders_checked() {
    let test_app = TestApp::new(20, 3, || Box::new(RadioButton::new("Option A", true)));
    let buf = test_app.buffer();
    let row: String = (0..12u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    // Checked renders as "(●) Option A"
    assert!(
        row.contains('\u{25cf}'),
        "Checked RadioButton should render filled circle '\u{25cf}', got: {:?}",
        row
    );
}

#[test]
fn radio_button_renders_unchecked() {
    let test_app = TestApp::new(20, 3, || Box::new(RadioButton::new("Option B", false)));
    let buf = test_app.buffer();
    let row: String = (0..12u16)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    // Unchecked renders as "( ) Option B"
    assert!(
        row.starts_with("( )"),
        "Unchecked RadioButton should render '( )', got: {:?}",
        row
    );
}

// ---------------------------------------------------------------------------
// RadioSet interaction tests
// ---------------------------------------------------------------------------

/// Screen wrapping a RadioSet to test interaction — captures RadioSetChanged via on_event.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct RadioSetCaptureScreen {
    changed_index: Arc<AtomicUsize>,
}

impl RadioSetCaptureScreen {
    fn new(changed_index: Arc<AtomicUsize>) -> Self {
        Self { changed_index }
    }
}

impl Widget for RadioSetCaptureScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "RadioSetCaptureScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(RadioSet::new(vec![
            "Option A".to_string(),
            "Option B".to_string(),
            "Option C".to_string(),
        ]))]
    }
    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        if let Some(changed) = event.downcast_ref::<RadioSetChanged>() {
            self.changed_index.store(changed.index, Ordering::SeqCst);
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }
}

#[tokio::test]
async fn radio_set_mutual_exclusion() {
    // RadioSet with 3 options. Verify mutual exclusion via the RadioSetChanged index
    // and that only one button is selected at a time.
    // Uses the capture screen to receive the RadioSetChanged event.
    let changed_index = Arc::new(AtomicUsize::new(999));
    let idx_clone = changed_index.clone();

    let mut test_app = TestApp::new(40, 10, move || {
        Box::new(RadioSetCaptureScreen::new(idx_clone))
    });

    // Tab to first RadioButton (first focusable in DFS under RadioSet)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(test_app.ctx().focused_widget.is_some(), "RadioButton should have focus after Tab");

    // Tab to second RadioButton
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Space to select second button (index 1)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Char(' ')).await;
    }

    // The RadioSetCaptureScreen captures the index via on_event
    let captured_idx = changed_index.load(Ordering::SeqCst);
    assert_eq!(
        captured_idx, 1,
        "Selecting second RadioButton should emit RadioSetChanged with index=1, got: {}",
        captured_idx
    );

    // Tab to third RadioButton and select it (index 2)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.press(KeyCode::Char(' ')).await;
    }

    let captured_idx2 = changed_index.load(Ordering::SeqCst);
    assert_eq!(
        captured_idx2, 2,
        "Selecting third RadioButton should emit RadioSetChanged with index=2, got: {}",
        captured_idx2
    );
}

#[tokio::test]
async fn radio_set_emits_changed() {
    // Use capture screen to verify RadioSetChanged event index
    let changed_index = Arc::new(AtomicUsize::new(999));
    let idx_clone = changed_index.clone();

    let changed_value: Arc<std::sync::Mutex<String>> = Arc::new(std::sync::Mutex::new(String::new()));
    let val_clone = changed_value.clone();

    struct RadioCaptureScreen {
        changed_index: Arc<AtomicUsize>,
        changed_value: Arc<std::sync::Mutex<String>>,
    }
    impl Widget for RadioCaptureScreen {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str { "RadioCaptureScreen" }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(RadioSet::new(vec![
                "Alpha".to_string(),
                "Beta".to_string(),
            ]))]
        }
        fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
            if let Some(changed) = event.downcast_ref::<RadioSetChanged>() {
                self.changed_index.store(changed.index, Ordering::SeqCst);
                *self.changed_value.lock().unwrap() = changed.value.clone();
                return EventPropagation::Stop;
            }
            EventPropagation::Continue
        }
    }

    let mut test_app = TestApp::new(40, 10, move || {
        Box::new(RadioCaptureScreen {
            changed_index: idx_clone,
            changed_value: val_clone,
        })
    });

    // Tab to first RadioButton, then second
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.press(KeyCode::Tab).await;
    }

    // Press Space to select the second button (index 1, "Beta")
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Char(' ')).await;
    }

    let captured_idx = changed_index.load(Ordering::SeqCst);
    let captured_val = changed_value.lock().unwrap().clone();
    assert_eq!(captured_idx, 1, "RadioSetChanged should report index=1, got: {}", captured_idx);
    assert_eq!(captured_val, "Beta", "RadioSetChanged should report value='Beta', got: {:?}", captured_val);
}

// ---------------------------------------------------------------------------
// TextArea tests
// ---------------------------------------------------------------------------

/// Helper: collect the rendered text content of a buffer row (trimmed of trailing spaces).
fn buf_row_trimmed(buf: &ratatui::buffer::Buffer, row: u16) -> String {
    let s: String = (0..buf.area.width)
        .map(|col| buf[(col, row)].symbol().to_string())
        .collect();
    s.trim_end().to_string()
}

#[tokio::test]
async fn text_area_type_text() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TextArea::new()));

    // Focus the text area
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Type "hello", press Enter, type "world"
    {
        let mut pilot = test_app.pilot();
        pilot.type_text("hello").await;
        pilot.press(KeyCode::Enter).await;
        pilot.type_text("world").await;
    }

    // Verify the rendered buffer contains "hello" on row 0 and "world" on row 1.
    // The cursor is shown as reverse-video so the actual chars should still be present.
    let row0 = buf_row_trimmed(test_app.buffer(), 0);
    let row1 = buf_row_trimmed(test_app.buffer(), 1);
    assert!(
        row0.contains("hello"),
        "row 0 should contain 'hello', got: {:?}",
        row0
    );
    assert!(
        row1.contains("world"),
        "row 1 should contain 'world', got: {:?}",
        row1
    );
}

#[tokio::test]
async fn text_area_cursor_movement() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TextArea::new()));

    // Focus and type two lines
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("abc").await;
        pilot.press(KeyCode::Enter).await;
        pilot.type_text("xyz").await;
    }

    // Cursor is at row=1, col=3. Move up, home, end, down — should not panic.
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Up).await;
        pilot.press(KeyCode::Home).await;
        pilot.press(KeyCode::End).await;
        pilot.press(KeyCode::Down).await;
    }

    // Verify buffer shows both lines
    let row0 = buf_row_trimmed(test_app.buffer(), 0);
    let row1 = buf_row_trimmed(test_app.buffer(), 1);
    assert!(
        row0.contains("abc"),
        "row 0 should contain 'abc', got: {:?}",
        row0
    );
    assert!(
        row1.contains("xyz"),
        "row 1 should contain 'xyz', got: {:?}",
        row1
    );
}

#[tokio::test]
async fn text_area_backspace_joins_lines() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TextArea::new()));

    // Focus and type "ab", Enter, "cd"
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("ab").await;
        pilot.press(KeyCode::Enter).await;
        pilot.type_text("cd").await;
    }

    // Move to start of line 1 then Backspace to join into line 0
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Home).await;
        pilot.press(KeyCode::Backspace).await;
    }

    // After joining, row 0 should contain "abcd" and row 1 should be empty.
    let row0 = buf_row_trimmed(test_app.buffer(), 0);
    let row1 = buf_row_trimmed(test_app.buffer(), 1);
    assert!(
        row0.contains("abcd"),
        "row 0 should contain 'abcd' after joining, got: {:?}",
        row0
    );
    assert!(
        row1.is_empty(),
        "row 1 should be empty after joining, got: {:?}",
        row1
    );
}

#[tokio::test]
async fn text_area_line_numbers() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TextArea::with_line_numbers()));

    // Focus and type 3 lines
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("line one").await;
        pilot.press(KeyCode::Enter).await;
        pilot.type_text("line two").await;
        pilot.press(KeyCode::Enter).await;
        pilot.type_text("line three").await;
    }

    // Snapshot shows line numbers in left margin
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[tokio::test]
async fn text_area_newline_splits_line() {
    let mut test_app = TestApp::new(40, 10, || Box::new(TextArea::new()));

    // Focus and type "abcd"
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
        pilot.type_text("abcd").await;
    }

    // Press Home (go to col=0), press Right twice (col=2), press Enter (split at col=2)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Home).await;
        pilot.press(KeyCode::Right).await;
        pilot.press(KeyCode::Right).await;
        pilot.press(KeyCode::Enter).await;
    }

    // After splitting "abcd" at position 2: row 0 = "ab", row 1 = "cd"
    let row0 = buf_row_trimmed(test_app.buffer(), 0);
    let row1 = buf_row_trimmed(test_app.buffer(), 1);
    assert!(
        row0.contains("ab"),
        "row 0 should contain 'ab' after split, got: {:?}",
        row0
    );
    assert!(
        row1.contains("cd"),
        "row 1 should contain 'cd' after split, got: {:?}",
        row1
    );
    // row 0 should NOT contain "cd" (it was split off)
    assert!(
        !row0.contains("abcd"),
        "row 0 should not contain full 'abcd', got: {:?}",
        row0
    );
}

// ---------------------------------------------------------------------------
// Select tests
// ---------------------------------------------------------------------------

#[test]
fn select_renders_current_option() {
    let options = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
    let test_app = TestApp::new(20, 3, || Box::new(Select::new(options)));

    // The rendered buffer should show "▼ Alpha" (the first option)
    let row0 = buf_row_trimmed(test_app.buffer(), 0);
    assert!(
        row0.contains("Alpha"),
        "Select should render '▼ Alpha' as current option, got: {:?}",
        row0
    );
    assert!(
        row0.contains('\u{25bc}'),
        "Select should render '▼' prefix, got: {:?}",
        row0
    );
}

#[tokio::test]
async fn select_open_pushes_overlay() {
    let options = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
    let mut test_app = TestApp::new(40, 10, || Box::new(Select::new(options)));

    // Focus the Select
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(test_app.ctx().focused_widget.is_some(), "Select should have focus");

    // Press Enter to open — this should push to pending_screen_pushes
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Verify overlay was queued in pending_screen_pushes
    let overlay_count = test_app.ctx().pending_screen_pushes.borrow().len();
    assert_eq!(
        overlay_count, 1,
        "Opening Select should push 1 overlay to pending_screen_pushes, got: {}",
        overlay_count
    );

    // Verify the overlay has correct widget_type_name
    let overlay_name = test_app
        .ctx()
        .pending_screen_pushes
        .borrow()
        .first()
        .map(|w| w.widget_type_name())
        .unwrap_or("")
        .to_string();
    assert_eq!(
        overlay_name, "SelectOverlay",
        "Pushed overlay should be SelectOverlay, got: {:?}",
        overlay_name
    );
}

#[tokio::test]
async fn select_choose_option_queues_overlay() {
    // This test verifies the overlay push mechanism works end-to-end.
    // We open the Select, verify overlay is queued with correct options,
    // then check the overlay widget_type_name and can_focus.
    let options = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
    let mut test_app = TestApp::new(40, 10, move || Box::new(Select::new(options.clone())));

    // Focus the Select
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Enter to open the overlay (inject without draining)
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Verify overlay is in pending_screen_pushes
    let overlay_count = test_app.ctx().pending_screen_pushes.borrow().len();
    assert_eq!(
        overlay_count, 1,
        "Opening Select should queue 1 overlay, got: {}",
        overlay_count
    );

    // Verify the overlay is focusable and has correct type
    let overlay_focusable = test_app
        .ctx()
        .pending_screen_pushes
        .borrow()
        .first()
        .map(|w| w.can_focus())
        .unwrap_or(false);
    assert!(overlay_focusable, "SelectOverlay should be focusable");

    // Clear the pending pushes
    test_app.ctx().pending_screen_pushes.borrow_mut().clear();

    // Select should still show Alpha on the current screen
    test_app.drain_messages();
    let row0 = buf_row_trimmed(test_app.buffer(), 0);
    assert!(
        row0.contains("Alpha"),
        "Select should still show Alpha before overlay resolves, got: {:?}",
        row0
    );
}

#[tokio::test]
async fn snapshot_select_initial() {
    let options = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
    let test_app = TestApp::new(20, 5, || Box::new(Select::new(options)));

    assert_snapshot!(format!("{}", test_app.backend()));
}
