//! Progress bar widget with determinate and indeterminate (animated) modes.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::cell::Cell;

use super::context::AppContext;
use super::Widget;
use crate::canvas;
use crate::reactive::Reactive;

/// A progress bar widget that fills proportionally to a progress value (0.0–1.0).
/// Uses sub-cell block characters for smooth rendering (8 levels per cell).
/// When progress is `None`, the bar renders in indeterminate (animated) mode.
pub struct ProgressBar {
    /// Current progress value. `Some(0.0..=1.0)` for determinate; `None` for indeterminate.
    pub progress: Reactive<Option<f64>>,
    tick: Cell<u8>,
}

impl ProgressBar {
    /// Create a new determinate ProgressBar with the given progress value (clamped to `0.0..=1.0`).
    pub fn new(progress: f64) -> Self {
        Self {
            progress: Reactive::new(Some(progress.clamp(0.0, 1.0))),
            tick: Cell::new(0),
        }
    }

    /// Create an indeterminate ProgressBar that renders a bouncing animation.
    pub fn indeterminate() -> Self {
        Self {
            progress: Reactive::new(None),
            tick: Cell::new(0),
        }
    }
}

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

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Get colors from computed style
        let (fill_color, empty_color) = self.own_id_style(ctx);

        match self.progress.get_untracked() {
            Some(p) => {
                // Paint a half-block gradient on the empty (unfilled) portion of the track.
                // This gives the track a subtle depth/3D effect using sub-cell shading.
                // For single-row bars, half_block_cell packs two color shades into one cell.
                let track_top = canvas::blend_color(empty_color, Color::Rgb(50, 50, 50), 0.3);
                let track_bottom = empty_color;
                let filled_cells =
                    ((p.clamp(0.0, 1.0) * (area.width as f64) * 8.0).round() as usize) / 8;
                for col in (filled_cells as u16)..area.width {
                    canvas::half_block_cell(buf, area.x + col, area.y, track_top, track_bottom);
                }

                // Overlay the eighth-block progress fill on top
                canvas::progress_bar(buf, area.x, area.y, area.width, p, fill_color, empty_color);
            }
            None => {
                // Indeterminate: bouncing block animation
                let width = area.width;
                let block_len: u16 = 3;
                let max_start = width.saturating_sub(block_len);
                let period = (max_start * 2).max(1) as u8;
                let tick = self.tick.get();
                let pos = (tick % period) as u16;
                let block_start = if pos <= max_start {
                    pos
                } else {
                    max_start.saturating_sub(pos - max_start)
                };
                self.tick.set(tick.wrapping_add(1));

                for col in 0..width {
                    let in_block = col >= block_start && col < block_start + block_len;
                    if let Some(cell) = buf.cell_mut((area.x + col, area.y)) {
                        if in_block {
                            cell.set_symbol(canvas::FULL_BLOCK);
                            cell.set_fg(fill_color);
                            cell.set_bg(empty_color);
                        } else {
                            cell.set_symbol(" ");
                            cell.set_bg(empty_color);
                        }
                    }
                }
            }
        }
    }
}

impl ProgressBar {
    fn own_id_style(&self, ctx: &AppContext) -> (Color, Color) {
        // Try to get colors from computed CSS style
        let default_fill = Color::Rgb(0, 255, 163); // accent green
        let default_empty = Color::Rgb(74, 74, 90); // muted
                                                    // We don't have own_id stored, so use defaults.
                                                    // TODO: wire up own_id to read computed style colors
        let _ = ctx;
        (default_fill, default_empty)
    }
}
