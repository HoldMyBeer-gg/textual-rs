use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
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
    /// Single-frame pressed state: set true on press action, cleared after render.
    pressed: Cell<bool>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Default,
            own_id: Cell::new(None),
            pressed: Cell::new(false),
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
        "Button { border: inner; min-width: 16; height: 3; }"
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

    fn click_action(&self) -> Option<&str> {
        Some("press")
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "press" {
            self.pressed.set(true);
            if let Some(id) = self.own_id.get() {
                ctx.post_message(id, messages::Pressed);
            }
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        use ratatui::style::{Color, Modifier};

        if area.height == 0 || area.width == 0 {
            return;
        }
        let base_style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        let is_pressed = self.pressed.get();

        // Align label according to text-align CSS property (default: center)
        let text_align = self.own_id.get()
            .and_then(|id| ctx.computed_styles.get(id))
            .map(|cs| cs.text_align)
            .unwrap_or(crate::css::types::TextAlign::Center);
        let label_len = self.label.chars().count() as u16;
        let x = match text_align {
            crate::css::types::TextAlign::Center => {
                if area.width > label_len {
                    area.x + (area.width - label_len) / 2
                } else {
                    area.x
                }
            }
            crate::css::types::TextAlign::Right => {
                if area.width > label_len {
                    area.x + area.width - label_len
                } else {
                    area.x
                }
            }
            crate::css::types::TextAlign::Left => area.x,
        };
        let y = if area.height > 1 {
            area.y + area.height / 2
        } else {
            area.y
        };
        let display: String = self.label.chars().take(area.width as usize).collect();
        let label_style = if is_pressed {
            // Single-frame "flash" — invert the label style for pressed feedback
            self.pressed.set(false);
            base_style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            base_style.add_modifier(Modifier::BOLD)
        };
        buf.set_string(x, y, &display, label_style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use crate::widget::context::AppContext;
    use crate::widget::Widget;

    /// Helper: create a buffer pre-filled with a given background color.
    fn buf_with_bg(area: Rect, bg: Color) -> Buffer {
        let mut buf = Buffer::empty(area);
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                }
            }
        }
        buf
    }

    #[test]
    fn button_renders_label_centered() {
        let bg = Color::Rgb(42, 42, 62);
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = buf_with_bg(area, bg);
        let ctx = AppContext::new();
        let button = Button::new("OK");
        button.render(&ctx, area, &mut buf);

        // Middle row should contain "OK" somewhere
        let row: String = (0..16u16).map(|x| buf[(x, 1)].symbol().to_string()).collect();
        assert!(row.contains("OK"), "Button label should be rendered, got: {:?}", row.trim());
    }
}
