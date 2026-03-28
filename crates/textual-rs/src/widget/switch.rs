//! Animated on/off toggle switch widget.
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::cell::{Cell, RefCell};
use std::time::Duration;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::animation::{ease_in_out_cubic, Tween};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a Switch.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the switch is toggled.
    pub struct Changed {
        /// New on/off state after the toggle.
        pub value: bool,
    }

    impl Message for Changed {}
}

/// A focusable switch (on/off toggle) that emits `messages::Changed`.
///
/// Renders as a sliding pill-shaped toggle inspired by Python Textual's Switch widget.
/// On: green track with knob on right. Off: dim gray track with knob on left.
/// The knob position is animated using a Tween for smooth transitions.
pub struct Switch {
    /// Current on/off state of the switch.
    pub value: Reactive<bool>,
    own_id: Cell<Option<WidgetId>>,
    /// Animation tween for knob position (0.0 = left/off, 1.0 = right/on).
    knob_tween: RefCell<Option<Tween>>,
}

impl Switch {
    /// Create a new Switch with the given initial on/off state.
    pub fn new(value: bool) -> Self {
        Self {
            value: Reactive::new(value),
            own_id: Cell::new(None),
            knob_tween: RefCell::new(None),
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
            // Start animation from current position to target
            let from = if new_val { 0.0 } else { 1.0 };
            let to = if new_val { 1.0 } else { 0.0 };
            *self.knob_tween.borrow_mut() = Some(Tween::new(
                from,
                to,
                Duration::from_millis(200),
                ease_in_out_cubic,
            ));
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
        let _base_style = self
            .own_id
            .get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        // Textual-style sliding pill switch using block characters
        let track_width = area.width.min(8) as usize;
        let knob_width = 2;
        let track_inner = track_width.saturating_sub(2); // inside the caps

        // Determine knob position from tween or final state.
        // When skip_animations is true (e.g. in tests), snap immediately to the
        // target position for deterministic rendering.
        let knob_fraction = {
            let tween = self.knob_tween.borrow();
            if ctx.skip_animations {
                if on {
                    1.0
                } else {
                    0.0
                }
            } else if let Some(ref tw) = *tween {
                if tw.is_complete() {
                    if on {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    tw.value()
                }
            } else {
                if on {
                    1.0
                } else {
                    0.0
                }
            }
        };

        // Clean up completed tweens
        {
            let should_clear = self
                .knob_tween
                .borrow()
                .as_ref()
                .is_some_and(|tw| tw.is_complete());
            if should_clear {
                *self.knob_tween.borrow_mut() = None;
            }
        }

        // Interpolate colors based on knob_fraction
        let (track_color, knob_color, track_bg) = {
            let on_track = (0u8, 80u8, 50u8);
            let on_knob = (0u8, 255u8, 163u8);
            let on_bg = (0u8, 60u8, 40u8);
            let off_track = (50u8, 50u8, 60u8);
            let off_knob = (120u8, 120u8, 130u8);
            let off_bg = (30u8, 30u8, 38u8);

            let f = knob_fraction as f32;
            let lerp = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * f) as u8 };

            (
                Color::Rgb(
                    lerp(off_track.0, on_track.0),
                    lerp(off_track.1, on_track.1),
                    lerp(off_track.2, on_track.2),
                ),
                Color::Rgb(
                    lerp(off_knob.0, on_knob.0),
                    lerp(off_knob.1, on_knob.1),
                    lerp(off_knob.2, on_knob.2),
                ),
                Color::Rgb(
                    lerp(off_bg.0, on_bg.0),
                    lerp(off_bg.1, on_bg.1),
                    lerp(off_bg.2, on_bg.2),
                ),
            )
        };

        // Knob position interpolated
        let max_knob_start = track_inner.saturating_sub(knob_width);
        let knob_start = (knob_fraction * max_knob_start as f64).round() as usize;

        let mut x = area.x;

        // Left cap
        if x < area.x + area.width {
            let style = ratatui::style::Style::default()
                .fg(track_color)
                .bg(track_bg);
            buf.set_string(x, area.y, "\u{2590}", style); // ▐
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
                buf.set_string(x, area.y, "\u{2588}", style); // █
            } else {
                let style = ratatui::style::Style::default()
                    .fg(track_color)
                    .bg(track_bg);
                buf.set_string(x, area.y, "\u{2501}", style); // ━
            }
            x += 1;
        }

        // Right cap
        if x < area.x + area.width {
            let style = ratatui::style::Style::default()
                .fg(track_color)
                .bg(track_bg);
            buf.set_string(x, area.y, "\u{258C}", style); // ▌
        }
    }
}
