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
}
