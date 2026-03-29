//! OSC 8 hyperlink support for textual-rs widgets.
//!
//! OSC 8 is a terminal escape sequence that creates clickable hyperlinks in
//! terminals that support it: iTerm2, kitty, Ghostty, WezTerm, Windows Terminal,
//! and most modern terminal emulators.
//!
//! ## How it works
//!
//! After rendering text to the ratatui [`Buffer`] normally, this module patches
//! the first and last cells of the text to embed OSC 8 open/close sequences in
//! the cell symbol. The ratatui crossterm backend writes cell symbols verbatim,
//! so the terminal receives the escape sequences and renders the text as a
//! clickable hyperlink.
//!
//! ## Usage in a custom widget
//!
//! ```no_run
//! use textual_rs::hyperlink::render_hyperlink;
//! use ratatui::style::Style;
//!
//! # use ratatui::buffer::Buffer;
//! # use ratatui::layout::Rect;
//! # fn example(buf: &mut Buffer, area: Rect) {
//! let style = Style::default();
//! render_hyperlink(buf, area.x, area.y, "https://github.com/owner/repo/commit/abc1234", "abc1234", style);
//! # }
//! ```
//!
//! ## Fallback
//!
//! Terminals that don't support OSC 8 ignore the escape sequences and display
//! the visible text normally. No capability detection is required.

use ratatui::{buffer::Buffer, layout::Position, style::Style};
use unicode_width::UnicodeWidthStr;

/// Render `label` at `(x, y)` as a clickable OSC 8 hyperlink pointing to `url`.
///
/// Equivalent to `buf.set_string(x, y, label, style)` but wraps the text in
/// OSC 8 open/close escape sequences embedded in the first and last cells.
///
/// Returns the number of terminal columns consumed (same as the display width
/// of `label`).
///
/// Terminals that don't support OSC 8 display the label as plain text.
pub fn render_hyperlink(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    url: &str,
    label: &str,
    style: Style,
) -> u16 {
    let width = UnicodeWidthStr::width(label) as u16;
    if width == 0 || label.is_empty() {
        return 0;
    }

    // Render the label into the buffer normally
    buf.set_string(x, y, label, style);

    let open = format!("\x1b]8;;{url}\x1b\\");
    let close = "\x1b]8;;\x1b\\";

    // Patch the first cell: prepend OSC 8 open before the visible character
    if let Some(cell) = buf.cell_mut(Position { x, y }) {
        let sym = cell.symbol().to_string();
        cell.set_symbol(&format!("{open}{sym}"));
    }

    let last_x = x + width - 1;
    if last_x == x {
        // Single-cell label — append close to the same cell (already has open+char)
        if let Some(cell) = buf.cell_mut(Position { x, y }) {
            let sym = cell.symbol().to_string();
            cell.set_symbol(&format!("{sym}{close}"));
        }
    } else {
        // Multi-cell label — append close to the last cell
        if let Some(cell) = buf.cell_mut(Position { x: last_x, y }) {
            let sym = cell.symbol().to_string();
            cell.set_symbol(&format!("{sym}{close}"));
        }
    }

    width
}
