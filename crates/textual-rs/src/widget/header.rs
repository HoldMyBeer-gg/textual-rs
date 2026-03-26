use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::context::AppContext;
use super::Widget;
use crate::reactive::Reactive;

/// A header bar widget that displays a title (and optional subtitle) docked at the top.
pub struct Header {
    pub title: Reactive<String>,
    pub subtitle: Reactive<String>,
}

impl Header {
    pub fn new(title: &str) -> Self {
        Self {
            title: Reactive::new(title.to_string()),
            subtitle: Reactive::new(String::new()),
        }
    }

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
        "Header { dock: top; height: 1; background: $primary; color: $text; }"
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
        buf.set_string(x, area.y, &display, Style::default());
    }
}
