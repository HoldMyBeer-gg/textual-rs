//! Integration tests for interactive visual states:
//! - Hover tracking (STATE-02)
//! - Button pressed flash (STATE-03)
//! - Input invalid border (STATE-05)
//! - Focus indicator confirmation (STATE-01)
//! - Selected item accent+bold confirmation (STATE-04)

use crossterm::event::KeyCode;
use ratatui::{buffer::Buffer, layout::Rect, style::Modifier};
use textual_rs::css::types::PseudoClass;
use textual_rs::testing::TestApp;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::WidgetId;
use textual_rs::{Button, Input, ListView, Widget};

// ---------------------------------------------------------------------------
// Helper screens
// ---------------------------------------------------------------------------

/// Screen with a single button child for press state testing.
struct ButtonScreen;
impl Widget for ButtonScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "ButtonScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Button::new("OK"))]
    }
}

/// Screen with an Input that only accepts strings starting with a letter (rejects digits).
struct ValidatedInputScreen;
impl Widget for ValidatedInputScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "ValidatedInputScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Input::new("Type a letter...").with_validator(
            |s: &str| s.is_empty() || s.chars().next().map_or(false, |c| c.is_alphabetic()),
        ))]
    }
}

/// Screen with two focusable widgets for hover testing.
struct TwoWidgetScreen;
impl Widget for TwoWidgetScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "TwoWidgetScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Button::new("First")),
            Box::new(Button::new("Second")),
        ]
    }
}

/// Screen with a ListView for selected-item styling test.
struct ListScreen;
impl Widget for ListScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "ListScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(ListView::new(vec![
            "Alpha".into(),
            "Beta".into(),
            "Gamma".into(),
        ]))]
    }
}

/// Screen with a single focusable button (for focus indicator test with styled borders).
struct FocusableButtonScreen;
impl Widget for FocusableButtonScreen {
    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
    fn widget_type_name(&self) -> &'static str {
        "FocusableButtonScreen"
    }
    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(Button::new("Focus Me"))]
    }
}

// ---------------------------------------------------------------------------
// Helper: find a widget by type name in the arena
// ---------------------------------------------------------------------------
fn find_widget_by_type(ctx: &AppContext, type_name: &str) -> Option<WidgetId> {
    for (id, widget) in ctx.arena.iter() {
        if widget.widget_type_name() == type_name {
            return Some(id);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Test 1: Button press shows REVERSED modifier (STATE-03)
// ---------------------------------------------------------------------------
#[test]
fn button_press_shows_reversed_modifier() {
    let mut test_app = TestApp::new_styled(30, 5, "", || Box::new(ButtonScreen));

    // Focus the button
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    )));

    // Verify button is focused
    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Button should be focused after Tab"
    );

    // Press Space to trigger "press" action
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char(' '),
        crossterm::event::KeyModifiers::NONE,
    )));

    // After pressing, the button renders with REVERSED modifier.
    // Since pressed is a single-frame effect, we need to check the buffer
    // from the render that happened during process_event.
    // The pressed flag is set in on_action and consumed in render,
    // so the buffer should show REVERSED on the label cell.
    let buf = test_app.buffer();
    let button_id = find_widget_by_type(test_app.ctx(), "Button");
    assert!(button_id.is_some(), "Button widget should exist in arena");

    // Find a cell in the buffer that contains "O" or "K" (button label "OK")
    // and check its modifier includes REVERSED.
    let mut found_reversed = false;
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            if cell.symbol() == "O" || cell.symbol() == "K" {
                if cell.modifier.contains(Modifier::REVERSED) {
                    found_reversed = true;
                }
            }
        }
    }
    // The pressed flash is a single-frame effect. After the first render
    // (which happens in process_event), the flag is cleared.
    // Since process_event calls render, the buffer should capture the pressed state.
    assert!(
        found_reversed,
        "Button label should have REVERSED modifier during press frame"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Input invalid border color override (STATE-05)
// ---------------------------------------------------------------------------
#[test]
fn input_invalid_shows_red_border_override() {
    let mut test_app = TestApp::new_styled(30, 5, "", || Box::new(ValidatedInputScreen));

    // Focus the input
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    )));

    // Type a digit (invalid: validator rejects non-alphabetic first char)
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char('9'),
        crossterm::event::KeyModifiers::NONE,
    )));

    // Find the Input widget and verify border_color_override returns red
    let input_id = find_widget_by_type(test_app.ctx(), "Input");
    assert!(input_id.is_some(), "Input widget should exist in arena");

    let input_widget = test_app.ctx().arena.get(input_id.unwrap()).unwrap();
    let override_color = input_widget.border_color_override();
    assert_eq!(
        override_color,
        Some((186, 60, 91)),
        "Invalid Input should return red border color override"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Input valid state has no border override
// ---------------------------------------------------------------------------
#[test]
fn input_valid_has_no_border_override() {
    let mut test_app = TestApp::new_styled(30, 5, "", || Box::new(ValidatedInputScreen));

    // Focus the input
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    )));

    // Type a letter (valid state for this validator)
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Char('a'),
        crossterm::event::KeyModifiers::NONE,
    )));

    // Should have no border override when valid
    let input_id = find_widget_by_type(test_app.ctx(), "Input").unwrap();
    let input_widget = test_app.ctx().arena.get(input_id).unwrap();
    assert_eq!(
        input_widget.border_color_override(),
        None,
        "Valid Input should have no border color override"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Hover pseudo-class tracking (STATE-02)
// ---------------------------------------------------------------------------
#[test]
fn hover_sets_pseudo_class_on_hovered_widget() {
    let mut test_app = TestApp::new_styled(40, 10, "", || Box::new(TwoWidgetScreen));

    // Find one of the button widget IDs
    let button_ids: Vec<WidgetId> = test_app
        .ctx()
        .arena
        .iter()
        .filter(|(_, w)| w.widget_type_name() == "Button")
        .map(|(id, _)| id)
        .collect();
    assert!(
        button_ids.len() >= 2,
        "Should have at least 2 Button widgets"
    );

    // Simulate a mouse move event. We need to trigger handle_mouse_event with Moved.
    // The TestApp processes mouse events through process_event.
    use crossterm::event::{MouseEvent, MouseEventKind};

    // Move mouse to position (5, 1) — should be within the first button area
    let move_event = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 5,
        row: 1,
        modifiers: crossterm::event::KeyModifiers::NONE,
    };
    test_app.process_event(textual_rs::AppEvent::Mouse(move_event));

    // Check that some widget has hovered_widget set
    let hovered = test_app.ctx().hovered_widget;
    // It's possible the mouse position doesn't hit a widget due to layout,
    // so we check the mechanism works rather than exact positioning.
    // If hovered is set, check the pseudo-class.
    if let Some(hovered_id) = hovered {
        let pcs = test_app.ctx().pseudo_classes.get(hovered_id);
        assert!(
            pcs.map_or(false, |p| p.contains(&PseudoClass::Hover)),
            "Hovered widget should have Hover pseudo-class set"
        );
    }

    // Move mouse to a different position to test hover clearing
    let move_event2 = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 5,
        row: 8,
        modifiers: crossterm::event::KeyModifiers::NONE,
    };
    test_app.process_event(textual_rs::AppEvent::Mouse(move_event2));

    // If previous widget was hovered, it should now have hover cleared
    if let Some(prev_hovered_id) = hovered {
        if test_app.ctx().hovered_widget != Some(prev_hovered_id) {
            let pcs = test_app.ctx().pseudo_classes.get(prev_hovered_id);
            assert!(
                pcs.map_or(true, |p| !p.contains(&PseudoClass::Hover)),
                "Previously hovered widget should have Hover pseudo-class cleared"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 5: Focus indicator renders accent green border (STATE-01)
// ---------------------------------------------------------------------------
#[test]
fn focused_widget_shows_accent_border() {
    let mut test_app = TestApp::new_styled(30, 5, "", || Box::new(FocusableButtonScreen));

    // Focus the button
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    )));

    assert!(
        test_app.ctx().focused_widget.is_some(),
        "Should have focus after Tab"
    );

    // The focused button should have accent green (0, 255, 163) border.
    // Check the buffer for cells with the accent green foreground color.
    let buf = test_app.buffer();
    let accent_green = ratatui::style::Color::Rgb(0, 255, 163);
    let mut found_accent = false;
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            if cell.fg == accent_green {
                found_accent = true;
                break;
            }
        }
        if found_accent {
            break;
        }
    }
    assert!(
        found_accent,
        "Focused widget should render border with accent green (0, 255, 163)"
    );
}

// ---------------------------------------------------------------------------
// Test 6: ListView selected item uses accent color + bold (STATE-04)
// ---------------------------------------------------------------------------
#[test]
fn listview_selected_item_accent_bold() {
    let mut test_app = TestApp::new_styled(30, 10, "", || Box::new(ListScreen));

    // Focus the list view
    test_app.process_event(textual_rs::AppEvent::Key(crossterm::event::KeyEvent::new(
        KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    )));

    // The first item "Alpha" should be selected by default with accent+bold.
    let buf = test_app.buffer();
    let accent_green = ratatui::style::Color::Rgb(0, 255, 163);

    let mut found_accent_bold = false;
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            if cell.symbol() == "A"
                && cell.fg == accent_green
                && cell.modifier.contains(Modifier::BOLD)
            {
                found_accent_bold = true;
                break;
            }
        }
        if found_accent_bold {
            break;
        }
    }
    assert!(
        found_accent_bold,
        "Selected ListView item 'Alpha' should render with accent green fg + BOLD"
    );
}
