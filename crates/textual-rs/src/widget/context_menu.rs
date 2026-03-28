//! Right-click context menu overlay widget and supporting types.
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use std::cell::Cell;

use super::context::AppContext;
use super::{EventPropagation, Widget, WidgetId};
use crate::event::keybinding::KeyBinding;

/// A single item in a context menu.
#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    /// Display label shown in the menu.
    pub label: String,
    /// Action string dispatched to the source widget when this item is activated.
    pub action: String,
    /// Optional keyboard shortcut hint shown right-aligned in the menu.
    pub shortcut: Option<String>,
}

impl ContextMenuItem {
    /// Create a new ContextMenuItem with a label and action, no shortcut.
    pub fn new(label: &str, action: &str) -> Self {
        Self {
            label: label.to_string(),
            action: action.to_string(),
            shortcut: None,
        }
    }

    /// Set the optional shortcut hint displayed on the right side of the item.
    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }
}

/// Context menu overlay widget. Spawned on right-click, positioned at click coordinates.
/// Renders as a floating panel with selectable items.
pub(crate) struct ContextMenuOverlay {
    pub items: Vec<ContextMenuItem>,
    pub cursor: Cell<usize>,
    /// The widget that was right-clicked (receives the action).
    pub source_id: Option<WidgetId>,
    /// Screen position where the menu should appear.
    pub anchor_x: u16,
    pub anchor_y: u16,
    last_area: Cell<(u16, u16, u16, u16)>, // x, y, w, h for mouse hit testing
}

impl ContextMenuOverlay {
    pub fn new(
        items: Vec<ContextMenuItem>,
        source_id: Option<WidgetId>,
        anchor_x: u16,
        anchor_y: u16,
    ) -> Self {
        Self {
            items,
            cursor: Cell::new(0),
            source_id,
            anchor_x,
            anchor_y,
            last_area: Cell::new((0, 0, 0, 0)),
        }
    }
}

static CONTEXT_MENU_BINDINGS: &[KeyBinding] = &[
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
        action: "execute",
        description: "Execute",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        action: "close",
        description: "Close",
        show: false,
    },
];

impl Widget for ContextMenuOverlay {
    fn widget_type_name(&self) -> &'static str {
        "ContextMenu"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn is_overlay(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    fn widget_default_css(&self) -> &'static str {
        ""
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        CONTEXT_MENU_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(m) = event.downcast_ref::<MouseEvent>() {
            match m.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    let (ax, ay, aw, ah) = self.last_area.get();
                    // Click inside menu — select item
                    if m.column >= ax && m.column < ax + aw && m.row >= ay && m.row < ay + ah {
                        let local_row = (m.row - ay) as usize;
                        if local_row < self.items.len() {
                            self.cursor.set(local_row);
                            self.on_action("execute", ctx);
                            return EventPropagation::Stop;
                        }
                    }
                    // Click outside menu — close it
                    ctx.dismiss_overlay();
                    return EventPropagation::Stop;
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    // Right-click anywhere closes the menu
                    ctx.dismiss_overlay();
                    return EventPropagation::Stop;
                }
                _ => {}
            }
        }
        EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "cursor_up" => {
                let c = self.cursor.get();
                if c > 0 {
                    self.cursor.set(c - 1);
                } else {
                    self.cursor.set(self.items.len().saturating_sub(1));
                }
            }
            "cursor_down" => {
                let c = self.cursor.get();
                if c + 1 < self.items.len() {
                    self.cursor.set(c + 1);
                } else {
                    self.cursor.set(0);
                }
            }
            "execute" => {
                let idx = self.cursor.get();
                if let Some(item) = self.items.get(idx) {
                    // Dispatch the action to the source widget
                    if let Some(source_id) = self.source_id {
                        if let Some(widget) = ctx.arena.get(source_id) {
                            widget.on_action(&item.action, ctx);
                        }
                    }
                }
                ctx.dismiss_overlay();
            }
            "close" => {
                ctx.dismiss_overlay();
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 || self.items.is_empty() {
            return;
        }

        // Calculate menu dimensions
        let max_label_len = self
            .items
            .iter()
            .map(|item| {
                let shortcut_len = item.shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0);
                item.label.len() + shortcut_len
            })
            .max()
            .unwrap_or(10);
        let menu_width = (max_label_len + 4).min(area.width as usize) as u16; // +4 for padding
        let menu_height = (self.items.len() as u16 + 2).min(area.height); // +2 for border

        // Position: try to place at anchor, adjust if overflows
        let menu_x = self.anchor_x.min(area.x + area.width - menu_width);
        let menu_y = if self.anchor_y + menu_height > area.y + area.height {
            (area.y + area.height).saturating_sub(menu_height)
        } else {
            self.anchor_y
        };

        self.last_area.set((
            menu_x,
            menu_y + 1,
            menu_width,
            menu_height.saturating_sub(2),
        ));

        let border_color = Color::Rgb(100, 100, 120);
        let bg = Color::Rgb(30, 30, 42);
        let fg = Color::Rgb(224, 224, 224);

        // Draw McGugan box border
        crate::canvas::mcgugan_box(
            buf,
            menu_x,
            menu_y,
            menu_width,
            menu_height,
            border_color,
            bg,
            Color::Reset,
        );

        // Fill inside with bg
        for y in (menu_y + 1)..(menu_y + menu_height - 1) {
            for x in (menu_x + 1)..(menu_x + menu_width - 1) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_bg(bg);
                }
            }
        }

        // Render items
        let cursor = self.cursor.get();
        let inner_width = (menu_width - 2) as usize;
        for (i, item) in self.items.iter().enumerate() {
            let y = menu_y + 1 + i as u16;
            if y >= menu_y + menu_height - 1 {
                break;
            }

            let is_selected = i == cursor;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(0, 255, 163))
                    .bg(bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg).bg(bg)
            };

            // Label on left, shortcut on right
            let shortcut_text = item.shortcut.as_deref().unwrap_or("");
            let label_max = inner_width.saturating_sub(shortcut_text.len() + 1);
            let label: String = item.label.chars().take(label_max).collect();
            let padded = format!(" {:<width$}", label, width = inner_width - 1);
            buf.set_string(menu_x + 1, y, &padded, style);

            // Shortcut right-aligned
            if !shortcut_text.is_empty() {
                let shortcut_style = if is_selected {
                    Style::default().fg(Color::Rgb(0, 180, 120)).bg(bg)
                } else {
                    Style::default().fg(Color::Rgb(100, 100, 120)).bg(bg)
                };
                let sx = menu_x + menu_width - 1 - shortcut_text.len() as u16 - 1;
                buf.set_string(sx, y, shortcut_text, shortcut_style);
            }
        }
    }
}
