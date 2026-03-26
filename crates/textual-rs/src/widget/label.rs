use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::context::AppContext;
use super::Widget;

/// A widget that renders static text.
pub struct Label {
    pub text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for Label {
    fn widget_type_name(&self) -> &'static str {
        "Label"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Label { min-height: 1; }"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let text: &str = &self.text;
        let max_chars = area.width as usize;
        let display: String = text.chars().take(max_chars).collect();
        // Inherit style from buffer (set by paint_chrome)
        let style = buf.cell((area.x, area.y)).map(|c| c.style()).unwrap_or_default();
        buf.set_string(area.x, area.y, &display, style);
    }
}
