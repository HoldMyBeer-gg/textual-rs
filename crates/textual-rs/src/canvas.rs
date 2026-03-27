//! Sub-cell rendering primitives using Unicode half-block and block characters.
//!
//! Terminal cells are normally one character wide, one row tall. By using the
//! upper-half-block character "▀" with foreground = top color and background =
//! bottom color, we get **two vertical pixels per cell**, doubling effective
//! vertical resolution. This is how Python Textual achieves its polished look.
//!
//! Block level characters "▁▂▃▄▅▆▇█" provide 8 discrete fill levels within a
//! single cell, used for progress bars, scrollbar thumbs, and sparklines.

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};

/// Upper half block — foreground fills top half, background fills bottom half.
pub const UPPER_HALF: &str = "▀";
/// Lower half block — foreground fills bottom half, background fills top half.
pub const LOWER_HALF: &str = "▄";
/// Full block.
pub const FULL_BLOCK: &str = "█";

/// 8-level vertical fill blocks, from 1/8 to 8/8.
pub const VERTICAL_BLOCKS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// 8-level horizontal fill blocks (right to left), from 7/8 to 1/8.
pub const HORIZONTAL_BLOCKS: [&str; 8] = ["▉", "▊", "▋", "▌", "▍", "▎", "▏", " "];

/// Paint a single cell with two vertical colors using the half-block technique.
/// `top` is the color of the upper half, `bottom` is the color of the lower half.
pub fn half_block_cell(buf: &mut Buffer, x: u16, y: u16, top: Color, bottom: Color) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(UPPER_HALF);
        cell.set_fg(top);
        cell.set_bg(bottom);
    }
}

/// Fill a rectangular area with a vertical gradient from `color_top` to `color_bottom`.
/// Uses half-block characters to achieve 2x vertical resolution.
pub fn vertical_gradient(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    color_top: Color,
    color_bottom: Color,
) {
    if height == 0 || width == 0 {
        return;
    }
    // Total sub-cell rows = height * 2 (each cell has top and bottom halves)
    let total_steps = (height as f64) * 2.0;
    for row in 0..height {
        // Top half of this cell row
        let t_top = (row as f64 * 2.0) / (total_steps - 1.0);
        // Bottom half of this cell row
        let t_bottom = (row as f64 * 2.0 + 1.0) / (total_steps - 1.0);
        let top_color = blend_color(color_top, color_bottom, t_top);
        let bottom_color = blend_color(color_top, color_bottom, t_bottom.min(1.0));
        for col in x..x + width {
            half_block_cell(buf, col, y + row, top_color, bottom_color);
        }
    }
}

/// Linear interpolation between two RGB colors. `t` is 0.0 to 1.0.
pub fn blend_color(from: Color, to: Color, t: f64) -> Color {
    let (r1, g1, b1) = color_to_rgb(from);
    let (r2, g2, b2) = color_to_rgb(to);
    let t = t.clamp(0.0, 1.0);
    Color::Rgb(
        lerp_u8(r1, r2, t),
        lerp_u8(g1, g2, t),
        lerp_u8(b1, b2, t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u8
}

fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Red => (170, 0, 0),
        Color::Green => (0, 170, 0),
        Color::Blue => (0, 0, 170),
        Color::Yellow => (170, 170, 0),
        Color::Magenta => (170, 0, 170),
        Color::Cyan => (0, 170, 170),
        Color::Gray => (170, 170, 170),
        Color::DarkGray => (85, 85, 85),
        _ => (128, 128, 128),
    }
}

/// Render a sub-cell progress bar. Uses 8-level block characters for smooth fill.
/// Each cell can show 8 discrete fill levels, giving `width * 8` effective resolution.
///
/// `progress` is 0.0 to 1.0.
/// `fill_color` is the bar color, `empty_color` is the track color.
pub fn progress_bar(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    progress: f64,
    fill_color: Color,
    empty_color: Color,
) {
    if width == 0 {
        return;
    }
    let progress = progress.clamp(0.0, 1.0);
    let total_eighths = (progress * (width as f64) * 8.0).round() as usize;
    let full_cells = total_eighths / 8;
    let remainder = total_eighths % 8;

    let fill_style = Style::default().fg(fill_color).bg(empty_color);
    let empty_style = Style::default().fg(empty_color).bg(empty_color);

    for col in 0..width {
        let i = col as usize;
        if i < full_cells {
            // Fully filled cell
            if let Some(cell) = buf.cell_mut((x + col, y)) {
                cell.set_symbol(FULL_BLOCK);
                cell.set_style(fill_style);
            }
        } else if i == full_cells && remainder > 0 {
            // Partial fill using block character
            let block = VERTICAL_BLOCKS[remainder - 1];
            // For horizontal progress, we use the block chars rotated:
            // Actually, vertical blocks fill from bottom. For left-to-right,
            // we want horizontal blocks. Let's use a different approach:
            // Use "█" with width proportional fill via fg/bg trick.
            // Simpler: just use the vertical block with fg=fill, bg=empty.
            if let Some(cell) = buf.cell_mut((x + col, y)) {
                cell.set_symbol(block);
                cell.set_fg(fill_color);
                cell.set_bg(empty_color);
            }
        } else {
            // Empty cell
            if let Some(cell) = buf.cell_mut((x + col, y)) {
                cell.set_symbol(" ");
                cell.set_style(empty_style);
            }
        }
    }
}

/// Render a vertical scrollbar with sub-cell thumb positioning.
/// Uses 8-level block characters for smooth thumb edges.
///
/// `size` is the scrollbar height in cells.
/// `content_size` is the total scrollable content size.
/// `viewport_size` is the visible viewport size.
/// `position` is the current scroll offset.
/// `bar_color` is the thumb color, `track_color` is the background.
pub fn vertical_scrollbar(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    size: u16,
    content_size: usize,
    viewport_size: usize,
    position: usize,
    bar_color: Color,
    track_color: Color,
) {
    if size == 0 || content_size == 0 || viewport_size >= content_size {
        // No scrollbar needed — fill with track color
        for row in 0..size {
            if let Some(cell) = buf.cell_mut((x, y + row)) {
                cell.set_symbol(" ");
                cell.set_bg(track_color);
            }
        }
        return;
    }

    let bar_ratio = content_size as f64 / size as f64;
    let thumb_size = (viewport_size as f64 / bar_ratio).max(1.0);
    let max_scroll = content_size - viewport_size;
    let position_ratio = if max_scroll > 0 {
        position as f64 / max_scroll as f64
    } else {
        0.0
    };
    let thumb_pos = position_ratio * (size as f64 - thumb_size);

    // Convert to sub-cell units (8 per cell)
    let start_eighths = (thumb_pos * 8.0).round() as usize;
    let end_eighths = start_eighths + (thumb_size * 8.0).round() as usize;

    let start_cell = start_eighths / 8;
    let start_sub = start_eighths % 8;
    let end_cell = end_eighths / 8;
    let end_sub = end_eighths % 8;

    let track_style = Style::default().bg(track_color);

    for row in 0..size {
        let i = row as usize;
        if i < start_cell || i > end_cell {
            // Track
            if let Some(cell) = buf.cell_mut((x, y + row)) {
                cell.set_symbol(" ");
                cell.set_style(track_style);
            }
        } else if i == start_cell && start_sub > 0 {
            // Top edge of thumb — partial block
            let block_idx = 8 - start_sub; // invert: higher sub = less visible
            let block = if block_idx > 0 && block_idx <= 8 {
                VERTICAL_BLOCKS[block_idx.min(8) - 1]
            } else {
                " "
            };
            if let Some(cell) = buf.cell_mut((x, y + row)) {
                cell.set_symbol(block);
                cell.set_fg(bar_color);
                cell.set_bg(track_color);
            }
        } else if i == end_cell && end_sub > 0 && end_cell > start_cell {
            // Bottom edge of thumb — partial block
            let block = VERTICAL_BLOCKS[end_sub.min(8) - 1];
            if let Some(cell) = buf.cell_mut((x, y + row)) {
                cell.set_symbol(block);
                cell.set_fg(bar_color);
                cell.set_bg(track_color);
            }
        } else if i >= start_cell && i <= end_cell {
            // Full thumb cell
            if let Some(cell) = buf.cell_mut((x, y + row)) {
                cell.set_symbol(" ");
                cell.set_bg(bar_color);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// McGugan Box characters — 1/8-thick borders with independent inside/outside colors
// ---------------------------------------------------------------------------

/// Lower one-eighth block — thin bottom border line.
pub const LOWER_ONE_EIGHTH: &str = "\u{2581}";
/// Upper one-eighth block — thin top border line.
pub const UPPER_ONE_EIGHTH: &str = "\u{2594}";
/// Left one-quarter block — thin left border line.
pub const LEFT_ONE_QUARTER: &str = "\u{258E}";
/// Right one-quarter block — thin right border line (Unicode 13 Legacy Computing).
pub const RIGHT_ONE_QUARTER: &str = "\u{1FB87}";
/// Fallback right border character (U+2595 RIGHT ONE EIGHTH BLOCK).
/// U+1FB87 is from the Unicode 13 "Legacy Computing Supplement" block and is
/// not supported by many terminal fonts. U+2595 has been available since
/// Unicode 1.1 and is universally supported. Python Textual also uses this
/// character for broad compatibility.
pub const RIGHT_BORDER_FALLBACK: &str = "\u{2595}";

/// Draw a McGugan Box — a 1/8-cell-thick border with independent inside and outside colors.
///
/// This is the signature Textual rendering technique. The border is drawn using
/// one-eighth block characters with carefully assigned fg/bg colors so the border,
/// inside, and outside all have distinct colors in a single character cell.
///
/// Returns the inner content area.
pub fn mcgugan_box(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    border_color: Color,
    inside_color: Color,
    outside_color: Color,
) -> (u16, u16, u16, u16) {
    if width < 2 || height < 2 {
        return (x, y, width, height);
    }

    let x2 = x + width - 1;
    let y2 = y + height - 1;

    // Top edge: LOWER_ONE_EIGHTH with fg=border, bg=outside
    // The thin line appears at the bottom of the cell (the actual border line),
    // the rest of the cell shows the outside color as bg.
    let top_style = Style::default().fg(border_color).bg(outside_color);
    for cx in x..=x2 {
        buf.set_string(cx, y, LOWER_ONE_EIGHTH, top_style);
    }

    // Bottom edge: UPPER_ONE_EIGHTH with fg=border, bg=outside
    let bottom_style = Style::default().fg(border_color).bg(outside_color);
    for cx in x..=x2 {
        buf.set_string(cx, y2, UPPER_ONE_EIGHTH, bottom_style);
    }

    // Left edge (inner rows): LEFT_ONE_QUARTER with fg=border, bg=inside
    let left_style = Style::default().fg(border_color).bg(inside_color);
    for cy in (y + 1)..y2 {
        buf.set_string(x, cy, LEFT_ONE_QUARTER, left_style);
    }

    // Right edge (inner rows): RIGHT_BORDER_FALLBACK (U+2595) with fg=border, bg=inside
    // Uses the broadly-compatible fallback instead of U+1FB87 which is missing from many fonts.
    let right_style = Style::default().fg(border_color).bg(inside_color);
    for cy in (y + 1)..y2 {
        buf.set_string(x2, cy, RIGHT_BORDER_FALLBACK, right_style);
    }

    // Inner content area (shrunk by 1 on each side)
    (x + 1, y + 1, width.saturating_sub(2), height.saturating_sub(2))
}

// ---------------------------------------------------------------------------
// Quadrant characters — 2x2 sub-cell resolution (4 pixels per cell)
// ---------------------------------------------------------------------------

/// Quadrant block characters indexed by bitmask.
/// Bit 0 = top-left, bit 1 = top-right, bit 2 = bottom-left, bit 3 = bottom-right.
pub const QUADRANT_CHARS: [&str; 16] = [
    " ",         // 0b0000
    "\u{2598}",  // 0b0001 ▘ top-left
    "\u{259D}",  // 0b0010 ▝ top-right
    "\u{2580}",  // 0b0011 ▀ upper half
    "\u{2596}",  // 0b0100 ▖ bottom-left
    "\u{258C}",  // 0b0101 ▌ left half
    "\u{259E}",  // 0b0110 ▞ diagonal
    "\u{259B}",  // 0b0111 ▛ top-left + top-right + bottom-left
    "\u{2597}",  // 0b1000 ▗ bottom-right
    "\u{259A}",  // 0b1001 ▚ anti-diagonal
    "\u{2590}",  // 0b1010 ▐ right half
    "\u{259C}",  // 0b1011 ▜ top-left + top-right + bottom-right
    "\u{2584}",  // 0b1100 ▄ lower half
    "\u{2599}",  // 0b1101 ▙ top-left + bottom-left + bottom-right
    "\u{259F}",  // 0b1110 ▟ top-right + bottom-left + bottom-right
    "\u{2588}",  // 0b1111 █ full block
];

/// Set a single quadrant cell. `mask` is a 4-bit value where:
/// bit 0 = top-left, bit 1 = top-right, bit 2 = bottom-left, bit 3 = bottom-right.
/// `fg` is the filled quadrant color, `bg` is the empty quadrant color.
pub fn quadrant_cell(buf: &mut Buffer, x: u16, y: u16, mask: u8, fg: Color, bg: Color) {
    let idx = (mask & 0x0F) as usize;
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(QUADRANT_CHARS[idx]);
        cell.set_fg(fg);
        cell.set_bg(bg);
    }
}

// ---------------------------------------------------------------------------
// Braille characters — 2x4 sub-cell resolution (8 dots per cell)
// ---------------------------------------------------------------------------

/// Braille base character (empty pattern).
pub const BRAILLE_BASE: u32 = 0x2800;

/// Braille dot offsets. Each dot position maps to a bit in the Unicode Braille range.
/// Layout:  (0)(1)    Bit positions:
///          (2)(3)    0=0x01, 1=0x08
///          (4)(5)    2=0x02, 3=0x10
///          (6)(7)    4=0x04, 5=0x20
///                    6=0x40, 7=0x80
pub const BRAILLE_DOT_BITS: [u32; 8] = [
    0x01, // dot 0: top-left
    0x08, // dot 1: top-right
    0x02, // dot 2: mid-upper-left
    0x10, // dot 3: mid-upper-right
    0x04, // dot 4: mid-lower-left
    0x20, // dot 5: mid-lower-right
    0x40, // dot 6: bottom-left
    0x80, // dot 7: bottom-right
];

/// Set a Braille cell from an 8-bit dot pattern.
/// Each bit corresponds to a dot position (see BRAILLE_DOT_BITS layout).
/// `fg` is the dot color, `bg` is the background.
pub fn braille_cell(buf: &mut Buffer, x: u16, y: u16, dots: u8, fg: Color, bg: Color) {
    let mut codepoint = BRAILLE_BASE;
    for i in 0..8 {
        if dots & (1 << i) != 0 {
            codepoint |= BRAILLE_DOT_BITS[i];
        }
    }
    if let Some(ch) = char::from_u32(codepoint) {
        let s: String = ch.to_string();
        let style = Style::default().fg(fg).bg(bg);
        buf.set_string(x, y, &s, style);
    }
}

/// Set a single dot in a Braille cell at sub-cell position (dx, dy).
/// dx: 0-1 (left, right), dy: 0-3 (top to bottom).
/// Returns the dot index (0-7) for use with braille_cell.
pub fn braille_dot_index(dx: u8, dy: u8) -> u8 {
    let col = (dx & 1) as usize;
    let row = (dy.min(3)) as usize;
    (row * 2 + col) as u8
}

// ---------------------------------------------------------------------------
// Hatch pattern rendering
// ---------------------------------------------------------------------------

use crate::css::types::HatchStyle;

/// Characters used for hatch patterns.
const HATCH_CROSS: &str = "\u{2573}";      // Box Drawings Light Diagonal Cross
const HATCH_HORIZONTAL: &str = "\u{2500}";  // Box Drawings Light Horizontal
const HATCH_VERTICAL: &str = "\u{2502}";    // Box Drawings Light Vertical
const HATCH_LEFT: &str = "\u{2571}";        // Box Drawings Light Diagonal Upper Right to Lower Left
const HATCH_RIGHT: &str = "\u{2572}";       // Box Drawings Light Diagonal Upper Left to Lower Right

/// Render a hatch pattern fill over a rectangular area.
pub fn render_hatch(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    hatch: HatchStyle,
    fg: Color,
    bg: Color,
) {
    let ch = match hatch {
        HatchStyle::Cross => HATCH_CROSS,
        HatchStyle::Horizontal => HATCH_HORIZONTAL,
        HatchStyle::Vertical => HATCH_VERTICAL,
        HatchStyle::Left => HATCH_LEFT,
        HatchStyle::Right => HATCH_RIGHT,
    };
    let style = Style::default().fg(fg).bg(bg);
    for row in 0..height {
        for col in 0..width {
            buf.set_string(x + col, y + row, ch, style);
        }
    }
}

// ---------------------------------------------------------------------------
// Image rendering using half-block technique
// ---------------------------------------------------------------------------

/// Render an image from raw RGB pixel data using the half-block technique.
///
/// Each terminal cell represents 2 vertical pixels: the upper half-block character
/// with fg = top pixel color and bg = bottom pixel color.
///
/// `pixels` is a row-major array of (R, G, B) tuples. The pixel grid dimensions
/// are `pixel_width` x `pixel_height`. The image is rendered starting at terminal
/// position (x, y), occupying `pixel_width` columns and `ceil(pixel_height / 2)` rows.
///
/// If `pixel_height` is odd, the last row of pixels uses the background color as the
/// bottom half.
pub fn render_image_halfblock(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    pixel_width: u16,
    pixel_height: u16,
    pixels: &[(u8, u8, u8)],
) {
    if pixel_width == 0 || pixel_height == 0 || pixels.is_empty() {
        return;
    }

    let cell_rows = (pixel_height as usize).div_ceil(2);

    for cell_row in 0..cell_rows {
        let top_pixel_y = cell_row * 2;
        let bot_pixel_y = top_pixel_y + 1;

        for col in 0..(pixel_width as usize) {
            let top_idx = top_pixel_y * (pixel_width as usize) + col;
            let top_color = if top_idx < pixels.len() {
                let (r, g, b) = pixels[top_idx];
                Color::Rgb(r, g, b)
            } else {
                Color::Black
            };

            let bot_color = if bot_pixel_y < pixel_height as usize {
                let bot_idx = bot_pixel_y * (pixel_width as usize) + col;
                if bot_idx < pixels.len() {
                    let (r, g, b) = pixels[bot_idx];
                    Color::Rgb(r, g, b)
                } else {
                    Color::Black
                }
            } else {
                Color::Black
            };

            half_block_cell(buf, x + col as u16, y + cell_row as u16, top_color, bot_color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn blend_endpoints() {
        let black = Color::Rgb(0, 0, 0);
        let white = Color::Rgb(255, 255, 255);
        assert_eq!(blend_color(black, white, 0.0), Color::Rgb(0, 0, 0));
        assert_eq!(blend_color(black, white, 1.0), Color::Rgb(255, 255, 255));
        assert_eq!(blend_color(black, white, 0.5), Color::Rgb(128, 128, 128));
    }

    #[test]
    fn progress_bar_empty() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        progress_bar(&mut buf, 0, 0, 10, 0.0, Color::Green, Color::DarkGray);
        // All cells should be empty (space with empty_color bg)
        for x in 0..10 {
            assert_eq!(buf[(x, 0)].symbol(), " ");
        }
    }

    #[test]
    fn progress_bar_full() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        progress_bar(&mut buf, 0, 0, 10, 1.0, Color::Green, Color::DarkGray);
        // All cells should be full block
        for x in 0..10 {
            assert_eq!(buf[(x, 0)].symbol(), "█");
        }
    }

    #[test]
    fn progress_bar_partial() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        progress_bar(&mut buf, 0, 0, 10, 0.5, Color::Green, Color::DarkGray);
        // First 5 cells full, rest empty
        for x in 0..5 {
            assert_eq!(buf[(x, 0)].symbol(), "█");
        }
        for x in 5..10 {
            assert_eq!(buf[(x, 0)].symbol(), " ");
        }
    }

    #[test]
    fn half_block_sets_colors() {
        let area = Rect::new(0, 0, 1, 1);
        let mut buf = Buffer::empty(area);
        half_block_cell(&mut buf, 0, 0, Color::Red, Color::Blue);
        assert_eq!(buf[(0, 0)].symbol(), "▀");
        assert_eq!(buf[(0, 0)].fg, Color::Red);
        assert_eq!(buf[(0, 0)].bg, Color::Blue);
    }

    #[test]
    fn mcgugan_box_renders_eighth_blocks() {
        let area = Rect::new(0, 0, 6, 4);
        let mut buf = Buffer::empty(area);
        let border = Color::Rgb(255, 255, 255);
        let inside = Color::Rgb(30, 30, 30);
        let outside = Color::Rgb(0, 0, 0);

        let (ix, iy, iw, ih) = mcgugan_box(&mut buf, 0, 0, 6, 4, border, inside, outside);
        assert_eq!((ix, iy, iw, ih), (1, 1, 4, 2));

        // Top edge: lower one-eighth block
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), LOWER_ONE_EIGHTH);
        assert_eq!(buf.cell((5, 0)).unwrap().symbol(), LOWER_ONE_EIGHTH);

        // Bottom edge: upper one-eighth block
        assert_eq!(buf.cell((0, 3)).unwrap().symbol(), UPPER_ONE_EIGHTH);

        // Left edge: left one-quarter block
        assert_eq!(buf.cell((0, 1)).unwrap().symbol(), LEFT_ONE_QUARTER);
        assert_eq!(buf.cell((0, 2)).unwrap().symbol(), LEFT_ONE_QUARTER);

        // Right edge: right border fallback (U+2595) for broad terminal compatibility
        assert_eq!(buf.cell((5, 1)).unwrap().symbol(), RIGHT_BORDER_FALLBACK);
    }

    #[test]
    fn mcgugan_box_colors_independent() {
        let area = Rect::new(0, 0, 4, 3);
        let mut buf = Buffer::empty(area);
        let border = Color::Rgb(255, 0, 0);
        let inside = Color::Rgb(0, 255, 0);
        let outside = Color::Rgb(0, 0, 255);

        mcgugan_box(&mut buf, 0, 0, 4, 3, border, inside, outside);

        // Top edge: fg=border, bg=outside
        let top = buf.cell((1, 0)).unwrap();
        assert_eq!(top.fg, border);
        assert_eq!(top.bg, outside);

        // Left edge: fg=border, bg=inside
        let left = buf.cell((0, 1)).unwrap();
        assert_eq!(left.fg, border);
        assert_eq!(left.bg, inside);
    }

    #[test]
    fn quadrant_cell_renders_correct_chars() {
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);

        // Top-left only = ▘
        quadrant_cell(&mut buf, 0, 0, 0b0001, Color::White, Color::Black);
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "\u{2598}");

        // Full block = █
        quadrant_cell(&mut buf, 1, 0, 0b1111, Color::White, Color::Black);
        assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "\u{2588}");
    }

    #[test]
    fn braille_empty_and_full() {
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);

        // Empty braille
        braille_cell(&mut buf, 0, 0, 0, Color::White, Color::Black);
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "\u{2800}"); // ⠀

        // All 8 dots
        braille_cell(&mut buf, 1, 0, 0xFF, Color::White, Color::Black);
        assert_eq!(buf.cell((1, 0)).unwrap().symbol(), "\u{28FF}"); // ⣿
    }

    #[test]
    fn mcgugan_box_right_border_uses_fallback() {
        // The right border should use RIGHT_BORDER_FALLBACK (U+2595) for broad
        // terminal compatibility, not RIGHT_ONE_QUARTER (U+1FB87).
        let area = Rect::new(0, 0, 6, 4);
        let mut buf = Buffer::empty(area);
        let border = Color::Rgb(255, 255, 255);
        let inside = Color::Rgb(30, 30, 30);
        let outside = Color::Rgb(0, 0, 0);

        mcgugan_box(&mut buf, 0, 0, 6, 4, border, inside, outside);

        // Right edge inner rows should use the fallback character
        assert_eq!(buf.cell((5, 1)).unwrap().symbol(), RIGHT_BORDER_FALLBACK);
        assert_eq!(buf.cell((5, 2)).unwrap().symbol(), RIGHT_BORDER_FALLBACK);
    }

    #[test]
    fn braille_dot_index_layout() {
        // top-left = 0, top-right = 1
        assert_eq!(braille_dot_index(0, 0), 0);
        assert_eq!(braille_dot_index(1, 0), 1);
        // bottom-left = 6, bottom-right = 7
        assert_eq!(braille_dot_index(0, 3), 6);
        assert_eq!(braille_dot_index(1, 3), 7);
    }

    // --- Hatch rendering tests ---

    #[test]
    fn render_hatch_cross_fills_area() {
        let area = Rect::new(0, 0, 3, 2);
        let mut buf = Buffer::empty(area);
        render_hatch(&mut buf, 0, 0, 3, 2, HatchStyle::Cross, Color::White, Color::Black);
        for y in 0..2u16 {
            for x in 0..3u16 {
                assert_eq!(buf.cell((x, y)).unwrap().symbol(), HATCH_CROSS);
                assert_eq!(buf.cell((x, y)).unwrap().fg, Color::White);
                assert_eq!(buf.cell((x, y)).unwrap().bg, Color::Black);
            }
        }
    }

    #[test]
    fn render_hatch_all_styles() {
        let styles = [
            (HatchStyle::Cross, HATCH_CROSS),
            (HatchStyle::Horizontal, HATCH_HORIZONTAL),
            (HatchStyle::Vertical, HATCH_VERTICAL),
            (HatchStyle::Left, HATCH_LEFT),
            (HatchStyle::Right, HATCH_RIGHT),
        ];
        for (style, expected_char) in styles {
            let area = Rect::new(0, 0, 1, 1);
            let mut buf = Buffer::empty(area);
            render_hatch(&mut buf, 0, 0, 1, 1, style, Color::Red, Color::Blue);
            assert_eq!(
                buf.cell((0, 0)).unwrap().symbol(),
                expected_char,
                "hatch style {:?} should use char {}",
                style,
                expected_char,
            );
        }
    }

    // --- Image rendering tests ---

    #[test]
    fn render_image_halfblock_2x2() {
        // 2x2 pixel image -> 2 columns, 1 row
        let pixels = vec![
            (255, 0, 0), (0, 255, 0),  // row 0: red, green
            (0, 0, 255), (255, 255, 0), // row 1: blue, yellow
        ];
        let area = Rect::new(0, 0, 2, 1);
        let mut buf = Buffer::empty(area);
        render_image_halfblock(&mut buf, 0, 0, 2, 2, &pixels);

        // Cell (0,0): top=red, bottom=blue
        assert_eq!(buf.cell((0, 0)).unwrap().symbol(), UPPER_HALF);
        assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Rgb(255, 0, 0));
        assert_eq!(buf.cell((0, 0)).unwrap().bg, Color::Rgb(0, 0, 255));

        // Cell (1,0): top=green, bottom=yellow
        assert_eq!(buf.cell((1, 0)).unwrap().symbol(), UPPER_HALF);
        assert_eq!(buf.cell((1, 0)).unwrap().fg, Color::Rgb(0, 255, 0));
        assert_eq!(buf.cell((1, 0)).unwrap().bg, Color::Rgb(255, 255, 0));
    }

    #[test]
    fn render_image_halfblock_odd_height() {
        // 1x3 pixel image -> 1 column, 2 rows (last row bottom half = black)
        let pixels = vec![
            (255, 0, 0),   // row 0
            (0, 255, 0),   // row 1
            (0, 0, 255),   // row 2
        ];
        let area = Rect::new(0, 0, 1, 2);
        let mut buf = Buffer::empty(area);
        render_image_halfblock(&mut buf, 0, 0, 1, 3, &pixels);

        // Cell (0,0): top=red, bottom=green
        assert_eq!(buf.cell((0, 0)).unwrap().fg, Color::Rgb(255, 0, 0));
        assert_eq!(buf.cell((0, 0)).unwrap().bg, Color::Rgb(0, 255, 0));

        // Cell (0,1): top=blue, bottom=black (no pixel)
        assert_eq!(buf.cell((0, 1)).unwrap().fg, Color::Rgb(0, 0, 255));
        assert_eq!(buf.cell((0, 1)).unwrap().bg, Color::Black);
    }

    #[test]
    fn render_image_halfblock_empty() {
        let area = Rect::new(0, 0, 1, 1);
        let mut buf = Buffer::empty(area);
        // Should not panic with empty data
        render_image_halfblock(&mut buf, 0, 0, 0, 0, &[]);
        render_image_halfblock(&mut buf, 0, 0, 1, 1, &[]);
    }
}
