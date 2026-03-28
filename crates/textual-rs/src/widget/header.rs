//! Header bar widget displaying a centered title and optional subtitle.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use super::context::AppContext;
use super::Widget;
use crate::canvas;
use crate::reactive::Reactive;

/// A header bar widget that displays a title (and optional subtitle) docked at the top.
pub struct Header {
    /// Main title text displayed centered in the header.
    pub title: Reactive<String>,
    /// Optional subtitle appended after the title with `" -- "` separator.
    pub subtitle: Reactive<String>,
}

impl Header {
    /// Create a new Header with the given title and no subtitle.
    pub fn new(title: &str) -> Self {
        Self {
            title: Reactive::new(title.to_string()),
            subtitle: Reactive::new(String::new()),
        }
    }

    /// Set a subtitle displayed after the title.
    pub fn with_subtitle(self, subtitle: &str) -> Self {
        self.subtitle.set(subtitle.to_string());
        self
    }
}

impl Widget for Header {
    fn widget_type_name(&self) -> &'static str {
        "Header"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Header { height: 1; background: $primary; color: $text; }"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let title = self.title.get_untracked();
        let subtitle = self.subtitle.get_untracked();

        let text = if subtitle.is_empty() {
            title
        } else {
            format!("{} -- {}", title, subtitle)
        };

        // Center the text in the area
        let text_len = text.chars().count() as u16;
        let x = if area.width > text_len {
            area.x + (area.width - text_len) / 2
        } else {
            area.x
        };

        let display: String = text.chars().take(area.width as usize).collect();
        let base_style = buf
            .cell((area.x, area.y))
            .map(|c| c.style())
            .unwrap_or_default();
        let style = base_style.add_modifier(ratatui::style::Modifier::BOLD);
        buf.set_string(x, area.y, &display, style);

        // Half-block bottom separator for depth.
        // For multi-row headers (height > 1), paint the last row as a gradient
        // separator using half_block_cell with a subtle color transition.
        // For single-row headers, apply a blended background to all cells for a
        // subtle depth feel without overwriting text content.
        if area.height > 1 {
            let header_bg = base_style.bg.unwrap_or(Color::Rgb(36, 36, 58));
            let sep_top = canvas::blend_color(header_bg, Color::Rgb(20, 20, 30), 0.5);
            let sep_bottom = Color::Reset;
            let sep_y = area.y + area.height - 1;
            for col in area.x..area.x + area.width {
                canvas::half_block_cell(buf, col, sep_y, sep_top, sep_bottom);
            }
        } else {
            // Single-row: blend the background of each cell for subtle depth.
            let header_bg = base_style.bg.unwrap_or(Color::Rgb(36, 36, 58));
            let depth_bg = canvas::blend_color(header_bg, Color::Rgb(20, 20, 30), 0.15);
            for col in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((col, area.y)) {
                    cell.set_bg(depth_bg);
                }
            }
        }
    }
}
