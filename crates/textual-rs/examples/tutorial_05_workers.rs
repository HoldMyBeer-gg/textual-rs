// Tutorial 05: Workers — running async tasks without blocking the UI
//
// This tutorial shows how to:
//   1. Spawn a background worker with ctx.run_worker()
//   2. Show a "Loading..." state while the worker runs
//   3. Receive the WorkerResult<T> in on_event() and update state
//   4. Use Reactive<String> to reflect the result in the UI
//
// Run with: cargo run --example tutorial_05_workers
// Quit with: q or Ctrl+C

use std::cell::Cell;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use textual_rs::{App, Widget, Button, ButtonVariant, Label, Header, Footer};
use textual_rs::widget::context::AppContext;
use textual_rs::widget::{EventPropagation, WidgetId};
use textual_rs::reactive::Reactive;

// WorkerResult<T> is the message delivered to on_event() when a worker completes.
// T is whatever type the worker's future returns.
use textual_rs::WorkerResult;

use crossterm::event::{KeyCode, KeyModifiers};
use textual_rs::event::keybinding::KeyBinding;

const CSS: &str = r#"
WorkerScreen {
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
WorkerDemo {
    flex-grow: 1;
    layout-direction: vertical;
    padding: 1;
}
Button {
    border: heavy;
    height: 3;
    min-width: 20;
    color: #00ffa3;
}
Label {
    height: 1;
    color: #c8c8d8;
}
"#;

// ---------------------------------------------------------------------------
// WorkerDemo — a widget that spawns a background task.
//
// When the user presses "Start Worker", a simulated async task runs for
// 2 seconds (tokio::time::sleep). During that time the UI shows "Loading...".
// When the worker completes, WorkerResult<String> is delivered to on_event()
// and the result is displayed.
// ---------------------------------------------------------------------------
struct WorkerDemo {
    // Reactive state for the displayed status message.
    status: Reactive<String>,
    // True while the worker is running.
    loading: Reactive<bool>,
    own_id: Cell<Option<WidgetId>>,
}

impl WorkerDemo {
    fn new() -> Self {
        Self {
            status: Reactive::new(String::from("Press 'Start Worker' to run a background task.")),
            loading: Reactive::new(false),
            own_id: Cell::new(None),
        }
    }
}

// Key binding to start the worker manually (pressing 's').
static WORKER_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char('s'),
        modifiers: KeyModifiers::NONE,
        action: "start_worker",
        description: "Start Worker",
        show: true,
    },
];

impl Widget for WorkerDemo {
    fn widget_type_name(&self) -> &'static str {
        "WorkerDemo"
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
        WORKER_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "start_worker" {
            self.spawn_work(ctx);
        }
    }

    // on_event() receives the WorkerResult<String> when the worker completes.
    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        // Downcast to WorkerResult<String> — the type must match what the worker returns.
        if let Some(result) = event.downcast_ref::<WorkerResult<String>>() {
            // Worker finished — update reactive state.
            self.loading.set(false);
            self.status.set(result.value.clone());
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Background Worker Demo")),
            Box::new(Label::new("")),
            Box::new(Label::new("Workers run async tasks without blocking the UI.")),
            Box::new(Label::new("The event loop stays responsive while work happens.")),
            Box::new(Label::new("")),
            Box::new(Button::new("Start Worker").with_variant(ButtonVariant::Primary)),
        ]
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let is_loading = self.loading.get_untracked();
        let status = self.status.get_untracked();

        // Show "Loading..." with animation indicator if worker is running.
        let display_text = if is_loading {
            "Loading...".to_string()
        } else {
            format!("Result: {}", status)
        };

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        let display_style = if is_loading {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        };

        let display: String = display_text.chars().take(area.width as usize).collect();
        buf.set_string(area.x, area.y, &display, display_style);
    }
}

impl WorkerDemo {
    // Spawn the background worker.
    //
    // ctx.run_worker(source_id, future) schedules an async task on the Tokio LocalSet.
    // When the future completes, WorkerResult { source_id, value } is delivered
    // to the source widget's on_event() method via the message queue.
    //
    // The worker future must be 'static (no borrowed references to self).
    // Use owned data (clone strings, copy numbers) to pass values into the future.
    fn spawn_work(&self, ctx: &AppContext) {
        let Some(id) = self.own_id.get() else { return };

        // Already running — don't start another.
        if self.loading.get_untracked() {
            return;
        }

        self.loading.set(true);
        self.status.set(String::from("Worker started..."));

        // ctx.run_worker(source_id, async_future)
        //
        // The future runs on the tokio LocalSet (same thread as the UI).
        // It doesn't need to be Send. The result T must be Send + 'static.
        ctx.run_worker(id, async {
            // Simulate a 2-second background task (e.g., HTTP request, file read).
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            // Return a result. This becomes WorkerResult<String>.value.
            String::from("Done! Background task completed in 2 seconds.")
        });
    }
}

// ---------------------------------------------------------------------------
// Root screen — also handles the Button press from compose().
// ---------------------------------------------------------------------------
struct WorkerScreen;

impl Widget for WorkerScreen {
    fn widget_type_name(&self) -> &'static str {
        "WorkerScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Header::new("Tutorial 05: Workers").with_subtitle("async background tasks")),
            Box::new(WorkerDemo::new()),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(WorkerScreen)).with_css(CSS);
    app.run()
}
