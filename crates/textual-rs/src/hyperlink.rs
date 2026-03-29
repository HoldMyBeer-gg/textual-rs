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
//! ## High-level API: `LinkedSpan` and `LinkedLine`
//!
//! Most widgets accept `LinkedLine` (a `Vec<LinkedSpan>`) for their text content.
//! A `LinkedSpan` is a styled text fragment with an optional URL:
//!
//! ```no_run
//! use textual_rs::hyperlink::{LinkedSpan, LinkedLine};
//! use ratatui::style::{Color, Style};
//!
//! // Plain span (no link)
//! let plain = LinkedSpan::plain("INFO ");
//!
//! // Linked span
//! let link = LinkedSpan::linked("GitHub", "https://github.com");
//!
//! // Styled linked span
//! let styled = LinkedSpan {
//!     text: "docs".into(),
//!     style: Style::default().fg(Color::Cyan),
//!     url: Some("https://docs.rs".into()),
//! };
//!
//! let line: LinkedLine = vec![plain, link, styled];
//! ```
//!
//! Widgets that accept `LinkedLine`:
//! - [`textual_rs::Label`] — via `Label::new_linked()`
//! - [`textual_rs::RichLog`] — via `RichLog::write_linked_line()`; `write_line(Line)` still works
//!
//! ## Low-level API: `render_hyperlink`
//!
//! For custom widgets that manage their own layout:
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

use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// A text fragment with optional style and OSC 8 hyperlink URL.
///
/// Used as the building block for `LinkedLine`. Converts from `&str`, `String`,
/// and ratatui `Span` for ergonomic construction.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkedSpan {
    /// The visible text content.
    pub text: String,
    /// Ratatui style (fg, bg, modifiers).
    pub style: Style,
    /// Optional OSC 8 hyperlink URL. `None` renders as plain styled text.
    pub url: Option<String>,
}

impl LinkedSpan {
    /// Create a plain (no URL) span with default style.
    pub fn plain(text: impl Into<String>) -> Self {
        Self { text: text.into(), style: Style::default(), url: None }
    }

    /// Create a clickable linked span with default style.
    pub fn linked(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self { text: text.into(), style: Style::default(), url: Some(url.into()) }
    }

    /// Create a plain styled span with no URL.
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self { text: text.into(), style, url: None }
    }
}

impl From<&str> for LinkedSpan {
    fn from(s: &str) -> Self {
        Self::plain(s)
    }
}

impl From<String> for LinkedSpan {
    fn from(s: String) -> Self {
        Self::plain(s)
    }
}

impl From<Span<'static>> for LinkedSpan {
    fn from(span: Span<'static>) -> Self {
        Self { text: span.content.into_owned(), style: span.style, url: None }
    }
}

/// A line of text made up of [`LinkedSpan`] fragments.
///
/// Converts from ratatui `Line<'static>` so existing callers of
/// `RichLog::write_line` can pass a `Line` without changes.
pub type LinkedLine = Vec<LinkedSpan>;

/// Convert a ratatui `Line<'static>` into a `LinkedLine` (no URLs).
pub fn linked_line_from(line: Line<'static>) -> LinkedLine {
    line.spans.into_iter().map(LinkedSpan::from).collect()
}

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

/// Render a [`LinkedLine`] at `(x, y)` within `max_width` columns.
///
/// Spans with a URL are rendered as OSC 8 hyperlinks; plain spans use
/// `buf.set_string`. Returns the total number of columns consumed.
pub fn render_linked_line(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    line: &LinkedLine,
    max_width: u16,
) -> u16 {
    let mut cursor_x = x;
    let end_x = x + max_width;

    for span in line {
        if cursor_x >= end_x {
            break;
        }
        let remaining = end_x - cursor_x;
        let text: String = span.text.chars()
            .scan(0u16, |w, c| {
                let cw = UnicodeWidthStr::width(c.encode_utf8(&mut [0u8; 4])) as u16;
                if *w + cw > remaining { None } else { *w += cw; Some(c) }
            })
            .collect();
        if text.is_empty() {
            break;
        }
        let consumed = if let Some(url) = &span.url {
            render_hyperlink(buf, cursor_x, y, url, &text, span.style)
        } else {
            let w = UnicodeWidthStr::width(text.as_str()) as u16;
            buf.set_string(cursor_x, y, &text, span.style);
            w
        };
        cursor_x += consumed;
    }

    cursor_x - x
}
