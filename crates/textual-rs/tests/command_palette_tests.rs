//! Integration tests for the command palette: open, search, dispatch, and dismiss flows.

use std::cell::Cell;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use textual_rs::command::registry::fuzzy_score;
use textual_rs::testing::TestApp;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::{Widget, WidgetId};
use textual_rs::event::keybinding::KeyBinding;
use textual_rs::event::AppEvent;

// ---------------------------------------------------------------------------
// Helper widgets
// ---------------------------------------------------------------------------

/// A simple widget with visible key bindings for testing command discovery.
struct TestWidget {
    own_id: Cell<Option<WidgetId>>,
    /// Set to the action string when on_action is called.
    last_action: std::cell::RefCell<Option<String>>,
}

impl TestWidget {
    fn new() -> Self {
        Self {
            own_id: Cell::new(None),
            last_action: std::cell::RefCell::new(None),
        }
    }

    fn last_action(&self) -> Option<String> {
        self.last_action.borrow().clone()
    }
}

static TEST_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('s'),
        modifiers: KeyModifiers::CONTROL,
        action: "save",
        description: "Save File",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Char('o'),
        modifiers: KeyModifiers::CONTROL,
        action: "open",
        description: "Open File",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "submit",
        description: "internal submit",
        show: false, // hidden — should NOT appear in palette
    },
];

impl Widget for TestWidget {
    fn widget_type_name(&self) -> &'static str {
        "TestWidget"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        TEST_BINDINGS
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn on_action(&self, action: &str, _ctx: &AppContext) {
        *self.last_action.borrow_mut() = Some(action.to_string());
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---------------------------------------------------------------------------
// Test 1: command_palette_opens
// ---------------------------------------------------------------------------

/// Pressing Ctrl+P should push a new screen (CommandPalette) onto the screen_stack.
#[tokio::test]
async fn command_palette_opens() {
    let mut test_app = TestApp::new(80, 24, || Box::new(TestWidget::new()));

    let initial_screen_count = test_app.ctx().screen_stack.len();

    // Press Ctrl+P
    let ctrl_p = AppEvent::Key(KeyEvent {
        code: KeyCode::Char('p'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });
    test_app.process_event(ctrl_p);

    // Palette is now an active overlay, not a screen push
    assert!(
        test_app.ctx().active_overlay.borrow().is_some(),
        "Ctrl+P should set active_overlay to CommandPalette"
    );
}

// ---------------------------------------------------------------------------
// Test 2: command_palette_fuzzy_search
// ---------------------------------------------------------------------------

/// Unit tests for fuzzy_score function.
#[test]
fn command_palette_fuzzy_search() {
    // Empty query matches anything with score 1.0
    assert_eq!(fuzzy_score("", "Save File"), 1.0);

    // Exact substring match returns 1.0
    assert_eq!(fuzzy_score("save", "Save File"), 1.0);
    assert_eq!(fuzzy_score("sav", "Save File"), 1.0);

    // Case insensitive substring match
    assert_eq!(fuzzy_score("SAVE", "Save File"), 1.0);
    assert_eq!(fuzzy_score("FILE", "Save File"), 1.0);

    // Non-matching query has low score
    let low_score = fuzzy_score("xyz", "Save File");
    assert!(low_score < 0.5, "Non-matching query should have low score, got {}", low_score);

    // Partial match for "open" in "Open File"
    assert_eq!(fuzzy_score("open", "Open File"), 1.0);
}

// ---------------------------------------------------------------------------
// Test 3: command_palette_esc_dismisses
// ---------------------------------------------------------------------------

/// Opening the palette and pressing Esc should return screen_stack to original length.
#[tokio::test]
async fn command_palette_esc_dismisses() {
    let mut test_app = TestApp::new(80, 24, || Box::new(TestWidget::new()));

    let initial_screen_count = test_app.ctx().screen_stack.len();

    // Open palette
    let ctrl_p = AppEvent::Key(KeyEvent {
        code: KeyCode::Char('p'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });
    test_app.process_event(ctrl_p);
    assert!(test_app.ctx().active_overlay.borrow().is_some(), "Ctrl+P should open overlay");

    // Press Esc to dismiss
    let esc = AppEvent::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });
    test_app.process_event(esc);

    // Overlay should be dismissed
    assert!(
        test_app.ctx().active_overlay.borrow().is_none(),
        "Esc should dismiss the CommandPalette overlay"
    );
}

// ---------------------------------------------------------------------------
// Test 4: command_registry_discovers_bindings
// ---------------------------------------------------------------------------

/// CommandRegistry.discover_all() should include visible key bindings from mounted widgets.
#[test]
fn command_registry_discovers_bindings() {
    use textual_rs::command::CommandRegistry;
    use textual_rs::widget::tree::push_screen;

    let mut ctx = AppContext::new();

    // Build a screen with a TestWidget
    struct SimpleScreen {
        child: std::sync::Mutex<Option<Box<dyn Widget>>>,
    }

    impl SimpleScreen {
        fn new() -> Self {
            Self {
                child: std::sync::Mutex::new(Some(Box::new(TestWidget::new()))),
            }
        }
    }

    impl Widget for SimpleScreen {
        fn widget_type_name(&self) -> &'static str { "SimpleScreen" }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            let mut guard = self.child.lock().unwrap();
            if let Some(child) = guard.take() {
                vec![child]
            } else {
                vec![]
            }
        }
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    }

    push_screen(Box::new(SimpleScreen::new()), &mut ctx);

    let reg = CommandRegistry::new();
    let commands = reg.discover_all(&ctx);

    // Should discover "Save File" and "Open File" (show: true) but NOT "submit" (show: false)
    let names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"Save File"),
        "discover_all should include 'Save File' binding, got: {:?}", names
    );
    assert!(
        names.contains(&"Open File"),
        "discover_all should include 'Open File' binding, got: {:?}", names
    );
    assert!(
        !names.contains(&"internal submit"),
        "discover_all should NOT include hidden 'internal submit' binding"
    );
}

// ---------------------------------------------------------------------------
// Test 5: command_registry_app_commands
// ---------------------------------------------------------------------------

/// register() adds app-level commands that appear in discover_all.
#[test]
fn command_registry_app_commands() {
    use textual_rs::command::CommandRegistry;

    let mut reg = CommandRegistry::new();
    reg.register("Toggle Dark Mode", "toggle_theme");
    reg.register("Show Help", "help");

    let ctx = AppContext::new();
    let commands = reg.discover_all(&ctx);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].name, "Toggle Dark Mode");
    assert_eq!(commands[0].action, "toggle_theme");
    assert_eq!(commands[0].source, "app");
    assert!(commands[0].target_id.is_none());
    assert_eq!(commands[1].name, "Show Help");
}
