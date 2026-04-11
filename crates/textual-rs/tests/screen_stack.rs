//! Integration tests for the screen stack: push, pop, modal, focus restore,
//! input scoping, and multi-screen layered rendering.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::{buffer::Buffer, layout::Rect};
use std::cell::Cell;
use std::sync::Arc;
use textual_rs::event::{AppEvent, KeyBinding};
use textual_rs::testing::TestApp;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::screen::ModalScreen;
use textual_rs::widget::EventPropagation;
use textual_rs::{Widget, WidgetId};

// ---------------------------------------------------------------------------
// Helper widget types
// ---------------------------------------------------------------------------

/// A screen that fills its area with a repeated character (for render inspection).
struct FilledScreen {
    ch: char,
}

impl FilledScreen {
    fn new(ch: char) -> Self {
        Self { ch }
    }
}

impl Widget for FilledScreen {
    fn widget_type_name(&self) -> &'static str {
        "FilledScreen"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let line: String = std::iter::repeat(self.ch)
            .take(area.width as usize)
            .collect();
        for y in area.y..area.y + area.height {
            buf.set_string(area.x, y, &line, ratatui::style::Style::default());
        }
    }
}

/// A focusable widget that records whether it received a key event.
struct KeyRecordingWidget {
    received_key: Arc<Cell<Option<char>>>,
}

impl KeyRecordingWidget {
    fn new(received_key: Arc<Cell<Option<char>>>) -> Self {
        Self { received_key }
    }
}

impl Widget for KeyRecordingWidget {
    fn widget_type_name(&self) -> &'static str {
        "KeyRecordingWidget"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        if let Some(key) = event.downcast_ref::<KeyEvent>() {
            if let KeyCode::Char(c) = key.code {
                self.received_key.set(Some(c));
            }
        }
        EventPropagation::Continue
    }
}

/// Screen wrapping a KeyRecordingWidget.
struct KeyRecordingScreen {
    received: Arc<Cell<Option<char>>>,
}

impl KeyRecordingScreen {
    fn new(received: Arc<Cell<Option<char>>>) -> Self {
        Self { received }
    }
}

impl Widget for KeyRecordingScreen {
    fn widget_type_name(&self) -> &'static str {
        "KeyRecordingScreen"
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(KeyRecordingWidget::new(self.received.clone()))]
    }
}

/// A screen with a single focusable child widget.
struct SingleFocusScreen;

impl Widget for SingleFocusScreen {
    fn widget_type_name(&self) -> &'static str {
        "SingleFocusScreen"
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(FocusableWidget)]
    }
}

/// Minimal focusable widget used inside screens.
struct FocusableWidget;

impl Widget for FocusableWidget {
    fn widget_type_name(&self) -> &'static str {
        "FocusableWidget"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

/// A screen that pushes a modal when it receives an 'm' key binding.
struct ModalLaunchScreen;

impl Widget for ModalLaunchScreen {
    fn widget_type_name(&self) -> &'static str {
        "ModalLaunchScreen"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let line: String = std::iter::repeat('B').take(area.width as usize).collect();
        for y in area.y..area.y + area.height {
            buf.set_string(area.x, y, &line, ratatui::style::Style::default());
        }
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        &[
            // Statically defined — only one binding needed for test
        ]
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "open_modal" {
            ctx.push_screen_deferred(Box::new(ModalScreen::new(Box::new(ModalContent))));
        }
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key) = event.downcast_ref::<KeyEvent>() {
            if key.code == KeyCode::Char('m') && key.kind == KeyEventKind::Press {
                self.on_action("open_modal", ctx);
                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(FocusableWidget)]
    }
}

/// Content widget for the modal dialog (focusable so it receives key events).
struct ModalContent;

impl Widget for ModalContent {
    fn widget_type_name(&self) -> &'static str {
        "ModalContent"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let line: String = std::iter::repeat('M').take(area.width as usize).collect();
        for y in area.y..area.y + area.height {
            buf.set_string(area.x, y, &line, ratatui::style::Style::default());
        }
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key) = event.downcast_ref::<KeyEvent>() {
            if key.code == KeyCode::Esc && key.kind == KeyEventKind::Press {
                ctx.pop_screen_deferred();
                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }
}

/// Screen that pushes a second screen on 'n' key.
struct NavScreen {
    own_id: Cell<Option<WidgetId>>,
}

impl NavScreen {
    fn new() -> Self {
        Self {
            own_id: Cell::new(None),
        }
    }
}

impl Widget for NavScreen {
    fn widget_type_name(&self) -> &'static str {
        "NavScreen"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key) = event.downcast_ref::<KeyEvent>() {
            if key.code == KeyCode::Char('n') && key.kind == KeyEventKind::Press {
                ctx.push_screen_deferred(Box::new(SingleFocusScreen));
                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(FocusableWidget)]
    }
}

/// Screen that pops itself when 'b' is pressed.
struct PopScreen;

impl Widget for PopScreen {
    fn widget_type_name(&self) -> &'static str {
        "PopScreen"
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key) = event.downcast_ref::<KeyEvent>() {
            if key.code == KeyCode::Char('b') && key.kind == KeyEventKind::Press {
                ctx.pop_screen_deferred();
                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(FocusableWidget)]
    }
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a key press AppEvent.
fn key_event(code: KeyCode) -> AppEvent {
    AppEvent::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn screen_stack_initial_state() {
    let app = TestApp::new(80, 24, || Box::new(SingleFocusScreen));
    assert_eq!(app.ctx().screen_stack.len(), 1);
    assert!(
        app.ctx().focused_widget.is_some(),
        "first focusable widget should be auto-focused"
    );
}

#[test]
fn screen_stack_keyboard_scoped_to_top_screen() {
    // Base screen records key events; second screen records separate events.
    let base_received = Arc::new(Cell::new(None::<char>));
    let base_clone = base_received.clone();

    let second_received = Arc::new(Cell::new(None::<char>));
    let second_clone = second_received.clone();

    let mut app = TestApp::new(80, 24, move || {
        Box::new(KeyRecordingScreen::new(base_clone.clone()))
    });

    // Send 'a' to base screen — base should receive it
    app.process_event(key_event(KeyCode::Char('a')));
    assert_eq!(
        base_received.get(),
        Some('a'),
        "base screen should receive 'a'"
    );

    // Reset and push a second screen with its own recorder
    base_received.set(None);
    app.ctx()
        .push_screen_deferred(Box::new(KeyRecordingScreen::new(second_clone.clone())));
    // Send a dummy event to trigger process_deferred_screens
    app.process_event(AppEvent::RenderRequest);

    assert_eq!(app.ctx().screen_stack.len(), 2);

    // Send 'b' — only second screen should receive it (base is frozen)
    app.process_event(key_event(KeyCode::Char('b')));
    assert_eq!(
        base_received.get(),
        None,
        "base screen must NOT receive keys when second screen is on top"
    );
    assert_eq!(
        second_received.get(),
        Some('b'),
        "second screen should receive 'b'"
    );
}

#[test]
fn screen_stack_focus_history_tracks_pushes_and_pops() {
    let mut app = TestApp::new(80, 24, || Box::new(SingleFocusScreen));

    // Initial push gives focus_history.len() == 1
    assert_eq!(app.ctx().focus_history.len(), 1);
    let initial_focus = app.ctx().focused_widget;

    // Push another screen
    app.ctx().push_screen_deferred(Box::new(SingleFocusScreen));
    app.process_event(AppEvent::RenderRequest);

    assert_eq!(app.ctx().focus_history.len(), 2);
    assert_eq!(app.ctx().screen_stack.len(), 2);

    // Pop screen
    app.ctx().pop_screen_deferred();
    app.process_event(AppEvent::RenderRequest);

    assert_eq!(app.ctx().focus_history.len(), 1);
    assert_eq!(app.ctx().screen_stack.len(), 1);

    // Focus should be restored to initial_focus
    assert_eq!(app.ctx().focused_widget, initial_focus);
}

#[test]
fn screen_stack_pop_last_screen_is_noop() {
    let mut app = TestApp::new(80, 24, || Box::new(SingleFocusScreen));
    assert_eq!(app.ctx().screen_stack.len(), 1);

    // Popping the only screen should be a no-op
    app.ctx().pop_screen_deferred();
    app.process_event(AppEvent::RenderRequest);

    // Stack should remain at 1
    assert_eq!(
        app.ctx().screen_stack.len(),
        1,
        "popping last screen must be a no-op"
    );
}

#[test]
fn screen_stack_focus_restored_after_pop() {
    let mut app = TestApp::new(80, 24, || Box::new(NavScreen::new()));

    // Base screen's focusable child should be focused
    let base_focus = app.ctx().focused_widget;
    assert!(base_focus.is_some());

    // Push a second screen via event
    app.process_event(key_event(KeyCode::Char('n')));
    assert_eq!(app.ctx().screen_stack.len(), 2);

    // Push PopScreen on top
    app.ctx().push_screen_deferred(Box::new(PopScreen));
    app.process_event(AppEvent::RenderRequest);
    assert_eq!(app.ctx().screen_stack.len(), 3);

    // Pop via 'b'
    app.process_event(key_event(KeyCode::Char('b')));
    assert_eq!(app.ctx().screen_stack.len(), 2);

    // Pop again to return to base
    app.ctx().pop_screen_deferred();
    app.process_event(AppEvent::RenderRequest);
    assert_eq!(app.ctx().screen_stack.len(), 1);

    // Focus should be restored to base screen's focusable widget
    assert_eq!(
        app.ctx().focused_widget,
        base_focus,
        "focus should be restored to base screen widget after all pops"
    );
}

#[test]
fn screen_stack_modal_dismissed_via_pop() {
    let mut app = TestApp::new(80, 24, || Box::new(ModalLaunchScreen));

    let pre_modal_focus = app.ctx().focused_widget;
    assert!(pre_modal_focus.is_some());

    // Open modal via 'm' key
    app.process_event(key_event(KeyCode::Char('m')));
    assert_eq!(
        app.ctx().screen_stack.len(),
        2,
        "modal should be on stack after 'm'"
    );

    // Dismiss via Escape
    app.process_event(key_event(KeyCode::Esc));
    assert_eq!(
        app.ctx().screen_stack.len(),
        1,
        "modal should be dismissed after Esc"
    );

    // Focus restored
    assert_eq!(
        app.ctx().focused_widget,
        pre_modal_focus,
        "focus must restore to pre-modal widget after dismiss"
    );
}

#[test]
fn screen_stack_multi_screen_renders_all_screens() {
    // Push a background 'B' screen, then push a foreground 'M' screen.
    // Both screens fill their entire area. Rendering bottom-to-top means
    // the top screen (M) overwrites B — buffer should be all 'M'.
    let mut app = TestApp::new(10, 5, || Box::new(FilledScreen::new('B')));

    // Verify background renders B
    let content_initial: String = app
        .buffer()
        .content()
        .iter()
        .map(|c: &ratatui::buffer::Cell| c.symbol().chars().next().unwrap_or(' '))
        .collect();
    assert!(
        content_initial.contains('B'),
        "initial buffer should contain 'B'"
    );
    assert!(!content_initial.contains('M'), "no 'M' before overlay push");

    // Push overlay screen filling with 'M'
    app.ctx()
        .push_screen_deferred(Box::new(FilledScreen::new('M')));
    app.process_event(AppEvent::RenderRequest);

    let content_after: String = app
        .buffer()
        .content()
        .iter()
        .map(|c: &ratatui::buffer::Cell| c.symbol().chars().next().unwrap_or(' '))
        .collect();
    // Top screen 'M' overwrites 'B' — all cells should be 'M'
    assert!(content_after.contains('M'), "top screen should render 'M'");
    // Confirm no 'B' remains visible (top screen covered the whole terminal)
    assert!(
        !content_after.contains('B'),
        "background 'B' should be covered by top 'M' screen"
    );
}

#[test]
fn screen_stack_is_modal_returns_true_for_modal_screen() {
    // Verify ModalScreen.is_modal() returns true (used by framework for future use)
    let modal = ModalScreen::new(Box::new(FocusableWidget));
    assert!(
        modal.is_modal(),
        "ModalScreen must report is_modal() == true"
    );
}

// ---------------------------------------------------------------------------
// push_screen_wait / pop_screen_with tests (D-03)
// ---------------------------------------------------------------------------

/// A modal content widget that pops with a bool result when 'y' or 'n' is pressed.
struct ResultDialog;

impl Widget for ResultDialog {
    fn widget_type_name(&self) -> &'static str {
        "ResultDialog"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(key) = event.downcast_ref::<KeyEvent>() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('y') => {
                        ctx.pop_screen_with(true);
                        return EventPropagation::Stop;
                    }
                    KeyCode::Char('n') => {
                        ctx.pop_screen_with(false);
                        return EventPropagation::Stop;
                    }
                    _ => {}
                }
            }
        }
        EventPropagation::Continue
    }
}

#[test]
fn test_push_screen_wait_returns_result_true() {
    // push_screen_wait pushes a modal; pop_screen_with delivers the result
    let mut app = TestApp::new(80, 24, || Box::new(SingleFocusScreen));
    assert_eq!(app.ctx().screen_stack.len(), 1);

    // Push a modal via push_screen_wait; capture the receiver
    let mut rx = app
        .ctx()
        .push_screen_wait(Box::new(ModalScreen::new(Box::new(ResultDialog))));

    // Trigger process_deferred_screens to push the modal and register the sender
    app.process_event(AppEvent::RenderRequest);
    assert_eq!(app.ctx().screen_stack.len(), 2, "modal should be on stack");

    // Before pop, result should not yet be available
    assert!(
        rx.try_recv().is_err(),
        "result should not be available before pop"
    );

    // Press 'y' — ResultDialog calls pop_screen_with(true)
    app.process_event(key_event(KeyCode::Char('y')));
    assert_eq!(app.ctx().screen_stack.len(), 1, "modal should be popped");

    // Result should now be available through the oneshot receiver
    let boxed = rx
        .try_recv()
        .expect("result should be delivered after pop_screen_with");
    let confirmed: bool = *boxed.downcast::<bool>().expect("result should be bool");
    assert!(confirmed, "pop_screen_with(true) should yield true");
}

#[test]
fn test_push_screen_wait_cancel_returns_false() {
    // Same as above but press 'n' — result should be false
    let mut app = TestApp::new(80, 24, || Box::new(SingleFocusScreen));

    let mut rx = app
        .ctx()
        .push_screen_wait(Box::new(ModalScreen::new(Box::new(ResultDialog))));

    app.process_event(AppEvent::RenderRequest);
    assert_eq!(app.ctx().screen_stack.len(), 2);

    // Press 'n' — ResultDialog calls pop_screen_with(false)
    app.process_event(key_event(KeyCode::Char('n')));
    assert_eq!(app.ctx().screen_stack.len(), 1, "modal should be popped");

    let boxed = rx
        .try_recv()
        .expect("result should be delivered after pop_screen_with(false)");
    let confirmed: bool = *boxed.downcast::<bool>().expect("result should be bool");
    assert!(!confirmed, "pop_screen_with(false) should yield false");
}

#[test]
fn test_pop_screen_with_no_wait_is_noop() {
    // Calling pop_screen_with on a screen pushed via push_screen_deferred (not push_screen_wait)
    // should simply pop the screen without error — result is discarded, no panic.
    let mut app = TestApp::new(80, 24, || Box::new(SingleFocusScreen));

    // Push a normal modal (not via push_screen_wait)
    app.ctx()
        .push_screen_deferred(Box::new(ModalScreen::new(Box::new(ResultDialog))));
    app.process_event(AppEvent::RenderRequest);
    assert_eq!(app.ctx().screen_stack.len(), 2);

    // Press 'y' — ResultDialog calls pop_screen_with(true), but nobody is waiting for the result
    app.process_event(key_event(KeyCode::Char('y')));

    // Should pop cleanly without panic; result is silently dropped
    assert_eq!(
        app.ctx().screen_stack.len(),
        1,
        "modal should be popped even when no wait receiver exists"
    );
}
