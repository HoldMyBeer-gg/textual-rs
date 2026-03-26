use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a Select.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when a new option is chosen from the overlay.
    pub struct Changed {
        pub value: String,
        pub index: usize,
    }

    impl Message for Changed {}
}

/// A dropdown selection widget that displays the current selection and opens an
/// overlay on Enter to let the user pick from a list of options.
pub struct Select {
    pub options: Vec<String>,
    pub selected: Reactive<usize>,
    own_id: Cell<Option<WidgetId>>,
}

impl Select {
    /// Create a new Select with the given options. Initial selection is index 0.
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: Reactive::new(0),
            own_id: Cell::new(None),
        }
    }
}

static SELECT_BINDINGS: &[KeyBinding] = &[KeyBinding {
    key: KeyCode::Enter,
    modifiers: KeyModifiers::NONE,
    action: "open",
    description: "Open",
    show: true,
}];

impl Widget for Select {
    fn widget_type_name(&self) -> &'static str {
        "Select"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Select { border: inner; height: 3; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        SELECT_BINDINGS
    }

    fn click_action(&self) -> Option<&str> {
        Some("open")
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "open" {
            let current = self.selected.get_untracked();
            let overlay = SelectOverlay {
                options: self.options.clone(),
                current,
                cursor: Cell::new(current),
                source_id: self.own_id.get(),
                last_area_y: Cell::new(0),
            };
            ctx.push_screen_deferred(Box::new(overlay));
        }
    }

    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> super::EventPropagation {
        if let Some(changed) = event.downcast_ref::<messages::Changed>() {
            self.selected.set(changed.index);
            return super::EventPropagation::Stop;
        }
        super::EventPropagation::Continue
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let style = self.own_id.get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        let selected = self.selected.get_untracked();
        let label = self.options.get(selected).map(|s| s.as_str()).unwrap_or("");
        let text = format!("\u{25bc} {}", label); // "▼ label"
        let display: String = text.chars().take(area.width as usize).collect();
        buf.set_string(area.x, area.y, &display, style);
    }
}

// ---------------------------------------------------------------------------
// SelectOverlay — private screen shown when Select is opened
// ---------------------------------------------------------------------------

struct SelectOverlay {
    options: Vec<String>,
    /// The index that was selected when the overlay opened (kept for future selection highlighting).
    #[allow(dead_code)]
    current: usize,
    /// Cursor position within the overlay list.
    cursor: Cell<usize>,
    /// The WidgetId of the originating Select widget (to post Changed back).
    source_id: Option<WidgetId>,
    /// Last rendered area origin for mouse hit-testing.
    last_area_y: Cell<u16>,
}

static OVERLAY_BINDINGS: &[KeyBinding] = &[
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
        key: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        action: "cancel",
        description: "Cancel",
        show: true,
    },
];

impl Widget for SelectOverlay {
    fn widget_type_name(&self) -> &'static str {
        "SelectOverlay"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn is_overlay(&self) -> bool {
        true
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        OVERLAY_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> super::EventPropagation {
        use crossterm::event::{MouseEvent, MouseEventKind};
        if let Some(m) = event.downcast_ref::<MouseEvent>() {
            if matches!(m.kind, MouseEventKind::Down(_)) {
                let content_y = self.last_area_y.get();
                let local_row = m.row.saturating_sub(content_y) as usize;
                if m.row >= content_y && local_row < self.options.len() {
                    self.cursor.set(local_row);
                    self.on_action("select", ctx);
                    return super::EventPropagation::Stop;
                }
                // Click outside dropdown — close it
                ctx.pop_screen_deferred();
                return super::EventPropagation::Stop;
            }
        }
        super::EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "cursor_up" => {
                let cursor = self.cursor.get();
                if cursor > 0 {
                    self.cursor.set(cursor - 1);
                } else {
                    // Wrap to bottom
                    self.cursor.set(self.options.len().saturating_sub(1));
                }
            }
            "cursor_down" => {
                let cursor = self.cursor.get();
                if cursor + 1 < self.options.len() {
                    self.cursor.set(cursor + 1);
                } else {
                    // Wrap to top
                    self.cursor.set(0);
                }
            }
            "select" => {
                let idx = self.cursor.get();
                // Post Changed to the originating Select widget
                if let Some(source_id) = self.source_id {
                    if let Some(value) = self.options.get(idx) {
                        // Update the Select widget's reactive selected field
                        if let Some(widget) = ctx.arena.get(source_id) {
                            // Downcast to Select to update selected index
                            if widget.widget_type_name() == "Select" {
                                // We post the Changed message; the Select widget's
                                // on_event can handle updating its own state.
                                ctx.post_message(
                                    source_id,
                                    messages::Changed {
                                        value: value.clone(),
                                        index: idx,
                                    },
                                );
                            }
                        }
                    }
                }
                ctx.pop_screen_deferred();
            }
            "cancel" => {
                ctx.pop_screen_deferred();
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        use ratatui::style::{Color, Style};

        if area.height == 0 || area.width == 0 || self.options.is_empty() {
            return;
        }

        // Calculate dropdown dimensions
        let max_label = self.options.iter().map(|o| o.chars().count()).max().unwrap_or(10);
        let dropdown_width = (max_label + 4).min(area.width as usize) as u16;
        let dropdown_height = (self.options.len() as u16 + 2).min(area.height); // +2 for border

        // Position: center horizontally, top of screen area
        let dx = if area.width > dropdown_width {
            area.x + (area.width - dropdown_width) / 2
        } else {
            area.x
        };
        let dy = if area.height > dropdown_height {
            area.y + (area.height - dropdown_height) / 2
        } else {
            area.y
        };

        self.last_area_y.set(dy + 1); // content starts after top border

        let border_color = Color::Rgb(100, 100, 120);
        let bg = Color::Rgb(30, 30, 42);
        let fg = Color::Rgb(224, 224, 224);

        // Draw McGugan box border
        crate::canvas::mcgugan_box(buf, dx, dy, dropdown_width, dropdown_height, border_color, bg, Color::Reset);

        // Fill inside
        for y in (dy + 1)..(dy + dropdown_height - 1) {
            for x in (dx + 1)..(dx + dropdown_width - 1) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_bg(bg);
                }
            }
        }

        // Render options
        let cursor = self.cursor.get();
        let inner_width = (dropdown_width - 2) as usize;
        for (i, option) in self.options.iter().enumerate() {
            let y = dy + 1 + i as u16;
            if y >= dy + dropdown_height - 1 {
                break;
            }
            let display: String = option.chars().take(inner_width).collect();
            let padded = format!(" {:<width$}", display, width = inner_width - 1);
            if i == cursor {
                let hl_style = Style::default()
                    .fg(Color::Rgb(0, 255, 163))
                    .bg(bg)
                    .add_modifier(Modifier::BOLD);
                buf.set_string(dx + 1, y, &padded, hl_style);
            } else {
                let style = Style::default().fg(fg).bg(bg);
                buf.set_string(dx + 1, y, &padded, style);
            }
        }
    }
}
