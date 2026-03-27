// Tutorial 04: Reactive — automatic re-rendering with Reactive<T>
//
// This tutorial shows how to:
//   1. Use Reactive<T> to hold widget state that triggers re-renders
//   2. Read Reactive values safely in render() with get_untracked()
//   3. Update Reactive values from on_event() handlers
//   4. Use Input widget messages to drive reactive state changes
//
// Run with: cargo run --example tutorial_04_reactive
// Quit with: q or Ctrl+C

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use textual_rs::{App, Widget, Input, Label, Header, Footer};
use textual_rs::widget::context::AppContext;
use textual_rs::widget::{EventPropagation, WidgetId};
use textual_rs::widget::input::messages::Changed;

// Reactive<T> wraps a value and notifies the framework when it changes,
// triggering a re-render automatically. Import from the reactive module.
use textual_rs::reactive::Reactive;

const CSS: &str = r#"
ReactiveScreen {
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
EchoWidget {
    flex-grow: 1;
    layout-direction: vertical;
    padding: 1;
}
Input {
    border: tall $primary;
    height: 3;
    color: $foreground;
}
Label {
    height: 1;
    color: $accent;
}
"#;

// ---------------------------------------------------------------------------
// EchoWidget — echoes what the user types into the Input field.
//
// The `echo` field is a Reactive<String>. When it changes, the framework
// triggers a re-render automatically (via the reactive effect system).
// ---------------------------------------------------------------------------
struct EchoWidget {
    // Reactive<T> holds a value and notifies dependents when it changes.
    // T must be Clone + PartialEq + Send + Sync + 'static.
    echo: Reactive<String>,
    own_id: Cell<Option<WidgetId>>,
}

// Cell for interior mutability on own_id.
use std::cell::Cell;

impl EchoWidget {
    fn new() -> Self {
        Self {
            // Reactive::new(value) creates a signal with an initial value.
            echo: Reactive::new(String::from("(nothing typed yet)")),
            own_id: Cell::new(None),
        }
    }
}

impl Widget for EchoWidget {
    fn widget_type_name(&self) -> &'static str {
        "EchoWidget"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Type in the field below:")),
            // Input widget emits messages::Changed on every keystroke.
            Box::new(Input::new("Type here...")),
        ]
    }

    // on_event() receives messages from child widgets (bubbled up the tree).
    // We listen for Input::Changed to update our reactive echo value.
    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        // Input emits `Changed { value, valid }` on every keystroke.
        if let Some(changed) = event.downcast_ref::<Changed>() {
            // Reactive::set() updates the value and schedules a re-render.
            self.echo.set(changed.value.clone());
        }
        EventPropagation::Continue
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // get_untracked() reads the Reactive value without creating a tracking
        // dependency. Always use get_untracked() inside render() to avoid
        // reactive tracking loops (the reactive graph is for signals outside renders).
        let current = self.echo.get_untracked();

        let label = format!("You typed: {}", current);
        let display: String = label.chars().take(area.width as usize).collect();

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        buf.set_string(area.x, area.y, &display, style);
    }
}

// ---------------------------------------------------------------------------
// Root screen
// ---------------------------------------------------------------------------
struct ReactiveScreen;

impl Widget for ReactiveScreen {
    fn widget_type_name(&self) -> &'static str {
        "ReactiveScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Tutorial 04: Reactive").with_subtitle("automatic re-rendering")),
            Box::new(EchoWidget::new()),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(ReactiveScreen)).with_css(CSS);
    app.run()
}
