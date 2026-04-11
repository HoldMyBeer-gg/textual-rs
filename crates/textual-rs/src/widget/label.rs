//! Non-interactive label widget that renders static text with optional OSC 8 hyperlinks.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cell::Cell;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::css::render_style::align_text;
use crate::hyperlink::{render_linked_line, LinkedLine, LinkedSpan};

/// A widget that renders static text, optionally with clickable OSC 8 hyperlinks.
///
/// # Plain text (unchanged API)
///
/// ```no_run
/// use textual_rs::Label;
/// let label = Label::new("Hello, world!");
/// ```
///
/// # Text with hyperlinks
///
/// ```no_run
/// use textual_rs::Label;
/// use textual_rs::hyperlink::LinkedSpan;
///
/// let label = Label::new_linked(vec![
///     LinkedSpan::plain("Visit "),
///     LinkedSpan::linked("docs.rs", "https://docs.rs"),
/// ]);
/// ```
///
/// # Breaking change (0.3.12)
///
/// The `text: String` public field has been replaced by `spans: LinkedLine`.
/// Code that read `label.text` should call `label.text()` instead.
pub struct Label {
    /// The styled (and optionally linked) spans making up this label.
    pub spans: LinkedLine,
    own_id: Cell<Option<WidgetId>>,
    /// Optional CSS classes for styling via selectors (e.g., `.section-title`).
    css_classes: Vec<&'static str>,
}

impl Label {
    /// Create a new Label with plain text and no hyperlinks.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            spans: vec![LinkedSpan::plain(text)],
            own_id: Cell::new(None),
            css_classes: Vec::new(),
        }
    }

    /// Create a new Label from a `Vec<LinkedSpan>`, enabling per-span hyperlinks.
    pub fn new_linked(spans: Vec<LinkedSpan>) -> Self {
        Self {
            spans,
            own_id: Cell::new(None),
            css_classes: Vec::new(),
        }
    }

    /// Add a CSS class to this label (for styling via CSS selectors).
    pub fn with_class(mut self, class: &'static str) -> Self {
        self.css_classes.push(class);
        self
    }

    /// Return the concatenated plain text of all spans (strips URL data).
    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
}

impl Widget for Label {
    fn widget_type_name(&self) -> &'static str {
        "Label"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn classes(&self) -> &[&str] {
        &self.css_classes
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Label { min-height: 1; }"
    }

    fn widget_default_css(&self) -> &'static str {
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

        // Single plain span: apply text-align (preserves existing behaviour).
        // Multi-span or linked: render left-to-right without alignment padding.
        let is_plain_single = self.spans.len() == 1 && self.spans[0].url.is_none();

        if is_plain_single {
            let text = &self.spans[0].text;
            let max_chars = area.width as usize;
            let truncated: String = text.chars().take(max_chars).collect();

            let text_align = self
                .own_id
                .get()
                .and_then(|id| ctx.computed_styles.get(id))
                .map(|cs| cs.text_align)
                .unwrap_or(crate::css::types::TextAlign::Left);
            let display = align_text(&truncated, max_chars, text_align);

            let style = buf
                .cell((area.x, area.y))
                .map(|c| c.style())
                .unwrap_or_default();
            buf.set_string(area.x, area.y, &display, style);
        } else {
            render_linked_line(buf, area.x, area.y, &self.spans, area.width);
        }
    }
}
