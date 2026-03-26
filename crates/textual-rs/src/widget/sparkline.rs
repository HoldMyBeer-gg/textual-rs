use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::context::AppContext;
use super::Widget;
use crate::reactive::Reactive;

/// A sparkline widget that renders data points as block characters.
///
/// Data is normalized to 8 levels (0–7) using the block character set "▁▂▃▄▅▆▇█".
/// One character is rendered per data point, clipped to the widget width.
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

/// The 8 block level characters, index 0 = lowest, index 7 = highest.
const BLOCK_CHARS: &str = "▁▂▃▄▅▆▇█";

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

        let width = area.width as usize;
        let blocks: Vec<char> = BLOCK_CHARS.chars().collect();

        // Find max value for normalization; use 1.0 if all zeros to avoid divide-by-zero
        let max_val = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let max_val = if max_val <= 0.0 { 1.0 } else { max_val };

        let text: String = data
            .iter()
            .take(width)
            .map(|&v| {
                // Normalize to 0.0–1.0 range (min anchored at 0)
                let normalized = (v.max(0.0) / max_val).clamp(0.0, 1.0);
                // Map to block index 0–7
                let idx = (normalized * 7.0).round() as usize;
                let idx = idx.min(7);
                blocks[idx]
            })
            .collect();

        buf.set_string(area.x, area.y, &text, Style::default());
    }
}
