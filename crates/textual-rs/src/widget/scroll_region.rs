//! Scrollable viewport for direct buffer rendering.
//!
//! Unlike [`ScrollView`](super::scroll_view::ScrollView) — which blits child widgets from a
//! virtual buffer — `ScrollRegion` is designed for widgets that do their own direct
//! buffer writes. Implement [`ScrollContent`] to provide the content height and a
//! `render_content` function; wrap it in a `ScrollRegion` to get keyboard/mouse
//! scrolling and a scrollbar for free.
//!
//! # Example
//!
//! ```no_run
//! use textual_rs::widget::scroll_region::{ScrollContent, ScrollRegion};
//! use textual_rs::widget::context::AppContext;
//! use ratatui::{buffer::Buffer, layout::Rect};
//!
//! struct MyList { items: Vec<String> }
//!
//! impl ScrollContent for MyList {
//!     fn content_height(&self) -> usize { self.items.len() }
//!
//!     fn render_content(&self, _ctx: &AppContext, buf: &mut Buffer, area: Rect, scroll_offset: usize) {
//!         for (i, item) in self.items.iter().enumerate().skip(scroll_offset).take(area.height as usize) {
//!             let row = (i - scroll_offset) as u16;
//!             buf.set_string(area.x, area.y + row, item, ratatui::style::Style::default());
//!         }
//!     }
//! }
//!
//! let region = ScrollRegion::new(Box::new(MyList { items: vec!["one".into(), "two".into()] }));
//! ```

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::cell::Cell;

use super::context::AppContext;
use super::Widget;
use crate::event::keybinding::KeyBinding;

/// Trait for types that render directly into a buffer inside a `ScrollRegion`.
///
/// Implement this instead of [`Widget`] when you need full control over
/// buffer writes but still want scrolling behaviour.
pub trait ScrollContent {
    /// Total number of rows in the virtual content (not the viewport height).
    fn content_height(&self) -> usize;

    /// Render visible rows into `buf`.
    ///
    /// `area` is the clipped viewport rect (already excludes the scrollbar column).
    /// `scroll_offset` is the first content row that should appear at `area.y`.
    /// Only rows `[scroll_offset .. scroll_offset + area.height]` need to be drawn.
    fn render_content(&self, ctx: &AppContext, buf: &mut Buffer, area: Rect, scroll_offset: usize);
}

/// A scrollable viewport that delegates rendering to a [`ScrollContent`] impl.
///
/// Handles Up/Down/PageUp/PageDown key bindings and mouse wheel events.
/// Draws a vertical scrollbar when content exceeds the viewport.
pub struct ScrollRegion {
    /// The content being scrolled.
    pub content: Box<dyn ScrollContent>,
    /// Current scroll offset (first visible row index).
    pub scroll_offset: Cell<usize>,
    viewport_height: Cell<u16>,
}

impl ScrollRegion {
    /// Create a new `ScrollRegion` wrapping the given content.
    pub fn new(content: Box<dyn ScrollContent>) -> Self {
        Self {
            content,
            scroll_offset: Cell::new(0),
            viewport_height: Cell::new(0),
        }
    }

    /// Scroll to the last row (convenience for "tail" / auto-scroll behaviour).
    pub fn scroll_to_bottom(&self) {
        let viewport = self.viewport_height.get() as usize;
        let max = self.content.content_height().saturating_sub(viewport);
        self.scroll_offset.set(max);
    }

    /// Scroll to the first row.
    pub fn scroll_to_top(&self) {
        self.scroll_offset.set(0);
    }

    fn max_offset(&self) -> usize {
        self.content
            .content_height()
            .saturating_sub(self.viewport_height.get() as usize)
    }
}

static SCROLL_REGION_BINDINGS: &[KeyBinding] = &[
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
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: "scroll_top",
        description: "Scroll to top",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: "scroll_bottom",
        description: "Scroll to bottom",
        show: false,
    },
];

impl Widget for ScrollRegion {
    fn widget_type_name(&self) -> &'static str {
        "ScrollRegion"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "ScrollRegion { flex-grow: 1; }"
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        SCROLL_REGION_BINDINGS
    }

    fn on_action(&self, action: &str, _ctx: &AppContext) {
        let offset = self.scroll_offset.get();
        let viewport = self.viewport_height.get() as usize;
        let max = self.max_offset();

        match action {
            "scroll_up" => self.scroll_offset.set(offset.saturating_sub(1)),
            "scroll_down" => self.scroll_offset.set((offset + 1).min(max)),
            "page_up" => self.scroll_offset.set(offset.saturating_sub(viewport)),
            "page_down" => self.scroll_offset.set((offset + viewport).min(max)),
            "scroll_top" => self.scroll_offset.set(0),
            "scroll_bottom" => self.scroll_offset.set(max),
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        self.viewport_height.set(area.height);

        let offset = self.scroll_offset.get();
        let content_height = self.content.content_height();
        let needs_scrollbar = content_height > area.height as usize;

        // Reserve the rightmost column for the scrollbar when needed
        let content_area = if needs_scrollbar && area.width > 1 {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width - 1,
                height: area.height,
            }
        } else {
            area
        };

        self.content.render_content(ctx, buf, content_area, offset);

        if needs_scrollbar {
            let bar_color = ratatui::style::Color::Rgb(0, 255, 163);
            let track_color = ratatui::style::Color::Rgb(30, 30, 40);
            crate::canvas::vertical_scrollbar(
                buf,
                area.x + area.width - 1,
                area.y,
                area.height,
                content_height,
                area.height as usize,
                offset,
                bar_color,
                track_color,
            );
        }
    }
}
