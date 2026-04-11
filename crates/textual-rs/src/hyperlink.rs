//! OSC 8 hyperlink support for textual-rs widgets.
//!
//! OSC 8 is a terminal escape sequence that creates clickable hyperlinks in
//! terminals that support it: iTerm2, kitty, Ghostty, WezTerm, Windows Terminal,
//! and most modern terminal emulators.
//!
//! ## How it works
//!
//! Hyperlinks are collected into a per-frame side table during rendering and
//! flushed to the terminal **after** `terminal.draw()` completes. This avoids
//! embedding OSC sequences directly in ratatui `Buffer` cells, which would
//! corrupt ratatui's diff renderer (it computes skip counts from `cell.symbol().width()`,
//! causing cells adjacent to a link to be skipped).
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
//! - [`Label`](crate::Label) — via `Label::new_linked()`
//! - [`RichLog`](crate::RichLog) — via `RichLog::write_linked_line()`; `write_line(Line)` still works
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

use std::cell::RefCell;
use std::io::Write;

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// A pending OSC 8 hyperlink to be flushed to the terminal after each frame.
#[derive(Debug, Clone)]
pub struct HyperlinkRecord {
    /// Absolute buffer column.
    pub x: u16,
    /// Absolute buffer row.
    pub y: u16,
    /// Visible label text (already rendered to the buffer).
    pub label: String,
    /// Hyperlink target URL.
    pub url: String,
    /// Full cell style (fg + bg + modifiers, merged with paint_chrome background).
    pub style: Style,
}

thread_local! {
    static FRAME_HYPERLINKS: RefCell<Vec<HyperlinkRecord>> = const { RefCell::new(Vec::new()) };
}

/// Flush all hyperlink records collected during the last render frame to `writer`.
///
/// Call this once after `terminal.draw()` returns. Each hyperlink is re-printed
/// at its buffer position with OSC 8 open/close sequences wrapping the label.
/// Cursor position and SGR attributes are saved and restored around the writes.
///
/// For the `TestBackend` path, call [`drain_frame_hyperlinks`] instead.
pub fn flush_frame_hyperlinks(writer: &mut impl Write) -> std::io::Result<()> {
    let links = FRAME_HYPERLINKS.with(|h| std::mem::take(&mut *h.borrow_mut()));
    if links.is_empty() {
        return Ok(());
    }
    // Hide cursor and save position to avoid flicker during overwrites.
    writer.write_all(b"\x1b[?25l\x1b7")?;
    for link in &links {
        // Move to start of link.
        write!(writer, "\x1b[{};{}H", link.y + 1, link.x + 1)?;
        // Apply the stored style so the overprint is visually identical.
        write_style(writer, link.style)?;
        // Emit OSC 8 open + label + OSC 8 close.
        write!(
            writer,
            "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            link.url, link.label
        )?;
    }
    // Reset SGR and restore saved cursor (position + visibility).
    writer.write_all(b"\x1b[0m\x1b8")?;
    writer.flush()
}

/// Convenience wrapper: flush directly to `stdout`.
///
/// Called by the main app loop after each `terminal.draw()`.
pub fn flush_frame_hyperlinks_to_stdout() -> std::io::Result<()> {
    // Fast-path: skip locking stdout when there are no hyperlinks.
    let is_empty = FRAME_HYPERLINKS.with(|h| h.borrow().is_empty());
    if is_empty {
        return Ok(());
    }
    let mut out = std::io::stdout().lock();
    flush_frame_hyperlinks(&mut out)
}

/// Drain and discard the frame hyperlink queue without writing to any terminal.
///
/// Use this in the `TestBackend` rendering path so records don't accumulate
/// between test cases.
pub fn drain_frame_hyperlinks() -> Vec<HyperlinkRecord> {
    FRAME_HYPERLINKS.with(|h| std::mem::take(&mut *h.borrow_mut()))
}

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
        Self {
            text: text.into(),
            style: Style::default(),
            url: None,
        }
    }

    /// Create a clickable linked span with default style.
    pub fn linked(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            url: Some(url.into()),
        }
    }

    /// Create a plain styled span with no URL.
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
            url: None,
        }
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
        Self {
            text: span.content.into_owned(),
            style: span.style,
            url: None,
        }
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
/// Writes the label into the buffer normally (identical to `buf.set_string`),
/// then enqueues a [`HyperlinkRecord`] that is flushed to the terminal after
/// `terminal.draw()` by [`flush_frame_hyperlinks_to_stdout`].
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

    // Write label into the buffer exactly as plain text.
    buf.set_string(x, y, label, style);

    // Read back the merged cell style (includes background applied by paint_chrome).
    let cell_style = buf.cell((x, y)).map(|c| c.style()).unwrap_or(style);

    FRAME_HYPERLINKS.with(|h| {
        h.borrow_mut().push(HyperlinkRecord {
            x,
            y,
            label: label.to_string(),
            url: url.to_string(),
            style: cell_style,
        });
    });

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
        let text: String = span
            .text
            .chars()
            .scan(0u16, |w, c| {
                let cw = UnicodeWidthStr::width(c.encode_utf8(&mut [0u8; 4])) as u16;
                if *w + cw > remaining {
                    None
                } else {
                    *w += cw;
                    Some(c)
                }
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

// ---------------------------------------------------------------------------
// Internal: write a ratatui Style as ANSI SGR sequences
// ---------------------------------------------------------------------------

fn write_style(writer: &mut impl Write, style: Style) -> std::io::Result<()> {
    writer.write_all(b"\x1b[0m")?; // reset first
    let m = style.add_modifier;
    if m.contains(Modifier::BOLD) {
        writer.write_all(b"\x1b[1m")?;
    }
    if m.contains(Modifier::DIM) {
        writer.write_all(b"\x1b[2m")?;
    }
    if m.contains(Modifier::ITALIC) {
        writer.write_all(b"\x1b[3m")?;
    }
    if m.contains(Modifier::UNDERLINED) {
        writer.write_all(b"\x1b[4m")?;
    }
    if m.contains(Modifier::SLOW_BLINK) {
        writer.write_all(b"\x1b[5m")?;
    }
    if m.contains(Modifier::RAPID_BLINK) {
        writer.write_all(b"\x1b[6m")?;
    }
    if m.contains(Modifier::REVERSED) {
        writer.write_all(b"\x1b[7m")?;
    }
    if m.contains(Modifier::HIDDEN) {
        writer.write_all(b"\x1b[8m")?;
    }
    if m.contains(Modifier::CROSSED_OUT) {
        writer.write_all(b"\x1b[9m")?;
    }
    if let Some(fg) = style.fg {
        write_color(writer, fg, false)?;
    }
    if let Some(bg) = style.bg {
        write_color(writer, bg, true)?;
    }
    Ok(())
}

fn write_color(writer: &mut impl Write, color: Color, bg: bool) -> std::io::Result<()> {
    let base_fg: u8 = if bg { 40 } else { 30 };
    match color {
        Color::Reset => write!(writer, "\x1b[{}m", if bg { 49 } else { 39 })?,
        Color::Black => write!(writer, "\x1b[{}m", base_fg)?,
        Color::Red => write!(writer, "\x1b[{}m", base_fg + 1)?,
        Color::Green => write!(writer, "\x1b[{}m", base_fg + 2)?,
        Color::Yellow => write!(writer, "\x1b[{}m", base_fg + 3)?,
        Color::Blue => write!(writer, "\x1b[{}m", base_fg + 4)?,
        Color::Magenta => write!(writer, "\x1b[{}m", base_fg + 5)?,
        Color::Cyan => write!(writer, "\x1b[{}m", base_fg + 6)?,
        Color::Gray => write!(writer, "\x1b[{}m", base_fg + 7)?,
        Color::DarkGray => write!(writer, "\x1b[{}m", base_fg + 60)?,
        Color::LightRed => write!(writer, "\x1b[{}m", base_fg + 61)?,
        Color::LightGreen => write!(writer, "\x1b[{}m", base_fg + 62)?,
        Color::LightYellow => write!(writer, "\x1b[{}m", base_fg + 63)?,
        Color::LightBlue => write!(writer, "\x1b[{}m", base_fg + 64)?,
        Color::LightMagenta => write!(writer, "\x1b[{}m", base_fg + 65)?,
        Color::LightCyan => write!(writer, "\x1b[{}m", base_fg + 66)?,
        Color::White => write!(writer, "\x1b[{}m", base_fg + 67)?,
        Color::Rgb(r, g, b) => write!(writer, "\x1b[{};2;{r};{g};{b}m", if bg { 48 } else { 38 })?,
        Color::Indexed(n) => write!(writer, "\x1b[{};5;{n}m", if bg { 48 } else { 38 })?,
    }
    Ok(())
}
