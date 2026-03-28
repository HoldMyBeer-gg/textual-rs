//! Scrollable list widget with keyboard navigation and item selection.
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use std::cell::Cell;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a ListView.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the user presses Enter on a highlighted item.
    pub struct Selected {
        /// Zero-based index of the selected item.
        pub index: usize,
        /// String value of the selected item.
        pub value: String,
    }
    impl Message for Selected {}

    /// Emitted when the highlighted item changes (cursor moves).
    pub struct Highlighted {
        /// Zero-based index of the newly highlighted item.
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
    /// List items to display.
    pub items: Vec<String>,
    /// Zero-based index of the currently highlighted/selected item.
    pub selected: Reactive<usize>,
    /// Vertical scroll offset in lines from the top.
    pub scroll_offset: Reactive<usize>,
    viewport_height: Cell<u16>,
    own_id: Cell<Option<WidgetId>>,
    last_area_y: Cell<u16>,
}

impl ListView {
    /// Create a new ListView with the given items and initial selection at index 0.
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: Reactive::new(0),
            scroll_offset: Reactive::new(0),
            viewport_height: Cell::new(0),
            own_id: Cell::new(None),
            last_area_y: Cell::new(0),
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
    // Mouse wheel aliases — dispatch_scroll_action looks for these
    KeyBinding {
        key: KeyCode::Null,
        modifiers: KeyModifiers::NONE,
        action: "scroll_up",
        description: "Scroll up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Null,
        modifiers: KeyModifiers::NONE,
        action: "scroll_down",
        description: "Scroll down",
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
        "ListView { min-height: 3; flex-grow: 1; }"
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

    fn context_menu_items(&self) -> Vec<super::context_menu::ContextMenuItem> {
        vec![
            super::context_menu::ContextMenuItem::new("Copy Selected", "copy_selected")
                .with_shortcut("Ctrl+C"),
        ]
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> super::EventPropagation {
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        if let Some(m) = event.downcast_ref::<MouseEvent>() {
            if matches!(m.kind, MouseEventKind::Down(MouseButton::Left)) {
                let local_row = m.row.saturating_sub(self.last_area_y.get()) as usize;
                let offset = self.scroll_offset.get_untracked();
                let item_idx = offset + local_row;
                if item_idx < self.items.len() {
                    self.selected.set(item_idx);
                    if let Some(id) = self.own_id.get() {
                        ctx.post_message(id, messages::Highlighted { index: item_idx });
                        // Also fire Selected on click (single click = select)
                        let value = self.items[item_idx].clone();
                        ctx.post_message(
                            id,
                            messages::Selected {
                                index: item_idx,
                                value,
                            },
                        );
                    }
                    return super::EventPropagation::Stop;
                }
            }
        }
        super::EventPropagation::Continue
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
                    ctx.post_message(
                        id,
                        messages::Highlighted {
                            index: new_selected,
                        },
                    );
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
                    ctx.post_message(
                        id,
                        messages::Highlighted {
                            index: new_selected,
                        },
                    );
                }
            }
            "select" => {
                if count > 0 {
                    let value = self.items[current].clone();
                    ctx.post_message(
                        id,
                        messages::Selected {
                            index: current,
                            value,
                        },
                    );
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
            "scroll_up" => {
                let offset = self.scroll_offset.get_untracked();
                if offset > 0 {
                    self.scroll_offset.set(offset - 1);
                }
            }
            "scroll_down" => {
                let offset = self.scroll_offset.get_untracked();
                if viewport_h > 0 && count > viewport_h && offset < count - viewport_h {
                    self.scroll_offset.set(offset + 1);
                }
            }
            _ => {}
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        self.last_area_y.set(area.y);

        let style = self
            .own_id
            .get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

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
            let text_width = if area.width > 1 {
                area.width - 1
            } else {
                area.width
            };
            let item_text: String = self.items[item_idx]
                .chars()
                .take(text_width as usize)
                .collect();

            // Pad to text_width so selection highlight covers the whole row
            let padded = format!("{:<width$}", item_text, width = text_width as usize);

            if is_selected {
                let highlight_style = style
                    .fg(ratatui::style::Color::Rgb(0, 255, 163))
                    .add_modifier(Modifier::BOLD);
                buf.set_string(area.x, y, &padded, highlight_style);
            } else {
                buf.set_string(area.x, y, &padded, style);
            };
        }

        // Draw sub-cell vertical scrollbar in rightmost column
        if count > area.height as usize && area.width > 0 {
            let scroll_x = area.x + area.width - 1;
            let bar_color = ratatui::style::Color::Rgb(0, 255, 163);
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
