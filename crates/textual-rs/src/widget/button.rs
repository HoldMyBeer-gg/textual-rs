use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;

/// Visual variant of a Button — affects border/text color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Warning,
    Error,
    Success,
}

/// Messages emitted by a Button.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the button is pressed (Enter or Space key).
    pub struct Pressed;

    impl Message for Pressed {}
}

/// A focusable button widget that emits `messages::Pressed` on Enter/Space.
pub struct Button {
    pub label: String,
    pub variant: ButtonVariant,
    own_id: Cell<Option<WidgetId>>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Default,
            own_id: Cell::new(None),
        }
    }

    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

static BUTTON_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "press",
        description: "Press",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        action: "press",
        description: "Press",
        show: false,
    },
];

impl Widget for Button {
    fn widget_type_name(&self) -> &'static str {
        "Button"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Button { border: tall; min-width: 16; height: 3; text-align: center; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        BUTTON_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "press" {
            if let Some(id) = self.own_id.get() {
                ctx.post_message(id, messages::Pressed);
            }
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        // Render label centered in the area
        let label_len = self.label.chars().count() as u16;
        let x = if area.width > label_len {
            area.x + (area.width - label_len) / 2
        } else {
            area.x
        };
        let y = if area.height > 1 {
            area.y + area.height / 2
        } else {
            area.y
        };
        let display: String = self.label.chars().take(area.width as usize).collect();
        buf.set_string(x, y, &display, Style::default());
    }
}
