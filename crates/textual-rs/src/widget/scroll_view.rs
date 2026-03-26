use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// A scrollable container widget.
///
/// Children are rendered into a virtual buffer and the visible portion is blitted
/// into the actual terminal buffer based on `scroll_offset_x`/`scroll_offset_y`.
/// A vertical scrollbar is shown in the rightmost column when content exceeds the
/// viewport height, and a horizontal scrollbar in the bottom row when content
/// exceeds the viewport width.
pub struct ScrollView {
    pub scroll_offset_x: Reactive<usize>,
    pub scroll_offset_y: Reactive<usize>,
    pub children: Vec<Box<dyn Widget>>,
    /// Estimated content height — set by the caller. Defaults to sum of children (1 each for v1).
    pub content_height: usize,
    /// Estimated content width — set by the caller. Defaults to a wide virtual canvas.
    pub content_width: usize,
    viewport_width: Cell<u16>,
    viewport_height: Cell<u16>,
}

impl ScrollView {
    pub fn new(children: Vec<Box<dyn Widget>>) -> Self {
        let content_height = children.len().max(1);
        Self {
            scroll_offset_x: Reactive::new(0),
            scroll_offset_y: Reactive::new(0),
            children,
            content_height,
            content_width: 200, // wide virtual canvas for horizontal scrolling
            viewport_width: Cell::new(0),
            viewport_height: Cell::new(0),
        }
    }

    /// Set content height explicitly (needed when children render taller than 1 row each).
    pub fn with_content_height(mut self, h: usize) -> Self {
        self.content_height = h;
        self
    }

    /// Set content width explicitly.
    pub fn with_content_width(mut self, w: usize) -> Self {
        self.content_width = w;
        self
    }
}

static SCROLL_VIEW_BINDINGS: &[KeyBinding] = &[
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
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "scroll_left",
        description: "Scroll left",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "scroll_right",
        description: "Scroll right",
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

impl Widget for ScrollView {
    fn widget_type_name(&self) -> &'static str {
        "ScrollView"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "ScrollView { overflow: auto; }"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        // ScrollView renders children directly in render(), not via the widget tree compose path.
        // Returning empty here prevents double-registration in the arena.
        vec![]
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        SCROLL_VIEW_BINDINGS
    }

    fn on_action(&self, action: &str, _ctx: &AppContext) {
        let offset_y = self.scroll_offset_y.get_untracked();
        let offset_x = self.scroll_offset_x.get_untracked();
        let viewport_h = self.viewport_height.get() as usize;
        let viewport_w = self.viewport_width.get() as usize;

        let max_scroll_y = self.content_height.saturating_sub(viewport_h);
        let max_scroll_x = self.content_width.saturating_sub(viewport_w);

        match action {
            "scroll_up" => {
                self.scroll_offset_y.set(offset_y.saturating_sub(1));
            }
            "scroll_down" => {
                self.scroll_offset_y.set((offset_y + 1).min(max_scroll_y));
            }
            "scroll_left" => {
                self.scroll_offset_x.set(offset_x.saturating_sub(1));
            }
            "scroll_right" => {
                self.scroll_offset_x.set((offset_x + 1).min(max_scroll_x));
            }
            "page_up" => {
                self.scroll_offset_y.set(offset_y.saturating_sub(viewport_h));
            }
            "page_down" => {
                self.scroll_offset_y.set((offset_y + viewport_h).min(max_scroll_y));
            }
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Store viewport dimensions for action handlers
        self.viewport_height.set(area.height);
        self.viewport_width.set(area.width);

        let offset_y = self.scroll_offset_y.get_untracked();
        let offset_x = self.scroll_offset_x.get_untracked();

        // Determine if scrollbars are needed
        let need_vscroll = self.content_height > area.height as usize;
        let need_hscroll = self.content_width > area.width as usize;

        // Adjust effective viewport to leave room for scrollbars
        let render_w = if need_vscroll && area.width > 1 { area.width - 1 } else { area.width };
        let render_h = if need_hscroll && area.height > 1 { area.height - 1 } else { area.height };

        // Allocate a virtual buffer for the full content area
        let vbuf_w = self.content_width as u16;
        let vbuf_h = self.content_height as u16;

        if vbuf_w == 0 || vbuf_h == 0 {
            return;
        }

        let virtual_area = Rect { x: 0, y: 0, width: vbuf_w, height: vbuf_h };
        let mut virtual_buf = Buffer::empty(virtual_area);

        // Render each child into the virtual buffer, stacking vertically (1 child per row for v1)
        let mut child_y: u16 = 0;
        for child in &self.children {
            if child_y >= vbuf_h {
                break;
            }
            let child_area = Rect {
                x: 0,
                y: child_y,
                width: vbuf_w,
                height: 1,
            };
            child.render(ctx, child_area, &mut virtual_buf);
            child_y += 1;
        }

        // Blit the visible portion of the virtual buffer into the actual buffer
        let src_x_start = offset_x as u16;
        let src_y_start = offset_y as u16;

        for row in 0..render_h {
            for col in 0..render_w {
                let vx = src_x_start + col;
                let vy = src_y_start + row;
                if vx < vbuf_w && vy < vbuf_h {
                    let src_cell = virtual_buf[(vx, vy)].clone();
                    let dst_x = area.x + col;
                    let dst_y = area.y + row;
                    if dst_x < buf.area.right() && dst_y < buf.area.bottom() {
                        *buf.cell_mut((dst_x, dst_y)).unwrap() = src_cell;
                    }
                }
            }
        }

        // Draw vertical scrollbar
        if need_vscroll && area.width > 0 {
            let max_offset = self.content_height.saturating_sub(area.height as usize);
            let scroll_x = area.x + area.width - 1;
            for row in 0..area.height {
                let y = area.y + row;
                let thumb_row = if max_offset > 0 {
                    (offset_y as f32 / max_offset as f32 * (area.height - 1) as f32) as u16
                } else {
                    0
                };
                let ch = if row == thumb_row { "█" } else { "│" };
                buf.set_string(scroll_x, y, ch, Style::default());
            }
        }

        // Draw horizontal scrollbar
        if need_hscroll && area.height > 0 {
            let max_offset = self.content_width.saturating_sub(area.width as usize);
            let scroll_y = area.y + area.height - 1;
            for col in 0..render_w {
                let x = area.x + col;
                let thumb_col = if max_offset > 0 {
                    (offset_x as f32 / max_offset as f32 * (render_w - 1) as f32) as u16
                } else {
                    0
                };
                let ch = if col == thumb_col { "█" } else { "─" };
                buf.set_string(x, scroll_y, ch, Style::default());
            }
        }
    }
}
