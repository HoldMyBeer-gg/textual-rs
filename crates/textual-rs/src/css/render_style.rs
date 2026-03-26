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
    }
}

/// Draw a border around the area using the ComputedStyle's border and color settings.
/// Returns the inner content area (shrunk by 1 on each bordered side).
pub fn draw_border(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
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
            buf.set_string(x1 + 2, y1, &format!(" {} ", display), title_style);
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

/// Paint background and borders for a widget. Returns the content area for the widget to render into.
pub fn paint_chrome(cs: &ComputedStyle, area: Rect, buf: &mut Buffer) -> Rect {
    fill_background(cs, area, buf);
    draw_border(cs, area, buf)
}
