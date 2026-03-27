use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::context::AppContext;
use super::Widget;
use crate::reactive::Reactive;

/// A sparkline widget that renders data points using braille characters for high-resolution
/// visualization (2x4 dots per cell = 8 sub-pixels).
///
/// When height > 1, uses braille for smooth curves across multiple rows.
/// When height == 1, falls back to eighth-block characters for maximum density.
pub struct Sparkline {
    pub data: Reactive<Vec<f64>>,
}

impl Sparkline {
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            data: Reactive::new(data),
        }
    }
}

impl Widget for Sparkline {
    fn widget_type_name(&self) -> &'static str {
        "Sparkline"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Sparkline { height: 1; }"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let data = self.data.get_untracked();
        if data.is_empty() {
            return;
        }

        let style = buf.cell((area.x, area.y)).map(|c| c.style()).unwrap_or_default();
        let fg = style.fg.unwrap_or(Color::Rgb(0, 212, 255));
        let bg = style.bg.unwrap_or(Color::Reset);

        if area.height == 1 {
            // Single row: use eighth-block characters for dense 1D sparkline
            render_eighth_block(&data, area, buf, fg, bg);
        } else {
            // Multi-row: use braille for high-resolution 2D sparkline
            render_braille(&data, area, buf, fg, bg);
        }
    }
}

/// Single-row sparkline using eighth-block characters (▁▂▃▄▅▆▇█)
fn render_eighth_block(data: &[f64], area: Rect, buf: &mut Buffer, fg: Color, bg: Color) {
    const BLOCKS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_val = if max_val <= 0.0 { 1.0 } else { max_val };
    let width = area.width as usize;

    for (i, &v) in data.iter().take(width).enumerate() {
        let normalized = (v.max(0.0) / max_val).clamp(0.0, 1.0);
        let idx = (normalized * 7.0).round() as usize;
        let block = BLOCKS[idx.min(7)];
        let style = ratatui::style::Style::default().fg(fg).bg(bg);
        buf.set_string(area.x + i as u16, area.y, block, style);
    }
}

/// Multi-row sparkline using braille characters (2x4 dots per cell)
/// Each column of data maps to 2 braille columns (left dot column).
/// Each row of cells provides 4 vertical dot positions.
/// Total vertical resolution = area.height * 4 dot rows.
fn render_braille(data: &[f64], area: Rect, buf: &mut Buffer, fg: Color, bg: Color) {
    let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_val = if max_val <= 0.0 { 1.0 } else { max_val };

    // Each cell is 2 dots wide, 4 dots tall
    let cell_cols = area.width as usize;
    let cell_rows = area.height as usize;
    let dot_rows = cell_rows * 4;

    // We'll use 1 data point per cell column (left dot column filled, right empty)
    // This gives width data points displayed across the sparkline
    let width = cell_cols;

    // Normalize all data points to dot-row positions (0 = bottom, dot_rows-1 = top)
    let points: Vec<usize> = data
        .iter()
        .take(width)
        .map(|&v| {
            let normalized = (v.max(0.0) / max_val).clamp(0.0, 1.0);
            let dot_y = (normalized * (dot_rows as f64 - 1.0)).round() as usize;
            dot_y.min(dot_rows - 1)
        })
        .collect();

    // Build a grid of braille dot patterns
    // grid[cell_row][cell_col] = 8-bit dot mask
    let mut grid = vec![vec![0u8; cell_cols]; cell_rows];

    for (col, &dot_y) in points.iter().enumerate() {
        if col >= cell_cols {
            break;
        }
        // dot_y is from bottom (0) to top (dot_rows-1)
        // Convert to cell_row and dot position within cell
        // Top of display = cell_row 0, dot 0
        // Bottom of display = cell_row (cell_rows-1), dot 3
        let inverted_y = (dot_rows - 1) - dot_y; // flip so 0=top
        let cell_row = inverted_y / 4;
        let dot_in_cell = inverted_y % 4; // 0=top, 3=bottom

        if cell_row < cell_rows {
            // Set the left-column dot (dx=0) at this dy position
            let dot_idx = crate::canvas::braille_dot_index(0, dot_in_cell as u8);
            grid[cell_row][col] |= 1 << dot_idx;

            // Also fill dots below this point to create a filled area effect
            // Fill from this dot down to the bottom of the grid
            for fill_y in (inverted_y + 1)..dot_rows {
                let fill_row = fill_y / 4;
                let fill_dot = fill_y % 4;
                if fill_row < cell_rows {
                    let fill_idx = crate::canvas::braille_dot_index(0, fill_dot as u8);
                    grid[fill_row][col] |= 1 << fill_idx;
                }
            }

            // Connect adjacent points with line segments using right dot column
            // If the next point exists, draw connecting dots
            if col + 1 < points.len() {
                let next_dot_y = points[col + 1];
                let next_inverted = (dot_rows - 1) - next_dot_y;
                // Interpolate between current and next for the right column (dx=1)
                let mid_inverted = (inverted_y + next_inverted) / 2;
                let mid_row = mid_inverted / 4;
                let mid_dot = mid_inverted % 4;
                if mid_row < cell_rows {
                    let mid_idx = crate::canvas::braille_dot_index(1, mid_dot as u8);
                    grid[mid_row][col] |= 1 << mid_idx;
                    // Fill below midpoint too
                    for fill_y in (mid_inverted + 1)..dot_rows {
                        let fill_row = fill_y / 4;
                        let fill_dot = fill_y % 4;
                        if fill_row < cell_rows {
                            let fill_idx = crate::canvas::braille_dot_index(1, fill_dot as u8);
                            grid[fill_row][col] |= 1 << fill_idx;
                        }
                    }
                }
            }
        }
    }

    // Render the grid
    let _style = ratatui::style::Style::default().fg(fg).bg(bg);
    for row in 0..cell_rows {
        for col in 0..cell_cols {
            let dots = grid[row][col];
            if dots != 0 {
                crate::canvas::braille_cell(
                    buf,
                    area.x + col as u16,
                    area.y + row as u16,
                    dots,
                    fg,
                    bg,
                );
            }
        }
    }
}
