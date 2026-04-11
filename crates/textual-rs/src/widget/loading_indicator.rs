//! Loading spinner widget and overlay helper for showing async loading state.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::context::AppContext;
use super::Widget;

/// Braille spinner frames — 8 characters, one full rotation.
/// At 30fps app tick with frame = tick/2, animation runs at ~15fps (one cycle per ~533ms).
/// These are: ⣾ ⣽ ⣻ ⢿ ⡿ ⣟ ⣯ ⣷
const SPINNER_FRAMES: [char; 8] = [
    '\u{28FE}', '\u{28FD}', '\u{28FB}', '\u{283F}', '\u{285F}', '\u{289F}', '\u{28AF}', '\u{28B7}',
];

/// A standalone loading spinner widget.
///
/// Renders a braille spinner animation centered in its area.
/// When `ctx.skip_animations` is true, renders static "Loading..." text instead,
/// which is safe for snapshot tests.
///
/// # Default CSS
/// ```css
/// LoadingIndicator { width: 100%; height: 100%; min-height: 1; }
/// ```
///
/// # Example
/// ```no_run
/// # use textual_rs::LoadingIndicator;
/// # use textual_rs::widget::layout::Vertical;
/// let layout = Vertical::with_children(vec![
///     Box::new(LoadingIndicator::new()),
/// ]);
/// ```
pub struct LoadingIndicator;

impl LoadingIndicator {
    /// Create a new LoadingIndicator.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for LoadingIndicator {
    fn widget_type_name(&self) -> &'static str {
        "LoadingIndicator"
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "LoadingIndicator { width: 100%; height: 100%; min-height: 1; }"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let style = Style::default().fg(Color::Rgb(0, 255, 163)); // accent green

        if ctx.skip_animations {
            // Deterministic: static text for snapshot tests
            let text = "Loading...";
            let x = area.x + area.width.saturating_sub(text.len() as u16) / 2;
            let y = area.y + area.height / 2;
            buf.set_string(x, y, text, style);
            return;
        }

        // Animated: use spinner_tick from AppContext for synchronized animation
        let frame_idx = (ctx.spinner_tick.get() / 2) as usize % SPINNER_FRAMES.len();
        let ch = SPINNER_FRAMES[frame_idx];
        let x = area.x + area.width / 2;
        let y = area.y + area.height / 2;
        buf.set_string(x, y, ch.to_string(), style);
    }
}

/// Draw a loading spinner overlay on top of a widget's area.
///
/// Called by `render_widget_tree` when a widget's ID is in `ctx.loading_widgets`.
/// Fills the area with a dark semi-opaque background and centers a spinner character.
///
/// `tick` is `ctx.spinner_tick.get()`. `skip_animations` gates deterministic mode.
pub fn draw_loading_spinner_overlay(area: Rect, buf: &mut Buffer, tick: u8, skip_animations: bool) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    // Fill area with dark background to dim the underlying widget
    let bg_style = Style::default().bg(Color::Rgb(20, 20, 28));
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_style(bg_style);
            }
        }
    }

    // Center the spinner or static text
    let fg_style = Style::default()
        .fg(Color::Rgb(0, 255, 163))
        .bg(Color::Rgb(20, 20, 28));

    if skip_animations {
        let text = "Loading...";
        let x = area.x + area.width.saturating_sub(text.len() as u16) / 2;
        let y = area.y + area.height / 2;
        buf.set_string(x, y, text, fg_style);
    } else {
        let frame_idx = (tick / 2) as usize % SPINNER_FRAMES.len();
        let ch = SPINNER_FRAMES[frame_idx];
        let x = area.x + area.width / 2;
        let y = area.y + area.height / 2;
        buf.set_string(x, y, ch.to_string(), fg_style);
    }
}
