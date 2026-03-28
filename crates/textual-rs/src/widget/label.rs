//! Non-interactive label widget that renders a static text string.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cell::Cell;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::css::render_style::align_text;

/// A widget that renders static text.
pub struct Label {
    /// The text string to display.
    pub text: String,
    own_id: Cell<Option<WidgetId>>,
}

impl Label {
    /// Create a new Label with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            own_id: Cell::new(None),
        }
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

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let text: &str = &self.text;
        let max_chars = area.width as usize;
        let truncated: String = text.chars().take(max_chars).collect();

        // Apply text-align from computed style
        let text_align = self
            .own_id
            .get()
            .and_then(|id| ctx.computed_styles.get(id))
            .map(|cs| cs.text_align)
            .unwrap_or(crate::css::types::TextAlign::Left);
        let display = align_text(&truncated, max_chars, text_align);

        // Inherit style from buffer (set by paint_chrome)
        let style = buf
            .cell((area.x, area.y))
            .map(|c| c.style())
            .unwrap_or_default();
        buf.set_string(area.x, area.y, &display, style);
    }
}
