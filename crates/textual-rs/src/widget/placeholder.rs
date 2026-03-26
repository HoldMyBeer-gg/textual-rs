use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::context::AppContext;
use super::Widget;

/// A placeholder widget for development use that shows its type name and area dimensions.
pub struct Placeholder {
    pub label: Option<String>,
}

impl Placeholder {
    pub fn new() -> Self {
        Self { label: None }
    }

    pub fn with_label(label: impl Into<String>) -> Self {
        Self { label: Some(label.into()) }
    }
}

impl Default for Placeholder {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Placeholder {
    fn widget_type_name(&self) -> &'static str {
        "Placeholder"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Placeholder { border: rounded; min-height: 3; min-width: 10; }"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let label = self.label.as_deref().unwrap_or("Placeholder");
        let dimensions = format!("{}x{}", area.width, area.height);

        // Render label on first line, dimensions on second line
        // Center both within the area
        let render_line = |buf: &mut Buffer, y: u16, text: &str| {
            let text_len = text.chars().count() as u16;
            let x = if area.width > text_len {
                area.x + (area.width - text_len) / 2
            } else {
                area.x
            };
            let display: String = text.chars().take(area.width as usize).collect();
            buf.set_string(x, y, &display, Style::default());
        };

        // If we have height, render label on first available row
        let mid_y = area.y + area.height / 2;

        if area.height == 1 {
            // Single row: show just the label
            render_line(buf, area.y, label);
        } else {
            // Multi-row: show label above center, dimensions below
            let label_y = if mid_y > area.y { mid_y - 1 } else { area.y };
            let dim_y = mid_y;
            render_line(buf, label_y, label);
            if dim_y < area.y + area.height {
                render_line(buf, dim_y, &dimensions);
            }
        }
    }
}
