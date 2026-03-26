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

        // --- 3D depth borders ---
        // Extract the button's background color from the buffer (set by CSS fill_background)
        let bg = buf.cell((area.x, area.y))
            .and_then(|c| c.style().bg)
            .unwrap_or(Color::Rgb(42, 42, 62));
        let light_edge = crate::canvas::blend_color(bg, Color::Rgb(255, 255, 255), 0.25);
        let dark_edge = crate::canvas::blend_color(bg, Color::Rgb(0, 0, 0), 0.35);

        let is_pressed = self.pressed.get();

        if area.height >= 3 {
            // Top row: lighter edge (or darker when pressed)
            let top_shade = if is_pressed { dark_edge } else { light_edge };
            let top_y = area.y;
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, top_y)) {
                    cell.set_bg(top_shade);
                }
            }
            // Bottom row: darker edge (or lighter when pressed)
            let bottom_shade = if is_pressed { light_edge } else { dark_edge };
            let bottom_y = area.y + area.height - 1;
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, bottom_y)) {
                    cell.set_bg(bottom_shade);
                }
            }
        }

        // Centered label, bold
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
    fn button_3d_depth_top_lighter_bottom_darker() {
        let bg = Color::Rgb(42, 42, 62);
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = buf_with_bg(area, bg);
        let ctx = AppContext::new();
        let button = Button::new("OK");

        button.render(&ctx, area, &mut buf);

        let light = crate::canvas::blend_color(bg, Color::Rgb(255, 255, 255), 0.25);
        let dark = crate::canvas::blend_color(bg, Color::Rgb(0, 0, 0), 0.35);

        // Top row should have light edge background
        for x in 0..16 {
            let cell_bg = buf.cell((x, 0)).unwrap().bg;
            assert_eq!(cell_bg, light, "top row x={} should be light edge", x);
        }
        // Bottom row should have dark edge background
        for x in 0..16 {
            let cell_bg = buf.cell((x, 2)).unwrap().bg;
            assert_eq!(cell_bg, dark, "bottom row x={} should be dark edge", x);
        }
        // Middle row keeps original bg (unless overwritten by label)
        // Check a non-label cell (e.g. x=0, the label is centered)
        let mid_bg = buf.cell((0, 1)).unwrap().bg;
        assert_eq!(mid_bg, bg, "middle row non-label cell should keep original bg");
    }

    #[test]
    fn button_pressed_inverts_depth_shading() {
        let bg = Color::Rgb(42, 42, 62);
        let area = Rect::new(0, 0, 16, 3);
        let mut buf = buf_with_bg(area, bg);
        let ctx = AppContext::new();
        let button = Button::new("OK");

        // Simulate press
        button.pressed.set(true);
        button.render(&ctx, area, &mut buf);

        let light = crate::canvas::blend_color(bg, Color::Rgb(255, 255, 255), 0.25);
        let dark = crate::canvas::blend_color(bg, Color::Rgb(0, 0, 0), 0.35);

        // When pressed: top row = dark, bottom row = light (inverted)
        for x in 0..16 {
            let cell_bg = buf.cell((x, 0)).unwrap().bg;
            assert_eq!(cell_bg, dark, "pressed: top row x={} should be dark edge", x);
        }
        for x in 0..16 {
            let cell_bg = buf.cell((x, 2)).unwrap().bg;
            assert_eq!(cell_bg, light, "pressed: bottom row x={} should be light edge", x);
        }
    }

    #[test]
    fn button_short_height_no_depth_borders() {
        // With height < 3, no depth borders should be applied
        let bg = Color::Rgb(42, 42, 62);
        let area = Rect::new(0, 0, 16, 2);
        let mut buf = buf_with_bg(area, bg);
        let ctx = AppContext::new();
        let button = Button::new("OK");

        button.render(&ctx, area, &mut buf);

        // Non-label cells should keep original bg
        let cell_bg = buf.cell((0, 0)).unwrap().bg;
        assert_eq!(cell_bg, bg, "short button should not have depth borders");
    }
}
