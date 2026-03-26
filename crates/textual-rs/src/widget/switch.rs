use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
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
/// Renders as a sliding pill-shaped toggle inspired by Python Textual's Switch widget.
/// On: green track with knob on right. Off: dim gray track with knob on left.
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

    fn click_action(&self) -> Option<&str> {
        Some("toggle")
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

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let on = self.value.get_untracked();
        let _base_style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        // Textual-style sliding pill switch using block characters
        // Track width = 8 chars: ▐████████▌ with knob position
        let track_width = area.width.min(8) as usize;
        let knob_width = 2;
        let track_inner = track_width.saturating_sub(2); // inside the ▐...▌ caps

        let (track_color, knob_color, track_bg) = if on {
            (Color::Rgb(0, 80, 50), Color::Rgb(0, 255, 163), Color::Rgb(0, 60, 40))
        } else {
            (Color::Rgb(50, 50, 60), Color::Rgb(120, 120, 130), Color::Rgb(30, 30, 38))
        };

        // Knob position: left when off, right when on
        let knob_start = if on {
            track_inner.saturating_sub(knob_width)
        } else {
            0
        };

        let mut x = area.x;

        // Left cap
        if x < area.x + area.width {
            let style = ratatui::style::Style::default().fg(track_color).bg(track_bg);
            buf.set_string(x, area.y, "▐", style);
            x += 1;
        }

        // Track interior with knob
        for i in 0..track_inner {
            if x >= area.x + area.width {
                break;
            }
            let in_knob = i >= knob_start && i < knob_start + knob_width;
            if in_knob {
                let style = ratatui::style::Style::default().fg(knob_color).bg(track_bg);
                buf.set_string(x, area.y, "█", style);
            } else {
                let style = ratatui::style::Style::default().fg(track_color).bg(track_bg);
                buf.set_string(x, area.y, "━", style);
            }
            x += 1;
        }

        // Right cap
        if x < area.x + area.width {
            let style = ratatui::style::Style::default().fg(track_color).bg(track_bg);
            buf.set_string(x, area.y, "▌", style);
        }
    }
}
