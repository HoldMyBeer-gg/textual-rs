//! Scrollable styled-text log widget that accepts ratatui `Line` objects.
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use std::cell::Cell;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// A scrolling log display widget for styled [`ratatui::text::Line`] objects.
///
/// Unlike [`super::log::Log`] which displays plain strings, `RichLog` accepts
/// `Line<'static>` values carrying full ratatui span styling — colors, bold,
/// italic, and other modifiers.
///
/// New lines are appended via [`RichLog::write_line`]. By default the view
/// auto-scrolls to the bottom on each new line. Pressing Up disables
/// auto-scroll; pressing End or scrolling to the bottom re-enables it.
///
/// # Example
///
/// ```no_run
/// use textual_rs::RichLog;
/// use ratatui::style::{Color, Style};
/// use ratatui::text::{Line, Span};
///
/// let log = RichLog::new();
/// log.write_line(Line::from(vec![
///     Span::styled("INFO", Style::default().fg(Color::Green)),
///     Span::raw(" Server started"),
/// ]));
/// ```
pub struct RichLog {
    /// The stored styled lines.
    pub lines: Reactive<Vec<Line<'static>>>,
    /// Current scroll offset in lines from the top.
    pub scroll_offset: Reactive<usize>,
    auto_scroll: Cell<bool>,
    /// Measured height of the rendered viewport, in terminal rows.
    pub viewport_height: Cell<u16>,
    own_id: Cell<Option<WidgetId>>,
    /// Maximum number of lines to retain. Oldest lines are evicted when exceeded.
    pub max_lines: Option<usize>,
}

impl RichLog {
    /// Create a new `RichLog` with no line limit and auto-scroll enabled.
    pub fn new() -> Self {
        Self {
            lines: Reactive::new(Vec::new()),
            scroll_offset: Reactive::new(0),
            auto_scroll: Cell::new(true),
            viewport_height: Cell::new(0),
            own_id: Cell::new(None),
            max_lines: None,
        }
    }

    /// Create a new `RichLog` that evicts the oldest line once `max` lines are stored.
    pub fn with_max_lines(max: usize) -> Self {
        Self {
            lines: Reactive::new(Vec::new()),
            scroll_offset: Reactive::new(0),
            auto_scroll: Cell::new(true),
            viewport_height: Cell::new(0),
            own_id: Cell::new(None),
            max_lines: Some(max),
        }
    }

    /// Append a styled line to the log.
    ///
    /// If `max_lines` is set and the buffer is full, the oldest line is removed
    /// and `scroll_offset` is decremented (to keep the view stable). Then the
    /// new line is pushed and, when auto-scroll is enabled, the offset is
    /// advanced to keep the last line visible.
    pub fn write_line(&self, line: Line<'static>) {
        // Evict oldest line if max_lines is set and at capacity
        if let Some(max) = self.max_lines {
            let len = self.lines.get_untracked().len();
            if len >= max {
                self.lines.update(|v| {
                    v.drain(0..1);
                });
                // Adjust scroll_offset so the view doesn't jump
                let offset = self.scroll_offset.get_untracked();
                if offset > 0 {
                    self.scroll_offset.set(offset - 1);
                }
            }
        }

        self.lines.update(|v| v.push(line));

        if self.auto_scroll.get() {
            let line_count = self.lines.get_untracked().len();
            let viewport_h = self.viewport_height.get() as usize;
            // Only auto-scroll once viewport has been measured (after first render)
            if viewport_h > 0 && line_count > viewport_h {
                self.scroll_offset.set(line_count - viewport_h);
            }
        }
    }

    /// Clear all lines and reset scroll to the top.
    pub fn clear(&self) {
        self.lines.update(|v| v.clear());
        self.scroll_offset.set(0);
        self.auto_scroll.set(true);
    }
}

impl Default for RichLog {
    fn default() -> Self {
        Self::new()
    }
}

static RICH_LOG_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: "scroll_up",
        description: "Scroll up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: "scroll_down",
        description: "Scroll down",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: "scroll_top",
        description: "Top",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: "scroll_bottom",
        description: "Bottom",
        show: false,
    },
    KeyBinding {
        key: KeyCode::PageUp,
        modifiers: KeyModifiers::NONE,
        action: "page_up",
        description: "Page up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::PageDown,
        modifiers: KeyModifiers::NONE,
        action: "page_down",
        description: "Page down",
        show: false,
    },
];

impl Widget for RichLog {
    fn widget_type_name(&self) -> &'static str {
        "RichLog"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "RichLog { min-height: 3; flex-grow: 1; }"
    }

    fn widget_default_css(&self) -> &'static str {
        "RichLog { min-height: 3; flex-grow: 1; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        RICH_LOG_BINDINGS
    }

    fn on_action(&self, action: &str, _ctx: &AppContext) {
        let offset = self.scroll_offset.get_untracked();
        let line_count = self.lines.get_untracked().len();
        let viewport_h = self.viewport_height.get() as usize;

        match action {
            "scroll_up" => {
                if offset > 0 {
                    self.scroll_offset.set(offset - 1);
                }
                self.auto_scroll.set(false);
            }
            "scroll_down" => {
                let max_offset = line_count.saturating_sub(viewport_h);
                let new_offset = (offset + 1).min(max_offset);
                self.scroll_offset.set(new_offset);
                if viewport_h > 0 && new_offset + viewport_h >= line_count {
                    self.auto_scroll.set(true);
                }
            }
            "scroll_top" => {
                self.scroll_offset.set(0);
                self.auto_scroll.set(false);
            }
            "scroll_bottom" => {
                if line_count > viewport_h {
                    self.scroll_offset.set(line_count - viewport_h);
                } else {
                    self.scroll_offset.set(0);
                }
                self.auto_scroll.set(true);
            }
            "page_up" => {
                let page = viewport_h.max(1);
                self.scroll_offset.set(offset.saturating_sub(page));
                self.auto_scroll.set(false);
            }
            "page_down" => {
                let page = viewport_h.max(1);
                let max_offset = line_count.saturating_sub(viewport_h);
                let new_offset = (offset + page).min(max_offset);
                self.scroll_offset.set(new_offset);
                if viewport_h > 0 && new_offset + viewport_h >= line_count {
                    self.auto_scroll.set(true);
                }
            }
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Store viewport height for action handlers and write_line
        self.viewport_height.set(area.height);

        let lines = self.lines.get_untracked();
        let offset = self.scroll_offset.get_untracked();
        let count = lines.len();

        // Reserve last column for scrollbar when there is overflow
        let text_width = if area.width > 1 { area.width - 1 } else { area.width };

        // Draw visible lines using buf.set_line for styled output
        let visible_count = (area.height as usize).min(count.saturating_sub(offset));
        for row in 0..visible_count {
            let line_idx = offset + row;
            let y = area.y + row as u16;
            buf.set_line(area.x, y, &lines[line_idx], text_width);
        }

        // Draw sub-cell vertical scrollbar in rightmost column when content overflows
        if count > area.height as usize && area.width > 0 {
            let scroll_x = area.x + area.width - 1;
            let bar_color = ratatui::style::Color::Rgb(0, 255, 163); // accent green
            let track_color = ratatui::style::Color::Rgb(30, 30, 40);
            crate::canvas::vertical_scrollbar(
                buf,
                scroll_x,
                area.y,
                area.height,
                count,
                area.height as usize,
                offset,
                bar_color,
                track_color,
            );
        }
    }
}
