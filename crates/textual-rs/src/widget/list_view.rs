use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a ListView.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the user presses Enter on a highlighted item.
    pub struct Selected {
        pub index: usize,
        pub value: String,
    }
    impl Message for Selected {}

    /// Emitted when the highlighted item changes (cursor moves).
    pub struct Highlighted {
        pub index: usize,
    }
    impl Message for Highlighted {}
}

/// A scrollable, selectable list widget.
///
/// Users navigate with Up/Down arrow keys and select items with Enter.
/// The viewport scrolls automatically to keep the selected item visible.
/// A visual scrollbar is drawn in the rightmost column when content exceeds the viewport.
pub struct ListView {
    pub items: Vec<String>,
    pub selected: Reactive<usize>,
    pub scroll_offset: Reactive<usize>,
    viewport_height: Cell<u16>,
    own_id: Cell<Option<WidgetId>>,
}

impl ListView {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: Reactive::new(0),
            scroll_offset: Reactive::new(0),
            viewport_height: Cell::new(0),
            own_id: Cell::new(None),
        }
    }
}

static LIST_VIEW_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: "cursor_up",
        description: "Move up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: "cursor_down",
        description: "Move down",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "select",
        description: "Select",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Home,
        modifiers: KeyModifiers::NONE,
        action: "cursor_home",
        description: "First item",
        show: false,
    },
    KeyBinding {
        key: KeyCode::End,
        modifiers: KeyModifiers::NONE,
        action: "cursor_end",
        description: "Last item",
        show: false,
    },
];

impl Widget for ListView {
    fn widget_type_name(&self) -> &'static str {
        "ListView"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "ListView { border: tall; min-height: 3; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        LIST_VIEW_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        let Some(id) = self.own_id.get() else { return };
        let current = self.selected.get_untracked();
        let offset = self.scroll_offset.get_untracked();
        let viewport_h = self.viewport_height.get() as usize;
        let count = self.items.len();

        match action {
            "cursor_up" => {
                if current > 0 {
                    let new_selected = current - 1;
                    self.selected.set(new_selected);
                    // Scroll up if selected moves above the viewport
                    if new_selected < offset {
                        self.scroll_offset.set(new_selected);
                    }
                    ctx.post_message(id, messages::Highlighted { index: new_selected });
                }
            }
            "cursor_down" => {
                if count > 0 && current < count - 1 {
                    let new_selected = current + 1;
                    self.selected.set(new_selected);
                    // Scroll down if selected moves below the viewport
                    if viewport_h > 0 && new_selected >= offset + viewport_h {
                        self.scroll_offset.set(new_selected - viewport_h + 1);
                    }
                    ctx.post_message(id, messages::Highlighted { index: new_selected });
                }
            }
            "select" => {
                if count > 0 {
                    let value = self.items[current].clone();
                    ctx.post_message(id, messages::Selected { index: current, value });
                }
            }
            "cursor_home" => {
                self.selected.set(0);
                self.scroll_offset.set(0);
                ctx.post_message(id, messages::Highlighted { index: 0 });
            }
            "cursor_end" => {
                if count > 0 {
                    let last = count - 1;
                    self.selected.set(last);
                    if viewport_h > 0 && last >= viewport_h {
                        self.scroll_offset.set(last - viewport_h + 1);
                    } else {
                        self.scroll_offset.set(0);
                    }
                    ctx.post_message(id, messages::Highlighted { index: last });
                }
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Store viewport height for action handlers
        self.viewport_height.set(area.height);

        let selected = self.selected.get_untracked();
        let offset = self.scroll_offset.get_untracked();
        let count = self.items.len();

        // Draw visible items
        let visible_count = (area.height as usize).min(count.saturating_sub(offset));
        for row in 0..visible_count {
            let item_idx = offset + row;
            let y = area.y + row as u16;
            let is_selected = item_idx == selected;

            // Reserve last column for scrollbar
            let text_width = if area.width > 1 { area.width - 1 } else { area.width };
            let item_text: String = self.items[item_idx].chars().take(text_width as usize).collect();

            // Pad to text_width so selection highlight covers the whole row
            let padded = format!("{:<width$}", item_text, width = text_width as usize);

            let style = if is_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            buf.set_string(area.x, y, &padded, style);
        }

        // Draw scrollbar in rightmost column
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
                buf.set_string(scroll_x, y, ch, Style::default());
            }
        }
    }
}
