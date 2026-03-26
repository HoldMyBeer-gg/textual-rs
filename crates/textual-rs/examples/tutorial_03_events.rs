// Tutorial 03: Events — key bindings, messages, and event handling
//
// This tutorial shows how to:
//   1. Declare key bindings with key_bindings()
//   2. Handle key actions with on_action()
//   3. Receive messages from child widgets via on_event()
//   4. Use Cell<T> for interior-mutable widget state
//   5. Use a Button widget and respond to its Pressed message
//
// Run with: cargo run --example tutorial_03_events
// Quit with: q or Ctrl+C

use std::cell::Cell;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use textual_rs::{App, Widget, Button, ButtonVariant, Label, Header, Footer};
use textual_rs::widget::context::AppContext;
use textual_rs::widget::{EventPropagation, WidgetId};
// KeyBinding declares a key → action mapping.
use textual_rs::widget::button::messages::Pressed;
use crossterm::event::{KeyCode, KeyModifiers};
use textual_rs::event::keybinding::KeyBinding;

const CSS: &str = r#"
EventScreen {
    background: #0a0a0f;
    color: #c8c8d8;
    layout-direction: vertical;
}
Header {
    height: 1;
    background: #12121a;
    color: #00d4ff;

}
Footer {
    height: 1;
    background: #12121a;
    color: #4a4a5a;

}
CounterWidget {
    flex-grow: 1;
    layout-direction: vertical;
    padding: 1;
}
Button {
    border: heavy;
    height: 3;
    min-width: 16;
    color: #00ffa3;
}
Label {
    height: 1;
    color: #c8c8d8;
}
"#;

// ---------------------------------------------------------------------------
// CounterWidget — a widget that counts button presses and key presses.
//
// Demonstrates:
//  - Cell<T> for state that changes in &self methods (on_event, on_action)
//  - on_event() to receive messages from child widgets
//  - key_bindings() + on_action() to handle keyboard shortcuts
// ---------------------------------------------------------------------------
struct CounterWidget {
    // Cell<T> provides interior mutability: change values from &self.
    // Widget trait methods receive &self, not &mut self — so Cell is required.
    count: Cell<i32>,
    // We store our own WidgetId to post messages or update focus if needed.
    own_id: Cell<Option<WidgetId>>,
}

impl CounterWidget {
    fn new() -> Self {
        Self {
            count: Cell::new(0),
            own_id: Cell::new(None),
        }
    }
}

// Declare key bindings as a static slice (zero allocation per render).
//
// Each KeyBinding has:
//   key:         the KeyCode to match
//   modifiers:   required modifier keys (NONE, CONTROL, SHIFT, ALT)
//   action:      a string sent to on_action() when the key is pressed
//   description: shown in Footer when this widget has focus
//   show:        true = display in Footer / command palette
static COUNTER_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('+'),
        modifiers: KeyModifiers::NONE,
        action: "increment",
        description: "Increment",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Char('-'),
        modifiers: KeyModifiers::NONE,
        action: "decrement",
        description: "Decrement",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Char('r'),
        modifiers: KeyModifiers::NONE,
        action: "reset",
        description: "Reset",
        show: true,
    },
];

impl Widget for CounterWidget {
    fn widget_type_name(&self) -> &'static str {
        "CounterWidget"
    }

    // can_focus() = true means Tab will bring focus here, enabling key_bindings.
    fn can_focus(&self) -> bool {
        true
    }

    fn on_mount(&self, id: WidgetId) {
        // Store our own ID so we can post messages later if needed.
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        COUNTER_BINDINGS
    }

    // on_action() is called when a key from key_bindings() is pressed.
    // The `action` string matches the `action` field in the KeyBinding.
    fn on_action(&self, action: &str, _ctx: &AppContext) {
        match action {
            "increment" => self.count.set(self.count.get() + 1),
            "decrement" => self.count.set(self.count.get() - 1),
            "reset"     => self.count.set(0),
            _ => {}
        }
    }

    // on_event() handles arbitrary events dispatched to this widget.
    // The framework dispatches events from child widgets up the tree (bubbling).
    //
    // Use downcast_ref::<MessageType>() to handle specific message types.
    // Return EventPropagation::Stop to consume the event (stops bubbling).
    // Return EventPropagation::Continue to let it keep bubbling to parents.
    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        // The Button widget emits `Pressed` when clicked (Enter or Space).
        if event.downcast_ref::<Pressed>().is_some() {
            // Button was pressed — increment the counter.
            self.count.set(self.count.get() + 1);
            // Return Stop to prevent the event from bubbling further.
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Press +/- to change count, r to reset.")),
            Box::new(Label::new("Or click the button:")),
            Box::new(Button::new("Increment").with_variant(ButtonVariant::Primary)),
        ]
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Render the count value at the top of our area.
        // (compose() children are laid out below this by the framework.)
        let count = self.count.get();
        let msg = format!("Count: {}", count);

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default()
            .add_modifier(Modifier::BOLD);

        buf.set_string(area.x, area.y, &msg, style);
    }
}

// ---------------------------------------------------------------------------
// Root screen
// ---------------------------------------------------------------------------
struct EventScreen;

impl Widget for EventScreen {
    fn widget_type_name(&self) -> &'static str {
        "EventScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Tutorial 03: Events").with_subtitle("key bindings + messages")),
            Box::new(CounterWidget::new()),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(EventScreen)).with_css(CSS);
    app.run()
}
