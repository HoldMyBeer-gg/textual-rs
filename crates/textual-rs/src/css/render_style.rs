//! Converts ComputedStyle → ratatui visual output.
//! Called by the render loop to paint backgrounds, borders, and provide text styles.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::border;

use super::types::{BorderStyle, ComputedStyle, TcssColor};

/// Convert a TcssColor to a ratatui Color.
pub fn to_ratatui_color(c: &TcssColor) -> Option<Color> {
    match c {
        TcssColor::Reset => None,
        TcssColor::Rgb(r, g, b) => Some(Color::Rgb(*r, *g, *b)),
        TcssColor::Rgba(r, g, b, _a) => Some(Color::Rgb(*r, *g, *b)),
        TcssColor::Named(name) => named_color(name),
    }
}

fn named_color(name: &str) -> Option<Color> {
    match name {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        _ => None,
    }
}

/// Build a ratatui Style from a ComputedStyle's color and background.
pub fn text_style(cs: &ComputedStyle) -> Style {
    let mut s = Style::default();
    if let Some(fg) = to_ratatui_color(&cs.color) {
        s = s.fg(fg);
    }
    if let Some(bg) = to_ratatui_color(&cs.background) {
        s = s.bg(bg);
    }
    s
}

/// Fill the entire area with the background color from a ComputedStyle.
pub fn fill_background(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) {
    if let Some(bg) = to_ratatui_color(&cs.background) {
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                }
            }
        }
    }
}

/// Border character set for each border style.
struct BorderChars {
    top_left: &'static str,
    top_right: &'static str,
    bottom_left: &'static str,
    bottom_right: &'static str,
    horizontal: &'static str,
    vertical: &'static str,
}

fn border_chars(style: &BorderStyle) -> Option<BorderChars> {
    match style {
        BorderStyle::None => None,
        BorderStyle::Solid => Some(BorderChars {
            top_left: border::PLAIN.top_left,
            top_right: border::PLAIN.top_right,
            bottom_left: border::PLAIN.bottom_left,
            bottom_right: border::PLAIN.bottom_right,
            horizontal: border::PLAIN.horizontal_top,
            vertical: border::PLAIN.vertical_left,
        }),
        BorderStyle::Rounded => Some(BorderChars {
            top_left: "╭",
            top_right: "╮",
            bottom_left: "╰",
            bottom_right: "╯",
            horizontal: "─",
            vertical: "│",
        }),
        BorderStyle::Heavy => Some(BorderChars {
            top_left: "┏",
            top_right: "┓",
            bottom_left: "┗",
            bottom_right: "┛",
            horizontal: "━",
            vertical: "┃",
        }),
        BorderStyle::Double => Some(BorderChars {
            top_left: "╔",
            top_right: "╗",
            bottom_left: "╚",
            bottom_right: "╝",
            horizontal: "═",
            vertical: "║",
        }),
        BorderStyle::Ascii => Some(BorderChars {
            top_left: "+",
            top_right: "+",
            bottom_left: "+",
            bottom_right: "+",
            horizontal: "-",
            vertical: "|",
        }),
        BorderStyle::Tall => None, // Tall uses custom rendering in draw_tall_border
        BorderStyle::McguganBox => None, // McGugan uses custom rendering
    }
}

/// Draw a border around the area using the ComputedStyle's border and color settings.
/// Returns the inner content area (shrunk by 1 on each bordered side).
///
/// When `unicode` is false, Tall and McGugan borders degrade to ASCII (+--|).
/// Pass `true` to get the full visual fidelity (default for most terminals).
pub fn draw_border(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
    draw_border_with_caps(cs, area, buf, true)
}

/// Draw a border with explicit unicode capability flag.
/// Tall and McGugan borders fall back to ASCII when `unicode` is false.
pub fn draw_border_with_caps(
    cs: &ComputedStyle,
    area: Rect,
    buf: &mut Buffer,
    unicode: bool,
) -> Rect {
    // Tall and McGugan require unicode half-block/eighth-block chars.
    // Fall back to ASCII border on non-unicode terminals.
    if cs.border == BorderStyle::Tall {
        if unicode {
            return draw_tall_border(cs, area, buf);
        } else {
            return draw_ascii_border(cs, area, buf);
        }
    }

    if cs.border == BorderStyle::McguganBox {
        if unicode {
            return draw_mcgugan_border(cs, area, buf);
        } else {
            return draw_ascii_border(cs, area, buf);
        }
    }

    let chars = match border_chars(&cs.border) {
        Some(c) => c,
        None => return area,
    };

    if area.width < 2 || area.height < 2 {
        return area;
    }

    let border_style = {
        let mut s = Style::default();
        // Border uses the foreground color
        if let Some(fg) = to_ratatui_color(&cs.color) {
            s = s.fg(fg);
        }
        if let Some(bg) = to_ratatui_color(&cs.background) {
            s = s.bg(bg);
        }
        s
    };

    let x1 = area.x;
    let y1 = area.y;
    let x2 = area.x + area.width - 1;
    let y2 = area.y + area.height - 1;

    // Corners
    buf.set_string(x1, y1, chars.top_left, border_style);
    buf.set_string(x2, y1, chars.top_right, border_style);
    buf.set_string(x1, y2, chars.bottom_left, border_style);
    buf.set_string(x2, y2, chars.bottom_right, border_style);

    // Top and bottom edges
    for x in (x1 + 1)..x2 {
        buf.set_string(x, y1, chars.horizontal, border_style);
        buf.set_string(x, y2, chars.horizontal, border_style);
    }

    // Left and right edges
    for y in (y1 + 1)..y2 {
        buf.set_string(x1, y, chars.vertical, border_style);
        buf.set_string(x2, y, chars.vertical, border_style);
    }

    // Border title (rendered on top edge after top-left corner)
    if let Some(ref title) = cs.border_title {
        let max_len = (area.width as usize).saturating_sub(4); // leave room for corners + padding
        let display: String = title.chars().take(max_len).collect();
        if !display.is_empty() {
            let title_style = {
                let mut s = Style::default();
                if let Some(fg) = to_ratatui_color(&cs.color) {
                    s = s.fg(fg);
                }
                if let Some(bg) = to_ratatui_color(&cs.background) {
                    s = s.bg(bg);
                }
                s.add_modifier(ratatui::style::Modifier::BOLD)
            };
            buf.set_string(x1 + 2, y1, format!(" {} ", display), title_style);
        }
    }

    // Return inner area
    Rect {
        x: x1 + 1,
        y: y1 + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

/// Draw a tall border using half-block characters (▀▄▐▌) — the signature Textual look.
/// Top edge uses ▀ (upper half), bottom uses ▄ (lower half), left uses ▐, right uses ▌.
/// This creates thin, elegant frames that feel web-like rather than DOS-like.
fn draw_tall_border(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
    if area.width < 2 || area.height < 2 {
        return area;
    }

    // Border color from foreground; background blends behind the half-blocks
    let fg = to_ratatui_color(&cs.color).unwrap_or(Color::White);
    let bg = to_ratatui_color(&cs.background).unwrap_or(Color::Reset);

    let x1 = area.x;
    let y1 = area.y;
    let x2 = area.x + area.width - 1;
    let y2 = area.y + area.height - 1;

    // Use interior bg so the non-border half of each border cell blends with content.
    let interior_bg = bg;

    // Top edge: ▀ — top half = border color (fg), bottom half = interior bg
    let top_style = Style::default().fg(fg).bg(interior_bg);
    for x in x1..=x2 {
        buf.set_string(x, y1, "▀", top_style);
    }

    // Bottom edge: ▄ — bottom half = border color (fg), top half = interior bg
    let bottom_style = Style::default().fg(fg).bg(interior_bg);
    for x in x1..=x2 {
        buf.set_string(x, y2, "▄", bottom_style);
    }

    // Left/right edges: fg=border_color, bg=interior_bg
    let side_style = Style::default().fg(fg).bg(interior_bg);
    for y in (y1 + 1)..y2 {
        buf.set_string(x1, y, "▐", side_style);
    }
    for y in (y1 + 1)..y2 {
        buf.set_string(x2, y, "▌", side_style);
    }

    // Border title on top edge
    if let Some(ref title) = cs.border_title {
        let max_len = (area.width as usize).saturating_sub(4);
        let display: String = title.chars().take(max_len).collect();
        if !display.is_empty() {
            let title_style = Style::default()
                .fg(fg)
                .bg(bg)
                .add_modifier(ratatui::style::Modifier::BOLD);
            buf.set_string(x1 + 2, y1, format!(" {} ", display), title_style);
        }
    }

    // Inner content area
    Rect {
        x: x1 + 1,
        y: y1 + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

/// Draw a McGugan Box — 1/8-cell-thick borders using one-eighth block characters.
/// This is the signature Textual rendering technique with independent inside/outside colors.
fn draw_mcgugan_border(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
    if area.width < 2 || area.height < 2 {
        return area;
    }

    let border_color = to_ratatui_color(&cs.color).unwrap_or(Color::White);
    let inside_color = to_ratatui_color(&cs.background).unwrap_or(Color::Reset);
    // Outside color: use the parent's background. Since we don't have parent context here,
    // use Reset (transparent) which lets the parent's fill show through.
    let outside_color = Color::Reset;

    let (ix, iy, iw, ih) = crate::canvas::mcgugan_box(
        buf,
        area.x,
        area.y,
        area.width,
        area.height,
        border_color,
        inside_color,
        outside_color,
    );

    // Fill inside with background color
    if let Some(bg) = to_ratatui_color(&cs.background) {
        for cy in iy..iy + ih {
            for cx in ix..ix + iw {
                if let Some(cell) = buf.cell_mut((cx, cy)) {
                    cell.set_bg(bg);
                }
            }
        }
    }

    // Border title on top edge
    if let Some(ref title) = cs.border_title {
        let max_len = (area.width as usize).saturating_sub(4);
        let display: String = title.chars().take(max_len).collect();
        if !display.is_empty() {
            let title_style = Style::default()
                .fg(border_color)
                .bg(outside_color)
                .add_modifier(ratatui::style::Modifier::BOLD);
            buf.set_string(area.x + 2, area.y, format!(" {} ", display), title_style);
        }
    }

    Rect {
        x: ix,
        y: iy,
        width: iw,
        height: ih,
    }
}

/// Draw an ASCII-only border (+--|) as fallback for non-unicode terminals.
/// Used when Tall or McGugan borders are requested but unicode is not available.
fn draw_ascii_border(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
    if area.width < 2 || area.height < 2 {
        return area;
    }

    let border_style = {
        let mut s = Style::default();
        if let Some(fg) = to_ratatui_color(&cs.color) {
            s = s.fg(fg);
        }
        if let Some(bg) = to_ratatui_color(&cs.background) {
            s = s.bg(bg);
        }
        s
    };

    let x1 = area.x;
    let y1 = area.y;
    let x2 = area.x + area.width - 1;
    let y2 = area.y + area.height - 1;

    // Corners
    buf.set_string(x1, y1, "+", border_style);
    buf.set_string(x2, y1, "+", border_style);
    buf.set_string(x1, y2, "+", border_style);
    buf.set_string(x2, y2, "+", border_style);

    // Top and bottom edges
    for x in (x1 + 1)..x2 {
        buf.set_string(x, y1, "-", border_style);
        buf.set_string(x, y2, "-", border_style);
    }

    // Left and right edges
    for y in (y1 + 1)..y2 {
        buf.set_string(x1, y, "|", border_style);
        buf.set_string(x2, y, "|", border_style);
    }

    // Border title
    if let Some(ref title) = cs.border_title {
        let max_len = (area.width as usize).saturating_sub(4);
        let display: String = title.chars().take(max_len).collect();
        if !display.is_empty() {
            let title_style = border_style.add_modifier(ratatui::style::Modifier::BOLD);
            buf.set_string(x1 + 2, y1, format!(" {} ", display), title_style);
        }
    }

    Rect {
        x: x1 + 1,
        y: y1 + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

/// Paint background and borders for a widget. Returns the content area for the widget to render into.
pub fn paint_chrome(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
    fill_background(cs, area, buf);
    draw_border(cs, area, buf)
}

/// Paint background and borders with explicit unicode capability.
/// Non-unicode terminals get ASCII fallback for Tall/McGugan borders.
pub fn paint_chrome_with_caps(
    cs: &ComputedStyle,
    area: Rect,
    buf: &mut Buffer,
    unicode: bool,
) -> Rect {
    fill_background(cs, area, buf);
    draw_border_with_caps(cs, area, buf, unicode)
}

/// Align text within a given width according to the text-align CSS property.
///
/// Returns a new string with leading spaces to achieve the desired alignment.
/// For Left alignment, returns the text unchanged.
pub fn align_text(text: &str, width: usize, align: super::types::TextAlign) -> String {
    use super::types::TextAlign;
    let text_width = text.chars().count();
    if text_width >= width {
        return text.to_string();
    }
    match align {
        TextAlign::Left => text.to_string(),
        TextAlign::Center => {
            let pad = (width - text_width) / 2;
            format!("{}{}", " ".repeat(pad), text)
        }
        TextAlign::Right => {
            let pad = width - text_width;
            format!("{}{}", " ".repeat(pad), text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn tall_border_renders_half_blocks() {
        let area = Rect::new(0, 0, 6, 4);
        let mut buf = Buffer::empty(area);
        let mut cs = ComputedStyle::default();
        cs.border = BorderStyle::Tall;
        cs.color = TcssColor::Rgb(255, 255, 255);

        let inner = draw_border(&cs, area, &mut buf);

        // Inner area should be shrunk by 1 on each side
        assert_eq!(inner, Rect::new(1, 1, 4, 2));

        // Top edge: all ▀
        for x in 0..6 {
            let cell = buf.cell((x, 0)).unwrap();
            assert_eq!(cell.symbol(), "▀", "top edge at x={}", x);
        }

        // Bottom edge: all ▄
        for x in 0..6 {
            let cell = buf.cell((x, 3)).unwrap();
            assert_eq!(cell.symbol(), "▄", "bottom edge at x={}", x);
        }

        // Left edge (inner rows): ▐
        assert_eq!(buf.cell((0, 1)).unwrap().symbol(), "▐");
        assert_eq!(buf.cell((0, 2)).unwrap().symbol(), "▐");

        // Right edge (inner rows): ▌
        assert_eq!(buf.cell((5, 1)).unwrap().symbol(), "▌");
        assert_eq!(buf.cell((5, 2)).unwrap().symbol(), "▌");
    }

    #[test]
    fn tall_border_too_small_returns_full_area() {
        let area = Rect::new(0, 0, 1, 1);
        let mut buf = Buffer::empty(Rect::new(0, 0, 2, 2));
        let mut cs = ComputedStyle::default();
        cs.border = BorderStyle::Tall;

        let inner = draw_border(&cs, area, &mut buf);
        assert_eq!(inner, area); // too small, returns unchanged
    }

    #[test]
    fn align_text_left() {
        use crate::css::types::TextAlign;
        let result = align_text("hello", 10, TextAlign::Left);
        assert_eq!(result, "hello");
    }

    #[test]
    fn align_text_center() {
        use crate::css::types::TextAlign;
        let result = align_text("hi", 10, TextAlign::Center);
        assert_eq!(result, "    hi");
    }

    #[test]
    fn align_text_right() {
        use crate::css::types::TextAlign;
        let result = align_text("hi", 10, TextAlign::Right);
        assert_eq!(result, "        hi");
    }

    #[test]
    fn align_text_wider_than_width() {
        use crate::css::types::TextAlign;
        // When text is wider than width, return unchanged
        let result = align_text("hello world", 5, TextAlign::Center);
        assert_eq!(result, "hello world");
    }
}
