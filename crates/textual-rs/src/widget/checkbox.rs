use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a Checkbox.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the checkbox is toggled.
    pub struct Changed {
        pub checked: bool,
    }

    impl Message for Changed {}
}

/// A focusable checkbox that toggles a boolean state and emits `messages::Changed`.
pub struct Checkbox {
    pub checked: Reactive<bool>,
    pub label: String,
    own_id: Cell<Option<WidgetId>>,
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            checked: Reactive::new(checked),
            label: label.into(),
            own_id: Cell::new(None),
        }
    }
}

static CHECKBOX_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        action: "toggle",
        description: "Toggle",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "toggle",
        description: "Toggle",
        show: false,
    },
];

impl Widget for Checkbox {
    fn widget_type_name(&self) -> &'static str {
        "Checkbox"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Checkbox { height: 1; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        CHECKBOX_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "toggle" {
            let new_val = !self.checked.get_untracked();
            self.checked.set(new_val);
            if let Some(id) = self.own_id.get() {
                ctx.post_message(id, messages::Changed { checked: new_val });
            }
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        // Use get_untracked() to avoid reactive tracking loops in render
        let checked = self.checked.get_untracked();
        let indicator = if checked { "[X]" } else { "[ ]" };
        let text = format!("{} {}", indicator, self.label);
        let display: String = text.chars().take(area.width as usize).collect();
        buf.set_string(area.x, area.y, &display, Style::default());
    }
}
