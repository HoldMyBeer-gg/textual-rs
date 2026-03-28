//! Development placeholder widget that shows a cross-hatch fill and dimensions.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::context::AppContext;
use super::Widget;

/// A placeholder widget for development use that shows its type name and area dimensions.
pub struct Placeholder {
    /// Optional custom label text; defaults to `"Placeholder"` when `None`.
    pub label: Option<String>,
}

impl Placeholder {
    /// Create a new Placeholder with no custom label.
    pub fn new() -> Self {
        Self { label: None }
    }

    /// Create a Placeholder with a custom label string.
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
        }
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
        use ratatui::style::{Color, Modifier, Style};

        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = buf
            .cell((area.x, area.y))
            .map(|c| c.style())
            .unwrap_or_default();
        let hatch_fg = Color::Rgb(40, 40, 55);
        let hatch_bg = base_style.bg.unwrap_or(Color::Rgb(20, 20, 30));

        // Fill with cross-hatch pattern using quadrant block characters.
        // Alternating anti-diagonal (▚) and diagonal (▞) quadrant masks
        // create a textured checkerboard at 2x2 sub-cell resolution.
        let pattern_a: u8 = 0b1001; // anti-diagonal: top-left + bottom-right
        let pattern_b: u8 = 0b0110; // diagonal: top-right + bottom-left
        for row in 0..area.height {
            for col in 0..area.width {
                let pattern = if (row + col) % 2 == 0 {
                    pattern_a
                } else {
                    pattern_b
                };
                crate::canvas::quadrant_cell(
                    buf,
                    area.x + col,
                    area.y + row,
                    pattern,
                    hatch_fg,
                    hatch_bg,
                );
            }
        }

        // Overlay label and dimensions centered
        let label = self.label.as_deref().unwrap_or("Placeholder");
        let dimensions = format!("{}×{}", area.width, area.height);

        let label_style = Style::default()
            .fg(Color::Rgb(140, 140, 160))
            .bg(hatch_bg)
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(Color::Rgb(90, 90, 110)).bg(hatch_bg);

        let mid_y = area.y + area.height / 2;

        let center_x = |text: &str| -> u16 {
            let len = text.chars().count() as u16;
            if area.width > len {
                area.x + (area.width - len) / 2
            } else {
                area.x
            }
        };

        if area.height == 1 {
            let x = center_x(label);
            let display: String = label.chars().take(area.width as usize).collect();
            buf.set_string(x, area.y, &display, label_style);
        } else {
            let label_y = if mid_y > area.y { mid_y - 1 } else { area.y };
            let dim_y = mid_y;
            let x = center_x(label);
            let display: String = label.chars().take(area.width as usize).collect();
            buf.set_string(x, label_y, &display, label_style);
            if dim_y < area.y + area.height {
                let x = center_x(&dimensions);
                let display: String = dimensions.chars().take(area.width as usize).collect();
                buf.set_string(x, dim_y, &display, dim_style);
            }
        }
    }
}
