use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::context::AppContext;
use super::Widget;
use crate::reactive::Reactive;

/// A progress bar widget that fills proportionally to a progress value (0.0–1.0).
/// When progress is `None`, the bar renders in indeterminate (animated) mode.
pub struct ProgressBar {
    pub progress: Reactive<Option<f64>>,
    /// Tick counter for indeterminate animation. Increments each render.
    tick: Cell<u8>,
}

impl ProgressBar {
    /// Create a determinate progress bar with the given progress (0.0–1.0).
    pub fn new(progress: f64) -> Self {
        Self {
            progress: Reactive::new(Some(progress.clamp(0.0, 1.0))),
            tick: Cell::new(0),
        }
    }

    /// Create an indeterminate progress bar (animated bouncing block).
    pub fn indeterminate() -> Self {
        Self {
            progress: Reactive::new(None),
            tick: Cell::new(0),
        }
    }
}

const FILLED: char = '█';
const EMPTY: char = '░';
const INDETERMINATE_BLOCK: &str = "███";
const INDETERMINATE_LEN: u16 = 3;

impl Widget for ProgressBar {
    fn widget_type_name(&self) -> &'static str {
        "ProgressBar"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "ProgressBar { height: 1; width: 1fr; }"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let width = area.width as usize;

        match self.progress.get_untracked() {
            Some(p) => {
                // Determinate: fill width proportionally
                let filled = (p.clamp(0.0, 1.0) * width as f64).round() as usize;
                let filled = filled.min(width);
                let text: String = std::iter::repeat(FILLED)
                    .take(filled)
                    .chain(std::iter::repeat(EMPTY).take(width - filled))
                    .collect();
                buf.set_string(area.x, area.y, &text, Style::default());
            }
            None => {
                // Indeterminate: bouncing block animation
                let total = width as u16;
                let max_start = total.saturating_sub(INDETERMINATE_LEN);

                // Bounce: go forward 0..max_start, then back max_start..0
                let period = (max_start * 2).max(1) as u8;
                let tick = self.tick.get();
                let pos_in_period = (tick % period) as u16;
                let block_start = if pos_in_period <= max_start {
                    pos_in_period
                } else {
                    // Reverse direction
                    max_start.saturating_sub(pos_in_period - max_start)
                };

                // Advance tick for next render
                self.tick.set(tick.wrapping_add(1));

                // Render: spaces before block, block chars, spaces after
                let block_start = block_start as usize;
                let block_end = (block_start + INDETERMINATE_LEN as usize).min(width);
                let mut text = String::with_capacity(width);
                for _ in 0..block_start {
                    text.push(' ');
                }
                for _ in block_start..block_end {
                    text.push(FILLED);
                }
                for _ in block_end..width {
                    text.push(' ');
                }
                buf.set_string(area.x, area.y, &text, Style::default());
            }
        }
    }
}
