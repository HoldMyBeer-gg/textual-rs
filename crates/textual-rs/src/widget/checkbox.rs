use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
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

    fn click_action(&self) -> Option<&str> {
        Some("toggle")
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

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        use ratatui::style::Color;

        if area.height == 0 || area.width == 0 {
            return;
        }
        let checked = self.checked.get_untracked();
        let base_style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        // Unicode checkbox with color state: green ✓ when checked, dim ☐ when unchecked
        let (indicator, indicator_style) = if checked {
            ("✓", base_style.fg(Color::Rgb(0, 255, 163)))
        } else {
            ("☐", base_style.fg(Color::Rgb(100, 100, 110)))
        };
        buf.set_string(area.x, area.y, indicator, indicator_style);

        // Label after indicator
        if area.width > 2 {
            let label_text: String = self.label.chars().take((area.width - 2) as usize).collect();
            buf.set_string(area.x + 2, area.y, &label_text, base_style);
        }
    }
}
