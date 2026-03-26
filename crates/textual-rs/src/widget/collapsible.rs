use std::cell::Cell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by a Collapsible.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the collapsible is expanded.
    pub struct Expanded;
    impl Message for Expanded {}

    /// Emitted when the collapsible is collapsed.
    pub struct Collapsed;
    impl Message for Collapsed {}
}

/// A container that can expand/collapse its children on Enter key.
///
/// The title row is always rendered. Children are rendered below it only when expanded.
/// Uses render-time visibility (Pitfall 6) — does NOT use compose() for dynamic children.
pub struct Collapsible {
    pub title: String,
    pub expanded: Reactive<bool>,
    pub children: Vec<Box<dyn Widget>>,
    own_id: Cell<Option<WidgetId>>,
}

impl Collapsible {
    pub fn new(title: &str, children: Vec<Box<dyn Widget>>) -> Self {
        Self {
            title: title.to_string(),
            expanded: Reactive::new(true),
            children,
            own_id: Cell::new(None),
        }
    }
}

static COLLAPSIBLE_BINDINGS: &[KeyBinding] = &[KeyBinding {
    key: KeyCode::Enter,
    modifiers: KeyModifiers::NONE,
    action: "toggle",
    description: "Toggle",
    show: true,
}];

impl Widget for Collapsible {
    fn widget_type_name(&self) -> &'static str {
        "Collapsible"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Collapsible { min-height: 1; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        COLLAPSIBLE_BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "toggle" {
            let was_expanded = self.expanded.get_untracked();
            self.expanded.set(!was_expanded);
            if let Some(id) = self.own_id.get() {
                if was_expanded {
                    ctx.post_message(id, messages::Collapsed);
                } else {
                    ctx.post_message(id, messages::Expanded);
                }
            }
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Use get_untracked() to avoid reactive tracking loops in render
        let expanded = self.expanded.get_untracked();

        // Always render the title row
        let arrow = if expanded { "▼" } else { "▶" };
        let title_text = format!("{} {}", arrow, self.title);
        let display: String = title_text.chars().take(area.width as usize).collect();
        buf.set_string(area.x, area.y, &display, Style::default());

        // Render children below title row only if expanded
        if expanded && area.height > 1 {
            let children_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: area.height - 1,
            };

            let mut child_y = children_area.y;
            for child in &self.children {
                if child_y >= children_area.y + children_area.height {
                    break;
                }
                let child_area = Rect {
                    x: children_area.x,
                    y: child_y,
                    width: children_area.width,
                    height: 1,
                };
                child.render(ctx, child_area, buf);
                child_y += 1;
            }
        }
    }
}
