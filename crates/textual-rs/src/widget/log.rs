use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// A scrolling log display widget.
///
/// New lines are appended via `push_line()`. By default the view auto-scrolls
/// to the bottom on each new line. Pressing Up disables auto-scroll; pressing
/// End or scrolling to the bottom re-enables it.
pub struct Log {
    pub lines: Reactive<Vec<String>>,
    pub scroll_offset: Reactive<usize>,
    auto_scroll: Cell<bool>,
    viewport_height: Cell<u16>,
    own_id: Cell<Option<WidgetId>>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            lines: Reactive::new(Vec::new()),
            scroll_offset: Reactive::new(0),
            auto_scroll: Cell::new(true),
            viewport_height: Cell::new(0),
            own_id: Cell::new(None),
        }
    }

    /// Append a line to the log.
    ///
    /// If auto-scroll is enabled the scroll offset is advanced to keep the
    /// last line visible.
    pub fn push_line(&self, line: String) {
        self.lines.update(|v| v.push(line));
        if self.auto_scroll.get() {
            let line_count = self.lines.get_untracked().len();
            let viewport_h = self.viewport_height.get() as usize;
            // Only auto-scroll if viewport has been measured (after first render)
            if viewport_h > 0 && line_count > viewport_h {
                self.scroll_offset.set(line_count - viewport_h);
            }
        }
    }
}

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}

static LOG_BINDINGS: &[KeyBinding] = &[
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
];

impl Widget for Log {
    fn widget_type_name(&self) -> &'static str {
        "Log"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Log { min-height: 3; flex-grow: 1; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        LOG_BINDINGS
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
                // Manual scroll up disables auto-scroll
                self.auto_scroll.set(false);
            }
            "scroll_down" => {
                let new_offset = offset + 1;
                self.scroll_offset.set(new_offset);
                // If we scrolled to the bottom, re-enable auto-scroll
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
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        // Store viewport height for action handlers and push_line
        self.viewport_height.set(area.height);

        let lines = self.lines.get_untracked();
        let offset = self.scroll_offset.get_untracked();
        let count = lines.len();

        // Draw visible lines
        let visible_count = (area.height as usize).min(count.saturating_sub(offset));
        for row in 0..visible_count {
            let line_idx = offset + row;
            let y = area.y + row as u16;
            // Reserve last column for scrollbar
            let text_width = if area.width > 1 { area.width - 1 } else { area.width };
            let line_text: String = lines[line_idx].chars().take(text_width as usize).collect();
            buf.set_string(area.x, y, &line_text, style);
        }

        // Draw vertical scrollbar in rightmost column
        if count > area.height as usize && area.width > 0 {
            let max_offset = count - area.height as usize;
            let scroll_x = area.x + area.width - 1;
            for row in 0..area.height {
                let y = area.y + row;
                let thumb_row = if max_offset > 0 {
                    (offset as f32 / max_offset as f32 * (area.height - 1) as f32) as u16
                } else {
                    0
                };
                let ch = if row == thumb_row { "█" } else { "│" };
                buf.set_string(scroll_x, y, ch, style);
            }
        }
    }
}
