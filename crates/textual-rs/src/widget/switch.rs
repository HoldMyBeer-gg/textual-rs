use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a Switch.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the switch is toggled.
    pub struct Changed {
        pub value: bool,
    }

    impl Message for Changed {}
}

/// A focusable switch (on/off toggle) that emits `messages::Changed`.
///
/// Renders as `━━━◉` (on) or `◉━━━` (off) similar to Python Textual's Switch widget.
pub struct Switch {
    pub value: Reactive<bool>,
    own_id: Cell<Option<WidgetId>>,
}

impl Switch {
    pub fn new(value: bool) -> Self {
        Self {
            value: Reactive::new(value),
            own_id: Cell::new(None),
        }
    }
}

static SWITCH_BINDINGS: &[KeyBinding] = &[
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

impl Widget for Switch {
    fn widget_type_name(&self) -> &'static str {
        "Switch"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Switch { height: 1; width: 8; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        SWITCH_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "toggle" {
            let new_val = !self.value.get_untracked();
            self.value.set(new_val);
            if let Some(id) = self.own_id.get() {
                ctx.post_message(id, messages::Changed { value: new_val });
            }
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        // Use get_untracked() to avoid reactive tracking loops in render
        // On: "━━━◉" | Off: "◉━━━"  (matching Python Textual's style)
        let on = self.value.get_untracked();
        let indicator = if on { "━━━◉" } else { "◉━━━" };
        let display: String = indicator.chars().take(area.width as usize).collect();
        buf.set_string(area.x, area.y, &display, Style::default());
    }
}
