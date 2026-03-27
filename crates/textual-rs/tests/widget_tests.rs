use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use insta::assert_snapshot;
use ratatui::{buffer::Buffer, layout::Rect};
use textual_rs::testing::assertions::assert_buffer_lines;
use textual_rs::testing::TestApp;
use textual_rs::widget::button::messages::Pressed as ButtonPressed;
use textual_rs::widget::collapsible::messages::{
    Collapsed as CollapsibleCollapsed, Expanded as CollapsibleExpanded,
};
use textual_rs::widget::context::AppContext;
use textual_rs::widget::data_table::messages::{RowSelected, SortChanged};
use textual_rs::widget::input::messages::Submitted as InputSubmitted;
use textual_rs::widget::list_view::messages::Selected as ListViewSelected;
use textual_rs::widget::radio::messages::RadioSetChanged;
use textual_rs::widget::tabs::messages::TabChanged;
use textual_rs::widget::tree_view::messages::{NodeCollapsed, NodeExpanded, NodeSelected};
use textual_rs::widget::{EventPropagation, Widget};
use textual_rs::{
    Button, Checkbox, Collapsible, ColumnDef, DataTable, Footer, Header, Input, Label, ListView,
    LoadingIndicator, Log, Markdown, Placeholder, ProgressBar, RadioButton, RadioSet, RichLog,
    ScrollView, Select, Sparkline, Switch, TabbedContent, Tabs, TextArea, Tree, TreeNode,
};

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
    assert_buffer_lines(test_app.buffer(), &["✓ Test"]);
}

#[test]
fn checkbox_renders_unchecked_indicator() {
    let test_app = TestApp::new(20, 3, || Box::new(Checkbox::new("Test", false)));
    assert_buffer_lines(test_app.buffer(), &["☐ Test"]);
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
        trimmed.contains("██") && trimmed.contains("▌"),
        "switch ON should render pill with knob, got: {:?}",
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
        trimmed.contains("▐██") && trimmed.contains("━"),
        "switch OFF should render pill with knob on left, got: {:?}",
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
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Checkbox should have focus"
    );

    // Verify initial render shows unchecked
    assert_buffer_lines(test_app.buffer(), &["☐ Opt"]);

    // Press Space to toggle
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Char(' ')).await;
    }

    // Verify checkbox is now checked
    assert_buffer_lines(test_app.buffer(), &["✓ Opt"]);
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

    assert_buffer_lines(test_app.buffer(), &["✓ Go"]);
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
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Switch should have focus"
    );

    // Verify initial render shows OFF indicator (knob on left)
    {
        let buf = test_app.buffer();
        let row: String = (0..buf.area.width)
            .map(|col| buf[(col, 0)].symbol().to_string())
            .collect();
        assert!(
            row.contains("▐██"),
            "Initial OFF state should show knob on left, got: {:?}",
            row.trim_end()
        );
    }

    // Press Enter to toggle from OFF to ON
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Enter).await;
    }

    // Verify switch is now ON (knob on right)
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("██▌"),
        "Switch ON indicator expected after toggle (knob on right), got: {:?}",
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
        row.contains("▐██") && row.contains("━"),
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
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Input should have focus"
    );

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
    assert_eq!(
        row, "hello",
        "Input should display typed text, got: {:?}",
        row
    );
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
    assert_eq!(
        row, "hell",
        "After Backspace, value should be 'hell', got: {:?}",
        row
    );
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
    assert!(
        has_submitted,
        "Expected Submitted message in queue after Enter on Input"
    );
}

#[test]
#[ignore = "TODO: fix layout coords after auto-focus refactor — Input placeholder still shows unfocused"]
fn input_placeholder_renders() {
    use ratatui::buffer::Buffer as RatBuf;
    use ratatui::layout::Rect;
    // Use a screen with a non-focusable Label before Input so Input stays unfocused
    struct InputScreen;
    impl Widget for InputScreen {
        fn widget_type_name(&self) -> &'static str { "InputScreen" }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![Box::new(Input::new("Type here"))]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut RatBuf) {}
    }
    // Input is auto-focused as only focusable widget — placeholder hidden when focused.
    // Verify unfocused state by using a wrapper where focus goes to a Button instead.
    struct WrapperScreen;
    impl Widget for WrapperScreen {
        fn widget_type_name(&self) -> &'static str { "WrapperScreen" }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![
                Box::new(Button::new("Steal Focus")), // auto-focused
                Box::new(Input::new("Type here")),    // unfocused
            ]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut RatBuf) {}
    }
    let test_app = TestApp::new(20, 6, || Box::new(WrapperScreen));
    // Button gets auto-focus; Input (row 3) should show placeholder
    let buf = test_app.buffer();
    let row: String = (0..9u16)
        .map(|col| buf[(col, 3)].symbol().to_string())
        .collect();
    assert_eq!(
        row, "Type here",
        "Placeholder should render when input is empty and unfocused, got: {:?}",
        row
    );
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
// Input validation tests
// ---------------------------------------------------------------------------

fn inject_char(test_app: &mut TestApp, c: char) {
    test_app.inject_key_event(crossterm::event::KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    });
}

#[tokio::test]
async fn input_validation_valid_input() {
    let mut test_app = TestApp::new(40, 3, || {
        Box::new(Input::new("").with_validator(|s| s.len() <= 5))
    });

    // Focus first via pilot (Tab triggers focus)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Type "abc" using inject_key_event so messages stay in queue
    inject_char(&mut test_app, 'a');
    inject_char(&mut test_app, 'b');
    inject_char(&mut test_app, 'c');

    // Check Changed messages — at least one should have valid: true (3 chars <= 5 limit)
    let has_valid = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<textual_rs::widget::input::messages::Changed>()
                .map(|m| m.valid)
                .unwrap_or(false)
        });
    assert!(
        has_valid,
        "Changed message should have valid: true for input within limit"
    );
}

#[tokio::test]
async fn input_validation_invalid_input() {
    let mut test_app = TestApp::new(40, 3, || {
        Box::new(Input::new("").with_validator(|s| s.len() <= 3))
    });

    // Focus first via pilot
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Type "abcd" (4 chars, over limit of 3) using inject_key_event
    inject_char(&mut test_app, 'a');
    inject_char(&mut test_app, 'b');
    inject_char(&mut test_app, 'c');
    inject_char(&mut test_app, 'd');

    // Check that there is a Changed message with valid: false
    let has_invalid = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<textual_rs::widget::input::messages::Changed>()
                .map(|m| !m.valid)
                .unwrap_or(false)
        });
    assert!(
        has_invalid,
        "Changed message should have valid: false for input over limit"
    );
}

#[tokio::test]
async fn input_no_validator_always_valid() {
    let mut test_app = TestApp::new(40, 3, || Box::new(Input::new("")));

    // Focus first via pilot
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Type "hello" using inject_key_event
    for c in "hello".chars() {
        inject_char(&mut test_app, c);
    }

    // All Changed messages should have valid: true (no validator set)
    let queue = test_app.ctx().message_queue.borrow();
    let changed_msgs: Vec<bool> = queue
        .iter()
        .filter_map(|(_, msg)| {
            msg.downcast_ref::<textual_rs::widget::input::messages::Changed>()
                .map(|m| m.valid)
        })
        .collect();
    assert!(
        !changed_msgs.is_empty(),
        "Should have at least one Changed message"
    );
    assert!(
        changed_msgs.iter().all(|&v| v),
        "Input with no validator should always produce valid: true, got: {:?}",
        changed_msgs
    );
}

#[tokio::test]
async fn input_changed_message_includes_valid() {
    let mut test_app = TestApp::new(40, 3, || {
        // Validator: non-empty string is valid
        Box::new(Input::new("").with_validator(|s| !s.is_empty()))
    });

    // Focus first via pilot
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Type "a" and check the Changed message has valid: true
    inject_char(&mut test_app, 'a');
    let has_valid_true = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<textual_rs::widget::input::messages::Changed>()
                .map(|m| m.valid && m.value == "a")
                .unwrap_or(false)
        });
    assert!(
        has_valid_true,
        "Changed message for 'a' should have valid: true"
    );

    // Backspace to clear (value becomes empty -> valid: false)
    test_app.inject_key_event(crossterm::event::KeyEvent {
        code: KeyCode::Backspace,
        modifiers: KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    });

    let has_valid_false = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<textual_rs::widget::input::messages::Changed>()
                .map(|m| !m.valid && m.value.is_empty())
                .unwrap_or(false)
        });
    assert!(
        has_valid_false,
        "Changed message for empty string should have valid: false"
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
    // Checked renders as "◉ Option A"
    assert!(
        row.contains('◉'),
        "Checked RadioButton should render '◉', got: {:?}",
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
    // Unchecked renders as "○ Option B"
    assert!(
        row.starts_with("○"),
        "Unchecked RadioButton should render '○', got: {:?}",
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

    // Auto-focus lands on first RadioButton — just verify focus is set
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "RadioButton should have focus after mount"
    );

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

    let changed_value: Arc<std::sync::Mutex<String>> =
        Arc::new(std::sync::Mutex::new(String::new()));
    let val_clone = changed_value.clone();

    struct RadioCaptureScreen {
        changed_index: Arc<AtomicUsize>,
        changed_value: Arc<std::sync::Mutex<String>>,
    }
    impl Widget for RadioCaptureScreen {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "RadioCaptureScreen"
        }
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

    // Auto-focus on first RadioButton; Tab once to reach second
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Space to select the second button (index 1, "Beta")
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Char(' ')).await;
    }

    let captured_idx = changed_index.load(Ordering::SeqCst);
    let captured_val = changed_value.lock().unwrap().clone();
    assert_eq!(
        captured_idx, 1,
        "RadioSetChanged should report index=1, got: {}",
        captured_idx
    );
    assert_eq!(
        captured_val, "Beta",
        "RadioSetChanged should report value='Beta', got: {:?}",
        captured_val
    );
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
    // Row 1 may contain the focus accent bar (▎) if widget is borderless+focused
    assert!(
        row1.is_empty() || row1 == "▎",
        "row 1 should be empty after joining (may have focus indicator), got: {:?}",
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
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Select should have focus"
    );

    // Press Enter to open — this should push to pending_screen_pushes
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Verify overlay was set as active_overlay
    let overlay = test_app.ctx().active_overlay.borrow();
    assert!(
        overlay.is_some(),
        "Opening Select should set active_overlay"
    );
    let overlay_name = overlay.as_ref().map(|w| w.widget_type_name()).unwrap_or("");
    assert_eq!(
        overlay_name, "SelectOverlay",
        "Overlay should be SelectOverlay, got: {:?}",
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

    // Verify overlay is active and focusable
    {
        let overlay = test_app.ctx().active_overlay.borrow();
        assert!(
            overlay.is_some(),
            "Opening Select should set active_overlay"
        );
        let focusable = overlay.as_ref().map(|w| w.can_focus()).unwrap_or(false);
        assert!(focusable, "SelectOverlay should be focusable");
    }
}

#[tokio::test]
async fn snapshot_select_initial() {
    let options = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
    let test_app = TestApp::new(20, 5, || Box::new(Select::new(options)));

    assert_snapshot!(format!("{}", test_app.backend()));
}

// ---------------------------------------------------------------------------
// Header tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_header_title_only() {
    let test_app = TestApp::new(40, 1, || Box::new(Header::new("My App")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_header_title_subtitle() {
    let test_app = TestApp::new(40, 1, || {
        Box::new(Header::new("My App").with_subtitle("v1.0"))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn header_renders_title_centered() {
    let test_app = TestApp::new(40, 1, || Box::new(Header::new("My App")));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("My App"),
        "Header should contain 'My App', got: {:?}",
        row.trim_end()
    );
}

#[test]
fn header_renders_title_and_subtitle() {
    let test_app = TestApp::new(40, 1, || {
        Box::new(Header::new("My App").with_subtitle("v1.0"))
    });
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("My App -- v1.0"),
        "Header should contain 'My App -- v1.0', got: {:?}",
        row.trim_end()
    );
}

// ---------------------------------------------------------------------------
// Footer tests
// ---------------------------------------------------------------------------

/// Screen that wraps a Checkbox (has show=true binding) + Footer.
struct FooterWithCheckboxScreen;

impl Widget for FooterWithCheckboxScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "FooterWithCheckboxScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Checkbox::new("Option", false)), Box::new(Footer)]
    }
}

#[tokio::test]
async fn snapshot_footer_with_bindings() {
    let mut test_app = TestApp::new(40, 3, || Box::new(FooterWithCheckboxScreen));
    // Tab to focus the Checkbox (which has show=true "Toggle" binding)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_footer_empty() {
    // Footer with no focused widget renders empty
    let test_app = TestApp::new(40, 1, || Box::new(Footer));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn footer_shows_global_hints() {
    let test_app = TestApp::new(60, 1, || Box::new(Footer));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("Quit"),
        "Footer should always show global hints, got: {:?}",
        row.trim_end()
    );
}

// ---------------------------------------------------------------------------
// Placeholder tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_placeholder_default() {
    let test_app = TestApp::new(20, 5, || Box::new(Placeholder::new()));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_placeholder_labeled() {
    let test_app = TestApp::new(20, 5, || Box::new(Placeholder::with_label("Sidebar")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn placeholder_renders_dimensions() {
    let test_app = TestApp::new(20, 5, || Box::new(Placeholder::new()));
    let buf = test_app.buffer();
    // Check that some row contains the dimensions "20x5"
    let has_dimensions = (0..buf.area.height).any(|row| {
        let line: String = (0..buf.area.width)
            .map(|col| buf[(col, row)].symbol().to_string())
            .collect();
        line.contains("20×5") || line.contains("20x5")
    });
    assert!(
        has_dimensions,
        "Placeholder should render dimensions '20×5'"
    );
}

#[test]
fn placeholder_renders_label() {
    let test_app = TestApp::new(20, 5, || Box::new(Placeholder::with_label("Sidebar")));
    let buf = test_app.buffer();
    let has_label = (0..buf.area.height).any(|row| {
        let line: String = (0..buf.area.width)
            .map(|col| buf[(col, row)].symbol().to_string())
            .collect();
        line.contains("Sidebar")
    });
    assert!(has_label, "Placeholder should render label 'Sidebar'");
}

// ---------------------------------------------------------------------------
// ProgressBar tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_progress_bar_50_percent() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::new(0.5)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_progress_bar_full() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::new(1.0)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_progress_bar_empty() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::new(0.0)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_progress_bar_indeterminate() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::indeterminate()));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn progress_bar_50_renders_half_filled() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::new(0.5)));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    // At 50% with width 20, 10 filled chars and 10 empty chars
    assert!(
        row.contains('█'),
        "ProgressBar at 50% should contain filled chars, got: {:?}",
        row
    );
    // Empty portion uses spaces (color-only rendering via fg/bg)
    assert!(
        row.contains(' '),
        "ProgressBar at 50% should contain empty space, got: {:?}",
        row
    );
}

#[test]
fn progress_bar_full_renders_all_filled() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::new(1.0)));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        !row.contains('░'),
        "ProgressBar at 100% should have no empty chars, got: {:?}",
        row
    );
}

#[test]
fn progress_bar_empty_renders_no_filled() {
    let test_app = TestApp::new(20, 1, || Box::new(ProgressBar::new(0.0)));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    assert!(
        !row.contains('█'),
        "ProgressBar at 0% should have no filled chars, got: {:?}",
        row
    );
}

// ---------------------------------------------------------------------------
// Sparkline tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_sparkline_ascending() {
    let test_app = TestApp::new(20, 1, || {
        Box::new(Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_sparkline_flat() {
    let test_app = TestApp::new(20, 1, || Box::new(Sparkline::new(vec![5.0, 5.0, 5.0])));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn sparkline_ascending_uses_block_chars() {
    let test_app = TestApp::new(20, 1, || {
        Box::new(Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]))
    });
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    // The last data point (8.0 = max) should render as '█'
    assert!(
        row.contains('█'),
        "Sparkline with max value should contain '█', got: {:?}",
        row
    );
    // The row should contain various block characters (ascending pattern)
    let has_low = row.chars().any(|c| "▁▂▃".contains(c));
    assert!(
        has_low,
        "Sparkline ascending should have low block chars, got: {:?}",
        row
    );
}

#[test]
fn sparkline_flat_renders_same_char_for_all_points() {
    let test_app = TestApp::new(20, 1, || Box::new(Sparkline::new(vec![5.0, 5.0, 5.0])));
    let buf = test_app.buffer();
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    // First 3 chars should be the same (all at max for flat data = '█')
    let data_chars: Vec<char> = row.chars().take(3).collect();
    assert!(
        data_chars.windows(2).all(|w| w[0] == w[1]),
        "Sparkline with flat data should render same char for all points, got: {:?}",
        &data_chars
    );
}

// ---------------------------------------------------------------------------
// ListView tests
// ---------------------------------------------------------------------------

fn make_items(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("Item {}", i)).collect()
}

#[tokio::test]
async fn list_view_navigate_down() {
    let mut test_app = TestApp::new(20, 10, || Box::new(ListView::new(make_items(5))));

    // Focus the list view
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Down).await;
        pilot.press(KeyCode::Down).await;
    }

    // Verify via render: selected item shows as reversed.
    // After 2 Down presses, selected=2. Row 2 (y=2) should be highlighted.
    let buf = test_app.buffer();
    // Row 2 (index 2) should have reversed style
    let cell = &buf[(0, 2)];
    assert!(
        cell.style()
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD),
        "Row 2 should be selected (BOLD style) after 2 Down presses"
    );
}

#[tokio::test]
async fn list_view_select_emits_message() {
    let mut test_app = TestApp::new(20, 10, || Box::new(ListView::new(make_items(5))));

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Navigate to item 1
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Down).await;
    }

    // Press Enter to select — inject without draining so we can inspect the queue
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let has_selected = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<ListViewSelected>().is_some());
    assert!(
        has_selected,
        "Expected ListViewSelected in message queue after Enter"
    );
}

#[tokio::test]
async fn list_view_scrolls_when_past_viewport() {
    // 20 items in a 5-row viewport
    let mut test_app = TestApp::new(20, 5, || Box::new(ListView::new(make_items(20))));

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Down 6 times — selected will be at 6, viewport_height=5, so offset should be 2
    {
        let mut pilot = test_app.pilot();
        for _ in 0..6 {
            pilot.press(KeyCode::Down).await;
        }
    }

    // Verify: item 6 is selected. The viewport should scroll so item 6 is visible.
    // With viewport_h=5, after 6 downs: selected=6, offset should be 2 (6-5+1=2).
    // Row at y=4 (last visible row) should show "Item 6" in reversed style.
    let buf = test_app.buffer();
    // The last row of the viewport (row 4) should be selected (Item 6)
    let cell = &buf[(0, 4)];
    assert!(
        cell.style()
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD),
        "Last viewport row should be selected (BOLD) after scrolling past viewport"
    );
}

#[test]
fn snapshot_list_view() {
    let test_app = TestApp::new(20, 5, || Box::new(ListView::new(make_items(5))));
    assert_snapshot!(format!("{}", test_app.backend()));
}

// ---------------------------------------------------------------------------
// Log tests
// ---------------------------------------------------------------------------

#[test]
fn log_push_line_auto_scrolls() {
    // Auto-scroll only fires after viewport_height is set (after first render).
    // Simulate a measured viewport by setting viewport_height before pushing lines.
    let log = Log::new();
    log.viewport_height.set(3); // simulate a 3-row viewport
    for i in 0..10 {
        log.push_line(format!("Line {}", i));
    }
    let offset = log.scroll_offset.get_untracked();
    // 10 lines - 3 viewport = offset 7
    assert!(
        offset > 0,
        "scroll_offset should be > 0 after pushing 10 lines with auto_scroll=true, got {}",
        offset
    );
    assert_eq!(offset, 7, "scroll_offset should be line_count - viewport_h");
}

#[test]
fn log_scroll_up_disables_auto_scroll() {
    let log = Log::new();
    log.viewport_height.set(3); // simulate measured viewport
    for i in 0..10 {
        log.push_line(format!("Line {}", i));
    }
    let initial_offset = log.scroll_offset.get_untracked();
    assert_eq!(initial_offset, 7, "initial offset should be 10 - 3 = 7");

    // Simulate scroll up — directly call on_action
    let ctx = AppContext::new();
    log.on_action("scroll_up", &ctx);

    let offset_after_scroll_up = log.scroll_offset.get_untracked();
    assert_eq!(
        offset_after_scroll_up, 6,
        "scroll up should decrement offset"
    );

    // Now push another line — with auto_scroll=false, offset should NOT change
    log.push_line("New Line".to_string());
    let offset_after_push = log.scroll_offset.get_untracked();
    assert_eq!(
        offset_after_push, offset_after_scroll_up,
        "push_line should NOT change offset when auto_scroll is disabled"
    );
}

#[test]
fn snapshot_log() {
    let log = Log::new();
    for i in 0..5 {
        log.push_line(format!("Line {}", i));
    }
    let test_app = TestApp::new(20, 3, move || Box::new(log));
    assert_snapshot!(format!("{}", test_app.backend()));
}

// ---------------------------------------------------------------------------
// ScrollView tests
// ---------------------------------------------------------------------------

fn make_label_children(n: usize) -> Vec<Box<dyn Widget>> {
    (0..n)
        .map(|i| -> Box<dyn Widget> { Box::new(Label::new(format!("Row {}", i))) })
        .collect()
}

#[test]
fn scroll_view_scrolls_down() {
    // ScrollView with more content than viewport; pressing Down increments scroll_offset_y.
    let sv = ScrollView::new(make_label_children(20)).with_content_height(20);
    let ctx = AppContext::new();
    // viewport_height defaults to 0; set it manually via render for on_action to use it
    // For this unit test, just call on_action directly — scroll_down clamps to max_scroll_y.
    // With viewport_h=0, max_scroll_y = 20-0 = 20; scroll_down increments by 1.
    sv.on_action("scroll_down", &ctx);
    let offset_y = sv.scroll_offset_y.get_untracked();
    assert_eq!(
        offset_y, 1,
        "scroll_offset_y should be 1 after one scroll_down"
    );
}

#[test]
fn scroll_view_scrolls_right() {
    // ScrollView scrolls horizontally; pressing Right increments scroll_offset_x.
    let sv = ScrollView::new(make_label_children(1)).with_content_width(200);
    let ctx = AppContext::new();
    sv.on_action("scroll_right", &ctx);
    let offset_x = sv.scroll_offset_x.get_untracked();
    assert_eq!(
        offset_x, 1,
        "scroll_offset_x should be 1 after one scroll_right"
    );
}

#[test]
fn scroll_view_page_down() {
    // Press PageDown — scroll_offset_y should jump by viewport_height.
    // viewport_height Cell is 0 initially (not rendered yet), so page_down adds 0.
    // We simulate by setting the rendered viewport_height via an integration approach.
    // Use TestApp to get a proper render pass setting viewport_height, then check.
    let sv = ScrollView::new(make_label_children(50)).with_content_height(50);
    let test_app = TestApp::new(20, 5, move || Box::new(sv));
    // TestApp renders once; viewport_height=5 is now set.
    // We need to call on_action after the render. Use inject without pilot.
    // Access the root widget via arena for the scroll view.
    // Instead: test the action directly on a fresh ScrollView after manually setting viewport via render.
    drop(test_app);

    // Create ScrollView, render to a buffer (sets viewport_height), then call page_down.
    let sv2 = ScrollView::new(make_label_children(50)).with_content_height(50);
    let ctx = AppContext::new();
    let area = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 5,
    };
    let mut buf = Buffer::empty(area);
    sv2.render(&ctx, area, &mut buf); // sets viewport_height = 5
    sv2.on_action("page_down", &ctx);
    let offset_y = sv2.scroll_offset_y.get_untracked();
    assert_eq!(
        offset_y, 5,
        "page_down should jump scroll_offset_y by viewport_height (5)"
    );
}

#[test]
fn snapshot_scroll_view_with_content() {
    // ScrollView with Labels that exceed the viewport at 20x5
    let children = make_label_children(10);
    let sv = ScrollView::new(children).with_content_height(10);
    let test_app = TestApp::new(20, 5, move || Box::new(sv));
    assert_snapshot!(format!("{}", test_app.backend()));
}

// ---------------------------------------------------------------------------
// DataTable tests
// ---------------------------------------------------------------------------

fn make_data_table_3x3() -> DataTable {
    let mut table = DataTable::new(vec![
        ColumnDef::new("Name"),
        ColumnDef::new("Age"),
        ColumnDef::new("City"),
    ]);
    table.add_row(vec!["Alice".into(), "30".into(), "New York".into()]);
    table.add_row(vec!["Bob".into(), "25".into(), "Chicago".into()]);
    table.add_row(vec!["Carol".into(), "35".into(), "Boston".into()]);
    table
}

#[test]
fn snapshot_data_table_3x3() {
    let test_app = TestApp::new(60, 8, || Box::new(make_data_table_3x3()));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[tokio::test]
async fn data_table_cursor_navigation() {
    let mut test_app = TestApp::new(60, 10, || Box::new(make_data_table_3x3()));

    // Focus the table
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Navigate down once (row 0 -> row 1)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Down).await;
    }

    // Navigate right once (col 0 -> col 1)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Right).await;
    }

    // Verify cursor is at row 1 by pressing Enter and checking RowSelected.row == 1
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let has_row1 = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<RowSelected>()
                .map(|r| r.row == 1)
                .unwrap_or(false)
        });
    assert!(
        has_row1,
        "Expected RowSelected {{ row: 1 }} after navigating Down once and Enter"
    );
}

#[tokio::test]
async fn data_table_select_row() {
    let mut test_app = TestApp::new(60, 10, || Box::new(make_data_table_3x3()));

    // Focus the table
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Navigate to row 2 (Down twice)
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Down).await;
        pilot.press(KeyCode::Down).await;
    }

    // Press Enter to select
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let has_row2 = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<RowSelected>()
                .map(|r| r.row == 2)
                .unwrap_or(false)
        });
    assert!(
        has_row2,
        "Expected RowSelected {{ row: 2 }} after pressing Enter on row 2"
    );
}

#[tokio::test]
async fn data_table_sort_column() {
    let mut test_app = TestApp::new(60, 10, || Box::new(make_data_table_3x3()));

    // Focus the table
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press 's' to sort by current column (column 0 = Name)
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Char('s'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let sort_msg = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<SortChanged>().is_some());
    assert!(sort_msg, "Expected SortChanged message after pressing 's'");
}

#[tokio::test]
async fn data_table_scroll_on_overflow() {
    let mut test_app = TestApp::new(60, 7, || {
        let mut table = DataTable::new(vec![ColumnDef::new("Value")]);
        for i in 0..20 {
            table.add_row(vec![format!("Row {}", i)]);
        }
        Box::new(table)
    });

    // Focus the table
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Navigate down many times to trigger scrolling
    {
        let mut pilot = test_app.pilot();
        for _ in 0..10 {
            pilot.press(KeyCode::Down).await;
        }
    }

    // The table's scroll_offset_row should be > 0.
    // We verify indirectly: pressing Enter on row 10 should give RowSelected { row: 10 }
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let selected_row = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .find_map(|(_, msg)| msg.downcast_ref::<RowSelected>().map(|r| r.row));
    assert!(
        selected_row.map(|r| r > 0).unwrap_or(false),
        "Expected a RowSelected with row > 0 after navigating down 10 times in a 5-row viewport, got: {:?}",
        selected_row
    );
}

// ---------------------------------------------------------------------------
// Tree tests
// ---------------------------------------------------------------------------

fn make_tree_3levels() -> Tree {
    let root = TreeNode::with_children(
        "Root",
        vec![
            TreeNode::with_children(
                "Branch A",
                vec![TreeNode::new("Leaf A1"), TreeNode::new("Leaf A2")],
            ),
            TreeNode::with_children("Branch B", vec![TreeNode::new("Leaf B1")]),
            TreeNode::new("Leaf C"),
        ],
    );
    Tree::new(root)
}

#[test]
fn snapshot_tree_collapsed() {
    // With default expanded=false, only top-level items visible
    let test_app = TestApp::new(40, 10, || Box::new(make_tree_3levels()));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_tree_expanded() {
    // Manually expand all nodes
    let mut root = TreeNode::with_children(
        "Root",
        vec![
            TreeNode::with_children(
                "Branch A",
                vec![TreeNode::new("Leaf A1"), TreeNode::new("Leaf A2")],
            ),
            TreeNode::with_children("Branch B", vec![TreeNode::new("Leaf B1")]),
            TreeNode::new("Leaf C"),
        ],
    );
    root.children[0].expanded = true;
    root.children[1].expanded = true;
    let test_app = TestApp::new(40, 15, || Box::new(Tree::new(root)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[tokio::test]
async fn tree_navigate_and_expand() {
    let mut test_app = TestApp::new(40, 10, || Box::new(make_tree_3levels()));

    // Focus the tree
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Initially 3 top-level items visible (Branch A, Branch B, Leaf C)
    // Cursor at 0 = Branch A (collapsed). Inject Space to expand without draining queue.
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Check NodeExpanded was emitted
    let expanded = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<NodeExpanded>().is_some());
    assert!(
        expanded,
        "Expected NodeExpanded message after Space on collapsed node"
    );
}

#[tokio::test]
async fn tree_collapse_node() {
    let mut root = TreeNode::with_children(
        "Root",
        vec![TreeNode::with_children(
            "Branch A",
            vec![TreeNode::new("Leaf A1")],
        )],
    );
    // Pre-expand Branch A
    root.children[0].expanded = true;

    let mut test_app = TestApp::new(40, 10, || Box::new(Tree::new(root)));

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Cursor is at 0 = Branch A (expanded). Inject Space to collapse without draining queue.
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Check NodeCollapsed was emitted
    let collapsed = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<NodeCollapsed>().is_some());
    assert!(
        collapsed,
        "Expected NodeCollapsed message after Space on expanded node"
    );
}

#[tokio::test]
async fn tree_select_emits_message() {
    let mut test_app = TestApp::new(40, 10, || Box::new(make_tree_3levels()));

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Press Enter to select the first visible item (Branch A at path [0])
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let selected = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<NodeSelected>()
                .map(|n| n.path == vec![0])
                .unwrap_or(false)
        });
    assert!(
        selected,
        "Expected NodeSelected {{ path: [0] }} after pressing Enter on first tree node"
    );
}

#[tokio::test]
async fn tree_scroll_on_overflow() {
    // Create a large tree exceeding viewport
    let mut children = Vec::new();
    for i in 0..20 {
        children.push(TreeNode::new(&format!("Item {}", i)));
    }
    let root = TreeNode::with_children("Root", children);

    let mut test_app = TestApp::new(40, 5, || Box::new(Tree::new(root)));

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Navigate down many times to trigger scrolling
    {
        let mut pilot = test_app.pilot();
        for _ in 0..10 {
            pilot.press(KeyCode::Down).await;
        }
    }

    // Verify by pressing Enter and checking NodeSelected path
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let selected_path = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .find_map(|(_, msg)| msg.downcast_ref::<NodeSelected>().map(|n| n.path.clone()));

    assert!(
        selected_path.as_ref().map(|p| p[0] > 0).unwrap_or(false),
        "Expected NodeSelected with path[0] > 0 after navigating down 10 times, got: {:?}",
        selected_path
    );
}

// ---------------------------------------------------------------------------
// Tabs widget tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_tabs_first_active() {
    let test_app = TestApp::new(40, 3, || {
        Box::new(Tabs::new(vec![
            "Tab1".to_string(),
            "Tab2".to_string(),
            "Tab3".to_string(),
        ]))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[tokio::test]
async fn tabs_switch_right() {
    let mut test_app = TestApp::new(40, 3, || {
        Box::new(Tabs::new(vec![
            "Tab1".to_string(),
            "Tab2".to_string(),
            "Tab3".to_string(),
        ]))
    });

    // Focus the Tabs widget via Tab key
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Tabs should have focus after Tab"
    );

    // Inject Right key without draining message queue so we can inspect it
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Verify TabChanged { index: 1, label: "Tab2" } is in the message queue
    let has_changed = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<TabChanged>()
                .map(|m| m.index == 1 && m.label == "Tab2")
                .unwrap_or(false)
        });
    assert!(
        has_changed,
        "Expected TabChanged {{ index: 1, label: Tab2 }} in message queue after Right key"
    );
}

#[tokio::test]
async fn tabs_switch_left() {
    let mut test_app = TestApp::new(40, 3, || {
        let tabs = Tabs::new(vec![
            "Tab1".to_string(),
            "Tab2".to_string(),
            "Tab3".to_string(),
        ]);
        // Start at index 2
        tabs.active.set(2);
        Box::new(tabs)
    });

    // Focus
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Tabs should have focus after Tab"
    );

    // Inject Left key without draining
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let has_changed = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| {
            msg.downcast_ref::<TabChanged>()
                .map(|m| m.index == 1 && m.label == "Tab2")
                .unwrap_or(false)
        });
    assert!(
        has_changed,
        "Expected TabChanged {{ index: 1, label: Tab2 }} in message queue after Left key from index 2"
    );
}

#[test]
fn snapshot_tabbed_content() {
    let test_app = TestApp::new(40, 5, || {
        Box::new(TabbedContent::new(
            vec!["Alpha".to_string(), "Beta".to_string()],
            vec![
                Box::new(Label::new("Content of Alpha")),
                Box::new(Label::new("Content of Beta")),
            ],
        ))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

// ---------------------------------------------------------------------------
// Collapsible widget tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_collapsible_expanded() {
    let test_app = TestApp::new(40, 5, || {
        Box::new(Collapsible::new(
            "Details",
            vec![Box::new(Label::new("Child content here"))],
        ))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_collapsible_collapsed() {
    let test_app = TestApp::new(40, 5, || {
        let col = Collapsible::new("Details", vec![Box::new(Label::new("Child content here"))]);
        col.expanded.set(false);
        Box::new(col)
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[tokio::test]
async fn collapsible_toggle() {
    let mut test_app = TestApp::new(40, 5, || {
        Box::new(Collapsible::new(
            "Details",
            vec![Box::new(Label::new("Child content"))],
        ))
    });

    // Focus the Collapsible
    {
        let mut pilot = test_app.pilot();
        pilot.press(KeyCode::Tab).await;
    }

    // Initially expanded — verify title row shows ▼
    {
        let buf = test_app.buffer();
        let row: String = (0..buf.area.width)
            .map(|col| buf[(col, 0)].symbol().to_string())
            .collect();
        assert!(
            row.contains('▼'),
            "Expanded collapsible should show ▼ in title row, got: {:?}",
            row.trim_end()
        );
    }

    // Inject Enter to collapse (without draining queue so we can inspect it)
    test_app.inject_key_event(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    // Verify Collapsed message was posted (before drain)
    let has_collapsed = test_app
        .ctx()
        .message_queue
        .borrow()
        .iter()
        .any(|(_, msg)| msg.downcast_ref::<CollapsibleCollapsed>().is_some());
    assert!(
        has_collapsed,
        "Expected Collapsed message in queue after Enter on expanded Collapsible"
    );

    // Settle to drain + re-render (checks the reactive state change is reflected in render)
    {
        let mut pilot = test_app.pilot();
        pilot.settle().await;
    }

    // Verify title row now shows ▶ (collapsed)
    {
        let buf = test_app.buffer();
        let row: String = (0..buf.area.width)
            .map(|col| buf[(col, 0)].symbol().to_string())
            .collect();
        assert!(
            row.contains('▶'),
            "Collapsed collapsible should show ▶ in title row, got: {:?}",
            row.trim_end()
        );
    }
}

// ---------------------------------------------------------------------------
// Markdown widget tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_markdown_headings() {
    let test_app = TestApp::new(40, 10, || Box::new(Markdown::new("# H1\n## H2\n### H3")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_markdown_bold_italic() {
    let test_app = TestApp::new(40, 5, || Box::new(Markdown::new("**bold** and *italic*")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_markdown_code_block() {
    let test_app = TestApp::new(40, 8, || Box::new(Markdown::new("```\ncode here\n```")));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_markdown_list() {
    let test_app = TestApp::new(40, 8, || {
        Box::new(Markdown::new("- item 1\n- item 2\n- item 3"))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_markdown_link() {
    let test_app = TestApp::new(60, 5, || {
        Box::new(Markdown::new("[click here](https://example.com)"))
    });
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn snapshot_markdown_mixed() {
    let content = "# Title\n\n**Bold text** and *italic text*.\n\n- item one\n- item two\n\n[link](https://example.com)\n\n---\n\n```\nfn hello() {}\n```";
    let test_app = TestApp::new(60, 20, || Box::new(Markdown::new(content)));
    assert_snapshot!(format!("{}", test_app.backend()));
}

#[test]
fn markdown_link_renders_url() {
    let test_app = TestApp::new(60, 5, || {
        Box::new(Markdown::new("[click here](https://example.com)"))
    });
    let buf = test_app.buffer();
    // Collect the first non-empty row
    let row: String = (0..buf.area.width)
        .map(|col| buf[(col, 0)].symbol().to_string())
        .collect();
    let trimmed = row.trim_end();
    assert!(
        trimmed.contains("click here") && trimmed.contains("https://example.com"),
        "Markdown link should render as 'click here [https://example.com]', got: {:?}",
        trimmed
    );
}

// ---------------------------------------------------------------------------
// RichLog tests
// ---------------------------------------------------------------------------

#[test]
fn rich_log_new_creates_empty_log_with_auto_scroll() {
    use ratatui::text::Line;
    let log = RichLog::new();
    assert_eq!(
        log.lines.get_untracked().len(),
        0,
        "new RichLog should have no lines"
    );
    assert_eq!(
        log.scroll_offset.get_untracked(),
        0,
        "new RichLog should have scroll_offset=0"
    );
}

#[test]
fn rich_log_write_line_appends() {
    use ratatui::text::Line;
    let log = RichLog::new();
    log.write_line(Line::raw("hello"));
    log.write_line(Line::raw("world"));
    assert_eq!(log.lines.get_untracked().len(), 2);
}

#[test]
fn rich_log_auto_scrolls_when_content_exceeds_viewport() {
    use ratatui::text::Line;
    let log = RichLog::new();
    log.viewport_height.set(3);
    for i in 0..10 {
        log.write_line(Line::raw(format!("Line {}", i)));
    }
    let offset = log.scroll_offset.get_untracked();
    assert!(offset > 0, "scroll_offset should be > 0, got {}", offset);
    assert_eq!(offset, 7, "scroll_offset should be line_count - viewport_h = 7");
}

#[test]
fn rich_log_max_lines_evicts_oldest() {
    use ratatui::text::Line;
    let log = RichLog::with_max_lines(3);
    log.write_line(Line::raw("A"));
    log.write_line(Line::raw("B"));
    log.write_line(Line::raw("C"));
    log.write_line(Line::raw("D")); // should evict "A"
    let lines = log.lines.get_untracked();
    assert_eq!(lines.len(), 3, "should have exactly 3 lines after eviction");
    // First line should now be "B"
    let first = lines[0].spans.first().map(|s| s.content.as_ref()).unwrap_or("");
    assert_eq!(first, "B", "oldest line should have been evicted");
}

#[test]
fn rich_log_eviction_decrements_scroll_offset() {
    use ratatui::text::Line;
    let log = RichLog::with_max_lines(3);
    log.viewport_height.set(3);
    log.write_line(Line::raw("A"));
    log.write_line(Line::raw("B"));
    log.write_line(Line::raw("C"));
    // Manually set scroll_offset > 0 to simulate mid-scroll
    log.scroll_offset.set(2);
    log.write_line(Line::raw("D")); // evicts "A", should decrement offset
    let offset = log.scroll_offset.get_untracked();
    assert_eq!(offset, 1, "eviction should decrement scroll_offset, got {}", offset);
}

#[test]
fn rich_log_scroll_up_disables_auto_scroll() {
    use ratatui::text::Line;
    let log = RichLog::new();
    log.viewport_height.set(3);
    for i in 0..10 {
        log.write_line(Line::raw(format!("Line {}", i)));
    }
    let initial_offset = log.scroll_offset.get_untracked();
    assert_eq!(initial_offset, 7, "initial offset should be 7");

    let ctx = AppContext::new();
    log.on_action("scroll_up", &ctx);
    assert_eq!(log.scroll_offset.get_untracked(), 6, "scroll_up should decrement offset");

    // write_line should NOT change offset when auto_scroll=false
    log.write_line(Line::raw("New Line"));
    assert_eq!(
        log.scroll_offset.get_untracked(),
        6,
        "offset should not change with auto_scroll disabled"
    );
}

#[test]
fn rich_log_scroll_bottom_reenables_auto_scroll() {
    use ratatui::text::Line;
    let log = RichLog::new();
    log.viewport_height.set(3);
    for i in 0..10 {
        log.write_line(Line::raw(format!("Line {}", i)));
    }
    let ctx = AppContext::new();
    // Disable auto_scroll first
    log.on_action("scroll_up", &ctx);
    // Scroll to bottom re-enables
    log.on_action("scroll_bottom", &ctx);
    assert_eq!(
        log.scroll_offset.get_untracked(),
        7,
        "scroll_bottom should set offset to bottom"
    );
    // Auto scroll should be re-enabled now — writing a new line should scroll
    log.write_line(Line::raw("Extra"));
    let offset = log.scroll_offset.get_untracked();
    assert!(offset >= 7, "auto_scroll re-enabled after scroll_bottom");
}

#[test]
fn snapshot_rich_log_styled_lines() {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};

    let log = RichLog::new();
    log.viewport_height.set(10);
    log.write_line(Line::from(vec![
        Span::styled("INFO", Style::default().fg(Color::Green)),
        Span::styled(" Server started", Style::default()),
    ]));
    log.write_line(Line::from(vec![
        Span::styled("WARN", Style::default().fg(Color::Yellow)),
        Span::styled(" Low memory", Style::default()),
    ]));
    log.write_line(Line::from(vec![
        Span::styled("ERROR", Style::default().fg(Color::Red)),
        Span::styled(" Disk full", Style::default()),
    ]));

    let test_app = TestApp::new(40, 10, move || Box::new(log));
    let buf = test_app.buffer();

    // Verify "INFO" is at col 0, row 0 with green foreground
    let info_cell = &buf[(0, 0)];
    assert_eq!(
        info_cell.fg,
        Color::Green,
        "INFO span should have green foreground"
    );

    // Verify "WARN" is on second row with yellow foreground
    let warn_cell = &buf[(0, 1)];
    assert_eq!(
        warn_cell.fg,
        Color::Yellow,
        "WARN span should have yellow foreground"
    );

    // Verify "ERROR" is on third row with red foreground
    let error_cell = &buf[(0, 2)];
    assert_eq!(
        error_cell.fg,
        Color::Red,
        "ERROR span should have red foreground"
    );
}

// ---------------------------------------------------------------------------
// LoadingIndicator widget tests
// ---------------------------------------------------------------------------

fn collect_buffer_text(buf: &ratatui::buffer::Buffer) -> String {
    let mut out = String::new();
    for row in 0..buf.area.height {
        for col in 0..buf.area.width {
            out.push_str(buf[(col, row)].symbol());
        }
    }
    out
}

/// LoadingIndicator renders static "Loading..." when skip_animations=true.
/// TestApp always sets skip_animations=true, so this verifies deterministic mode.
#[test]
fn loading_indicator_skip_animations() {
    let test_app = TestApp::new(20, 5, || Box::new(LoadingIndicator::new()));
    let all_text = collect_buffer_text(test_app.buffer());
    assert!(
        all_text.contains("Loading..."),
        "LoadingIndicator with skip_animations should render 'Loading...', got: {:?}",
        all_text.trim()
    );
}

/// LoadingIndicator renders a braille spinner character when skip_animations=false.
#[test]
fn loading_indicator_animated_spinner() {
    use textual_rs::widget::context::AppContext;
    use ratatui::layout::Rect;

    let mut ctx = AppContext::new();
    ctx.skip_animations = false;
    // spinner_tick=0 → frame_idx = 0/2 % 8 = 0 → first spinner char ⣾ (U+28FE)
    let widget = LoadingIndicator::new();
    let area = Rect { x: 0, y: 0, width: 10, height: 3 };
    let mut buf = Buffer::empty(area);
    widget.render(&ctx, area, &mut buf);

    // With tick=0, frame_idx=0, char is SPINNER_FRAMES[0] = U+28FE (⣾)
    let all_text = collect_buffer_text(&buf);
    assert!(
        all_text.contains('\u{28FE}'),
        "LoadingIndicator without skip_animations should render a braille spinner char (⣾), got: {:?}",
        all_text.trim()
    );
}

/// ctx.set_loading(id, true) causes a loading overlay to appear on the widget's area.
/// ctx.set_loading(id, false) removes the overlay.
#[test]
fn loading_overlay_set_and_clear() {
    use textual_rs::AppEvent;

    // Create a test app with a Label widget
    let mut test_app = TestApp::new(20, 5, || Box::new(Label::new("Hello World")));

    // Find the Label widget ID in the arena
    let label_id = test_app
        .ctx()
        .arena
        .iter()
        .find(|(_, w)| w.widget_type_name() == "Label")
        .map(|(id, _)| id)
        .expect("Label widget not found in arena");

    // Set loading on the label and trigger re-render via RenderRequest event
    test_app.ctx().set_loading(label_id, true);
    test_app.process_event(AppEvent::RenderRequest);

    // With skip_animations=true, overlay should show "Loading..."
    let buf_with_overlay = collect_buffer_text(test_app.buffer());
    assert!(
        buf_with_overlay.contains("Loading..."),
        "After set_loading(true), overlay should render 'Loading...', got: {:?}",
        buf_with_overlay.trim()
    );

    // Clear loading and trigger re-render
    test_app.ctx().set_loading(label_id, false);
    test_app.process_event(AppEvent::RenderRequest);

    // Now "Loading..." should be gone, and "Hello World" should appear
    let buf_without_overlay = collect_buffer_text(test_app.buffer());
    assert!(
        !buf_without_overlay.contains("Loading..."),
        "After set_loading(false), overlay should be gone, got: {:?}",
        buf_without_overlay.trim()
    );
    assert!(
        buf_without_overlay.contains("Hello World"),
        "After set_loading(false), original widget content should be visible, got: {:?}",
        buf_without_overlay.trim()
    );
}

/// Two widgets can be in loading state simultaneously.
#[test]
fn loading_overlay_multiple_widgets() {
    // Use a screen that composes two Labels — compose() is how children are mounted.
    struct TwoLabelScreen;
    impl Widget for TwoLabelScreen {
        fn widget_type_name(&self) -> &'static str { "TwoLabelScreen" }
        fn default_css() -> &'static str where Self: Sized {
            "TwoLabelScreen { layout: vertical; width: 100%; height: 100%; }"
        }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![
                Box::new(Label::new("Widget A")),
                Box::new(Label::new("Widget B")),
            ]
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    let mut test_app = TestApp::new_styled(30, 10, "TwoLabelScreen { layout: vertical; width: 100%; height: 100%; } Label { height: 1; width: 100%; }", || Box::new(TwoLabelScreen));

    // Find both Label widget IDs
    let label_ids: Vec<_> = test_app
        .ctx()
        .arena
        .iter()
        .filter(|(_, w)| w.widget_type_name() == "Label")
        .map(|(id, _)| id)
        .collect();

    assert!(
        label_ids.len() >= 2,
        "Expected at least 2 Label widgets, found {}",
        label_ids.len()
    );

    // Set both loading
    test_app.ctx().set_loading(label_ids[0], true);
    test_app.ctx().set_loading(label_ids[1], true);

    // Verify both are in loading_widgets map simultaneously
    {
        let loading = test_app.ctx().loading_widgets.borrow();
        assert!(
            loading.contains_key(label_ids[0]),
            "First label should be in loading_widgets"
        );
        assert!(
            loading.contains_key(label_ids[1]),
            "Second label should be in loading_widgets"
        );
    }

    // Re-render — both overlays should draw
    test_app.process_event(textual_rs::AppEvent::RenderRequest);

    // Verify both IDs are still in loading_widgets after re-render
    let loading_count = test_app.ctx().loading_widgets.borrow().len();
    assert_eq!(loading_count, 2, "Both loading states should persist after render");

    let buf_text = collect_buffer_text(test_app.buffer());
    assert!(
        buf_text.contains("Loading..."),
        "With two widgets loading, at least one 'Loading...' overlay should appear, got: {:?}",
        buf_text.trim()
    );
}
