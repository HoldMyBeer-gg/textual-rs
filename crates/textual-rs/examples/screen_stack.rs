// Screen Stack Demo — push/pop screens and modal dialogs
//
// This demo shows how to:
//   1. Push a new screen onto the stack with ctx.push_screen_deferred()
//   2. Pop back to the previous screen with ctx.pop_screen_deferred()
//   3. Present a modal dialog that overlays the background screen
//   4. Restore focus automatically when a screen or modal is dismissed
//
// Controls (main screen):
//   n         — push a second screen
//   m         — open a modal dialog over the current screen
//   Ctrl+C    — quit (double-tap)
//
// Controls (second screen):
//   b / Esc   — pop back to main screen
//   m         — open a modal dialog
//
// Controls (modal):
//   Enter / Esc — dismiss the modal
//
// Run with: cargo run --example screen_stack

use std::cell::Cell;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use textual_rs::event::keybinding::KeyBinding;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::screen::ModalScreen;
use textual_rs::widget::{EventPropagation, WidgetId};
use textual_rs::{App, Button, ButtonVariant, Footer, Header, Label, Widget};
use textual_rs::widget::button::messages::Pressed;

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

const CSS: &str = r#"
MainScreen {
    background: $background;
    color: $foreground;
    layout-direction: vertical;
}
SecondScreen {
    background: #1a1a2e;
    color: $foreground;
    layout-direction: vertical;
}
ModalScreen {
    background: transparent;
}
ModalDialog {
    background: $panel;
    border: mcgugan-box $primary;
    width: 50;
    height: 12;
    margin: 6 15;
    padding: 1 2;
    layout-direction: vertical;
}
ScreenContent {
    flex-grow: 1;
    layout-direction: vertical;
    padding: 1 2;
}
Label {
    height: 1;
    color: $foreground;
}
Button {
    border: mcgugan-box $accent;
    height: 3;
    min-width: 20;
    color: $accent;
    margin: 0 0 1 0;
}
Header {
    height: 1;
    background: $panel;
    color: $primary;
}
Footer {
    height: 1;
    background: $panel;
    color: $text;
}
"#;

// ---------------------------------------------------------------------------
// MainScreen — initial screen with push/modal buttons
// ---------------------------------------------------------------------------

struct ScreenContent {
    title: &'static str,
    description: &'static str,
}

impl ScreenContent {
    fn new(title: &'static str, description: &'static str) -> Self {
        Self { title, description }
    }
}

impl Widget for ScreenContent {
    fn widget_type_name(&self) -> &'static str {
        "ScreenContent"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height < 2 {
            return;
        }
        let style = Style::default()
            .fg(Color::Rgb(0, 255, 163))
            .add_modifier(Modifier::BOLD);
        buf.set_string(area.x, area.y, self.title, style);
        let desc_style = Style::default().fg(Color::Gray);
        if area.height > 1 {
            buf.set_string(area.x, area.y + 1, self.description, desc_style);
        }
        let _ = ctx;
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![]
    }
}

// ---------------------------------------------------------------------------
// Modal dialog widget
// ---------------------------------------------------------------------------

struct ModalDialog {
    message: &'static str,
    own_id: Cell<Option<WidgetId>>,
}

impl ModalDialog {
    fn new(message: &'static str) -> Self {
        Self {
            message,
            own_id: Cell::new(None),
        }
    }
}

static MODAL_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        action: "dismiss",
        description: "Dismiss",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "dismiss",
        description: "Confirm",
        show: true,
    },
];

impl Widget for ModalDialog {
    fn widget_type_name(&self) -> &'static str {
        "ModalDialog"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        MODAL_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "dismiss" {
            ctx.pop_screen_deferred();
        }
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if event.downcast_ref::<Pressed>().is_some() {
            ctx.pop_screen_deferred();
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }
        let title_style = Style::default()
            .fg(Color::Rgb(0, 255, 163))
            .add_modifier(Modifier::BOLD);
        buf.set_string(area.x, area.y, "Modal Dialog", title_style);

        if area.height > 1 {
            let msg_style = Style::default().fg(Color::White);
            buf.set_string(area.x, area.y + 1, self.message, msg_style);
        }
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Press Enter or Esc to dismiss")),
            Box::new(
                Button::new("  OK  ").with_variant(ButtonVariant::Primary),
            ),
        ]
    }
}

// ---------------------------------------------------------------------------
// SecondScreen — pushed on top of MainScreen
// ---------------------------------------------------------------------------

struct SecondScreen;

static SECOND_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('b'),
        modifiers: KeyModifiers::NONE,
        action: "back",
        description: "Go back",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        action: "back",
        description: "Back",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Char('m'),
        modifiers: KeyModifiers::NONE,
        action: "open_modal",
        description: "Open modal",
        show: true,
    },
];

impl Widget for SecondScreen {
    fn widget_type_name(&self) -> &'static str {
        "SecondScreen"
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        SECOND_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "back" => ctx.pop_screen_deferred(),
            "open_modal" => {
                ctx.push_screen_deferred(Box::new(ModalScreen::new(Box::new(ModalDialog::new(
                    "You opened a modal from the second screen.",
                )))));
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Second Screen").with_subtitle("pushed on top")),
            Box::new(ScreenContent::new(
                "You are on Screen 2",
                "This screen was pushed on top of the main screen.",
            )),
            Box::new(NavButtons::back_and_modal()),
            Box::new(Footer),
        ]
    }
}

// ---------------------------------------------------------------------------
// NavButtons — reusable button group
// ---------------------------------------------------------------------------

struct NavButtons {
    show_push: bool,
    show_back: bool,
    show_modal: bool,
}

impl NavButtons {
    fn main_screen() -> Self {
        Self { show_push: true, show_back: false, show_modal: true }
    }

    fn back_and_modal() -> Self {
        Self { show_push: false, show_back: true, show_modal: true }
    }
}

impl Widget for NavButtons {
    fn widget_type_name(&self) -> &'static str {
        "NavButtons"
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        // NavButtons doesn't handle events directly — parent screens do via key_bindings.
        let _ = (event, ctx);
        EventPropagation::Continue
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let mut buttons: Vec<Box<dyn Widget>> = Vec::new();
        if self.show_push {
            buttons.push(Box::new(
                Button::new("Push Screen (n)").with_variant(ButtonVariant::Primary),
            ));
        }
        if self.show_back {
            buttons.push(Box::new(
                Button::new("Go Back (b)").with_variant(ButtonVariant::Primary),
            ));
        }
        if self.show_modal {
            buttons.push(Box::new(
                Button::new("Open Modal (m)"),
            ));
        }
        buttons
    }
}

// ---------------------------------------------------------------------------
// MainScreen
// ---------------------------------------------------------------------------

struct MainScreen;

static MAIN_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('n'),
        modifiers: KeyModifiers::NONE,
        action: "push_screen",
        description: "Push screen",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Char('m'),
        modifiers: KeyModifiers::NONE,
        action: "open_modal",
        description: "Open modal",
        show: true,
    },
];

impl Widget for MainScreen {
    fn widget_type_name(&self) -> &'static str {
        "MainScreen"
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        MAIN_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "push_screen" => {
                ctx.push_screen_deferred(Box::new(SecondScreen));
            }
            "open_modal" => {
                ctx.push_screen_deferred(Box::new(ModalScreen::new(Box::new(ModalDialog::new(
                    "You opened a modal from the main screen.",
                )))));
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Screen Stack Demo").with_subtitle("push / pop / modal")),
            Box::new(ScreenContent::new(
                "Main Screen",
                "Press 'n' to push a second screen, 'm' for a modal. Ctrl+C to quit.",
            )),
            Box::new(NavButtons::main_screen()),
            Box::new(Footer),
        ]
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(MainScreen)).with_css(CSS);
    app.run()
}
