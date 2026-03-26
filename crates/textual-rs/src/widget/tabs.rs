use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by Tabs.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the active tab changes.
    pub struct TabChanged {
        pub index: usize,
        pub label: String,
    }

    impl Message for TabChanged {}
}

/// The tab bar widget — renders a row of tab labels with keyboard navigation.
///
/// Key bindings: Left → previous tab, Right → next tab.
/// Emits `messages::TabChanged` when the active tab changes.
pub struct Tabs {
    pub tab_labels: Vec<String>,
    pub active: Reactive<usize>,
    own_id: Cell<Option<WidgetId>>,
}

impl Tabs {
    pub fn new(labels: Vec<String>) -> Self {
        Self {
            tab_labels: labels,
            active: Reactive::new(0),
            own_id: Cell::new(None),
        }
    }
}

static TABS_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "prev_tab",
        description: "Previous tab",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "next_tab",
        description: "Next tab",
        show: false,
    },
];

impl Widget for Tabs {
    fn widget_type_name(&self) -> &'static str {
        "Tabs"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Tabs { height: 1; dock: top; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        TABS_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        let current = self.active.get_untracked();
        match action {
            "prev_tab" => {
                if current > 0 {
                    let new_idx = current - 1;
                    self.active.set(new_idx);
                    if let Some(id) = self.own_id.get() {
                        let label = self.tab_labels.get(new_idx).cloned().unwrap_or_default();
                        ctx.post_message(id, messages::TabChanged { index: new_idx, label });
                    }
                }
            }
            "next_tab" => {
                if current + 1 < self.tab_labels.len() {
                    let new_idx = current + 1;
                    self.active.set(new_idx);
                    if let Some(id) = self.own_id.get() {
                        let label = self.tab_labels.get(new_idx).cloned().unwrap_or_default();
                        ctx.post_message(id, messages::TabChanged { index: new_idx, label });
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        let active_idx = self.active.get_untracked();
        let separator = " | ";

        // Build the tab bar string, tracking positions for highlight
        let mut x = area.x;
        let y = area.y;

        for (i, label) in self.tab_labels.iter().enumerate() {
            if x >= area.x + area.width {
                break;
            }

            // Separator before all but first
            if i > 0 {
                for ch in separator.chars() {
                    if x >= area.x + area.width {
                        break;
                    }
                    buf[(x, y)].set_char(ch).set_style(Style::default());
                    x += 1;
                }
            }

            // Leading space
            if x < area.x + area.width {
                buf[(x, y)].set_char(' ').set_style(Style::default());
                x += 1;
            }

            // Tab label characters
            let style = if i == active_idx {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            for ch in label.chars() {
                if x >= area.x + area.width {
                    break;
                }
                buf[(x, y)].set_char(ch).set_style(style);
                x += 1;
            }

            // Trailing space
            if x < area.x + area.width {
                buf[(x, y)].set_char(' ').set_style(if i == active_idx { style } else { Style::default() });
                x += 1;
            }
        }
    }
}

/// A container that combines a Tabs bar with content panes, showing only the active pane.
///
/// Renders the tab bar in the first row and the active pane in the remaining area.
/// The tab bar is embedded and rendered directly — not composed as a child widget.
pub struct TabbedContent {
    pub tabs: Tabs,
    pub panes: Vec<Box<dyn Widget>>,
}

impl TabbedContent {
    pub fn new(labels: Vec<String>, panes: Vec<Box<dyn Widget>>) -> Self {
        Self {
            tabs: Tabs::new(labels),
            panes,
        }
    }
}

impl Widget for TabbedContent {
    fn widget_type_name(&self) -> &'static str {
        "TabbedContent"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "TabbedContent { min-height: 3; }"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Render tab bar in the first row
        let tab_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        self.tabs.render(ctx, tab_area, buf);

        // Render the active pane in the remaining area
        if area.height > 1 {
            let pane_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: area.height - 1,
            };
            let active_idx = self.tabs.active.get_untracked();
            if let Some(pane) = self.panes.get(active_idx) {
                pane.render(ctx, pane_area, buf);
            }
        }
    }
}
