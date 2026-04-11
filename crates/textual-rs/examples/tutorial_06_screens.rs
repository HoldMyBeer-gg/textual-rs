// Tutorial 06: Screens — push/pop navigation and modal dialogs
//
// This tutorial shows how to:
//   1. Push a new screen onto the screen stack for navigation
//   2. Pop a screen to go back
//   3. Push a ModalScreen that blocks input to screens below
//   4. Use push_screen_wait + pop_screen_with for typed modal results
//
// Run with: cargo run --example tutorial_06_screens
// Quit with: q or Ctrl+C

use std::cell::Cell;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use textual_rs::event::keybinding::KeyBinding;
use textual_rs::reactive::Reactive;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::screen::ModalScreen;
use textual_rs::widget::{EventPropagation, WidgetId};
use textual_rs::{App, Button, ButtonVariant, Footer, Header, Label, Widget, WorkerResult};

// ---------------------------------------------------------------------------
// CSS
// ---------------------------------------------------------------------------

const CSS: &str = r#"
ScreenDemoScreen {
    background: $background;
    color: $foreground;
    layout-direction: vertical;
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
MainContent {
    flex-grow: 1;
    layout-direction: vertical;
    padding: 1;
}
NavScreen {
    background: $surface;
    color: $foreground;
    layout-direction: vertical;
}
ConfirmDialog {
    layout-direction: vertical;
    width: 50;
    height: 8;
    background: $panel;
    border: mcgugan-box $primary;
    padding: 1;
    margin: 8 15;
}
ModalScreen {
    background: transparent;
}
Button {
    border: mcgugan-box $accent;
    height: 3;
    min-width: 24;
    color: $accent;
    margin: 0 0 1 0;
}
Label {
    height: 1;
    color: $foreground;
}
"#;

// ---------------------------------------------------------------------------
// MainContent — manages the demo content, result display, and modal push
// ---------------------------------------------------------------------------
//
// MainContent owns the `own_id` needed for ctx.run_worker() after
// push_screen_wait. It handles both key bindings and button actions.

struct MainContent {
    result_label: Reactive<String>,
    own_id: Cell<Option<WidgetId>>,
}

impl MainContent {
    fn new() -> Self {
        Self {
            result_label: Reactive::new(String::from("No result yet")),
            own_id: Cell::new(None),
        }
    }
}

static MAIN_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('n'),
        modifiers: KeyModifiers::NONE,
        action: "push_nav",
        description: "Push nav screen",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Char('m'),
        modifiers: KeyModifiers::NONE,
        action: "push_modal",
        description: "Push modal",
        show: true,
    },
];

impl Widget for MainContent {
    fn widget_type_name(&self) -> &'static str {
        "MainContent"
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
        MAIN_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "push_nav" => {
                // Push a regular (non-modal) navigation screen.
                // When the user pops it, focus returns to this widget automatically.
                ctx.push_screen_deferred(Box::new(NavScreen));
            }
            "push_modal" => {
                // push_screen_wait returns a oneshot receiver that resolves when
                // the modal calls pop_screen_with(value). We bridge the async gap
                // by spawning a worker to await the receiver, then deliver the
                // result back as WorkerResult<bool> via the message queue.
                let Some(self_id) = self.own_id.get() else {
                    return;
                };
                let rx = ctx.push_screen_wait(Box::new(ModalScreen::new(Box::new(ConfirmDialog))));
                ctx.run_worker(self_id, async move {
                    match rx.await {
                        Ok(boxed) => *boxed.downcast::<bool>().unwrap_or(Box::new(false)),
                        Err(_) => false,
                    }
                });
            }
            _ => {}
        }
    }

    // WorkerResult<bool> arrives here after the modal is dismissed via pop_screen_with.
    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        if let Some(wr) = event.downcast_ref::<WorkerResult<bool>>() {
            let msg = if wr.value {
                "You chose: OK"
            } else {
                "You chose: Cancel"
            };
            self.result_label.set(msg.to_string());
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Screen Stack Demo")),
            Box::new(Label::new("")),
            Box::new(Label::new(
                "Push a navigation screen, or open a modal dialog.",
            )),
            Box::new(Label::new("")),
            Box::new(Label::new(
                self.result_label.get_untracked().as_str().to_string(),
            )),
            Box::new(Label::new("")),
            Box::new(
                Button::new("Push Navigation Screen (n)").with_variant(ButtonVariant::Primary),
            ),
            Box::new(Button::new("Push Modal Dialog (m)")),
        ]
    }
}

// ---------------------------------------------------------------------------
// ScreenDemoScreen — root screen composing Header, MainContent, Footer
// ---------------------------------------------------------------------------

struct ScreenDemoScreen;

impl Widget for ScreenDemoScreen {
    fn widget_type_name(&self) -> &'static str {
        "ScreenDemoScreen"
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Tutorial 06: Screens").with_subtitle("push / pop / modal")),
            Box::new(MainContent::new()),
            Box::new(Footer),
        ]
    }
}

// ---------------------------------------------------------------------------
// NavScreen — non-modal navigation screen pushed on top of the main screen
// ---------------------------------------------------------------------------

struct NavScreen;

static NAV_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('b'),
        modifiers: KeyModifiers::NONE,
        action: "go_back",
        description: "Go back",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        action: "go_back",
        description: "Back",
        show: true,
    },
];

impl Widget for NavScreen {
    fn widget_type_name(&self) -> &'static str {
        "NavScreen"
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        NAV_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "go_back" {
            ctx.pop_screen_deferred();
        }
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Navigation Screen").with_subtitle("pushed on top")),
            Box::new(NavContent),
            Box::new(Footer),
        ]
    }
}

// ---------------------------------------------------------------------------
// NavContent — the content area of the pushed navigation screen
// ---------------------------------------------------------------------------

struct NavContent;

impl Widget for NavContent {
    fn widget_type_name(&self) -> &'static str {
        "NavContent"
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("")),
            Box::new(Label::new("This is a pushed navigation screen.")),
            Box::new(Label::new(
                "Keyboard focus has moved here from the main screen.",
            )),
            Box::new(Label::new(
                "Press 'b' or Esc to pop back, or press the button below.",
            )),
            Box::new(Label::new("")),
            Box::new(Button::new("Go Back (b)").with_variant(ButtonVariant::Primary)),
        ]
    }
}

// ---------------------------------------------------------------------------
// ConfirmDialog — inner widget for the ModalScreen
//
// OK delivers `true` via pop_screen_with; Cancel delivers `false`.
// The main screen's MainContent widget receives the result as
// WorkerResult<bool> in its on_event() handler.
// ---------------------------------------------------------------------------

struct ConfirmDialog;

impl Widget for ConfirmDialog {
    fn widget_type_name(&self) -> &'static str {
        "ConfirmDialog"
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "ok" => ctx.pop_screen_with(true),
            "cancel" => ctx.pop_screen_with(false),
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Are you sure?")),
            Box::new(Label::new("")),
            Box::new(Button::new("  OK  ").with_variant(ButtonVariant::Primary)),
            Box::new(Button::new("Cancel")),
        ]
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(ScreenDemoScreen)).with_css(CSS);
    app.run()
}
