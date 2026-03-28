//! Tab bar widget and TabbedContent container for multi-pane navigation.
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use std::cell::{Cell, RefCell};
use std::time::Duration;

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::animation::{ease_out_cubic, Tween};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by Tabs.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when the active tab changes.
    pub struct TabChanged {
        /// Zero-based index of the newly active tab.
        pub index: usize,
        /// Label of the newly active tab.
        pub label: String,
    }

    impl Message for TabChanged {}
}

/// The tab bar widget — renders a row of tab labels with keyboard navigation.
///
/// Key bindings: Left → previous tab, Right → next tab.
/// Emits `messages::TabChanged` when the active tab changes.
pub struct Tabs {
    /// Labels for each tab rendered in the bar.
    pub tab_labels: Vec<String>,
    /// Zero-based index of the currently active tab.
    pub active: Reactive<usize>,
    own_id: Cell<Option<WidgetId>>,
    /// Tween for the underline x-position when switching tabs.
    underline_tween: RefCell<Option<Tween>>,
}

impl Tabs {
    /// Create a new Tabs bar with the given tab labels, initially selecting index 0.
    pub fn new(labels: Vec<String>) -> Self {
        Self {
            tab_labels: labels,
            active: Reactive::new(0),
            own_id: Cell::new(None),
            underline_tween: RefCell::new(None),
        }
    }

    /// Compute the x-offset of a tab label within the bar.
    fn tab_x_offset(&self, tab_idx: usize) -> f64 {
        let separator = " | ";
        let mut x: f64 = 0.0;
        for (i, label) in self.tab_labels.iter().enumerate() {
            if i == tab_idx {
                return x;
            }
            if i > 0 {
                x += separator.len() as f64;
            }
            x += 1.0; // leading space
            x += label.chars().count() as f64;
            x += 1.0; // trailing space
        }
        x
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
        let new_idx = match action {
            "prev_tab" if current > 0 => current - 1,
            "next_tab" if current + 1 < self.tab_labels.len() => current + 1,
            _ => return,
        };

        // Start underline animation from old tab position to new tab position
        let from_x = self.tab_x_offset(current);
        let to_x = self.tab_x_offset(new_idx);
        *self.underline_tween.borrow_mut() = Some(Tween::new(
            from_x,
            to_x,
            Duration::from_millis(200),
            ease_out_cubic,
        ));

        self.active.set(new_idx);
        if let Some(id) = self.own_id.get() {
            let label = self.tab_labels.get(new_idx).cloned().unwrap_or_default();
            ctx.post_message(
                id,
                messages::TabChanged {
                    index: new_idx,
                    label,
                },
            );
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = self
            .own_id
            .get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

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
                    buf[(x, y)].set_char(ch).set_style(base_style);
                    x += 1;
                }
            }

            // Leading space
            if x < area.x + area.width {
                buf[(x, y)].set_char(' ').set_style(base_style);
                x += 1;
            }

            // Tab label characters — active tab gets accent color + underline
            let style = if i == active_idx {
                base_style
                    .fg(ratatui::style::Color::Rgb(0, 255, 163))
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED)
            } else {
                base_style.fg(ratatui::style::Color::Rgb(140, 140, 160))
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
                buf[(x, y)].set_char(' ').set_style(base_style);
                x += 1;
            }
        }
    }
}

/// Tab bar widget — child of TabbedContent. Focusable with Left/Right to switch tabs.
/// Shares `active` index with TabbedContent via Rc<Cell<usize>>.
struct TabBar {
    labels: Vec<String>,
    active: std::rc::Rc<Cell<usize>>,
    own_id: Cell<Option<WidgetId>>,
    /// Last rendered area — used to translate absolute mouse coords to tab positions.
    last_area_x: Cell<u16>,
}

static TAB_BAR_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "prev_tab",
        description: "Prev tab",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "next_tab",
        description: "Next tab",
        show: true,
    },
];

impl TabBar {
    fn switch_to(&self, idx: usize, ctx: &AppContext) {
        if idx >= self.labels.len() || idx == self.active.get() {
            return;
        }
        self.active.set(idx);
        if let Some(id) = self.own_id.get() {
            if let Some(&Some(parent_id)) = ctx.parent.get(id) {
                ctx.request_recompose(parent_id);
            }
        }
    }
}

impl Widget for TabBar {
    fn widget_type_name(&self) -> &'static str {
        "TabBar"
    }
    fn can_focus(&self) -> bool {
        true
    }
    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "TabBar { height: 1; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }
    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }
    fn key_bindings(&self) -> &[KeyBinding] {
        TAB_BAR_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> super::EventPropagation {
        use crossterm::event::{MouseEvent, MouseEventKind};
        if let Some(m) = event.downcast_ref::<MouseEvent>() {
            if matches!(m.kind, MouseEventKind::Down(_)) {
                // Translate absolute screen x to widget-local x
                let area_x = self.last_area_x.get();
                let local_col = m.column.saturating_sub(area_x);
                // Compute which tab label was clicked
                let separator = " | ";
                let mut x: u16 = 0;
                for (i, label) in self.labels.iter().enumerate() {
                    if i > 0 {
                        x += separator.len() as u16;
                    }
                    let start = x;
                    x += 1; // leading space
                    x += label.chars().count() as u16;
                    x += 1; // trailing space
                    if local_col >= start && local_col < x {
                        self.switch_to(i, ctx);
                        return super::EventPropagation::Stop;
                    }
                }
            }
        }
        super::EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        let current = self.active.get();
        let count = self.labels.len();
        let new_idx = match action {
            "prev_tab" if current > 0 => current - 1,
            "next_tab" if current + 1 < count => current + 1,
            _ => return,
        };
        self.switch_to(new_idx, ctx);
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        self.last_area_x.set(area.x);
        let base_style = ratatui::style::Style::default();
        let active_idx = self.active.get();
        let separator = " | ";
        let mut x = area.x;
        let y = area.y;
        for (i, label) in self.labels.iter().enumerate() {
            if x >= area.x + area.width {
                break;
            }
            if i > 0 {
                for ch in separator.chars() {
                    if x >= area.x + area.width {
                        break;
                    }
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(ch).set_style(base_style);
                    }
                    x += 1;
                }
            }
            if x < area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ').set_style(base_style);
                }
                x += 1;
            }
            let style = if i == active_idx {
                base_style
                    .fg(ratatui::style::Color::Rgb(0, 255, 163))
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED)
            } else {
                base_style.fg(ratatui::style::Color::Rgb(140, 140, 160))
            };
            for ch in label.chars() {
                if x >= area.x + area.width {
                    break;
                }
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(ch).set_style(style);
                }
                x += 1;
            }
            if x < area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ').set_style(base_style);
                }
                x += 1;
            }
        }
    }
}

/// A container that combines a Tabs bar with content panes, showing only the active pane.
///
/// Composes the active pane's children into the widget tree so they participate
/// in focus cycling and event dispatch. Tab switching triggers recomposition.
pub struct TabbedContent {
    /// Tab label strings corresponding to each pane.
    pub labels: Vec<String>,
    /// Content pane widgets, one per label.
    pub panes: Vec<Box<dyn Widget>>,
    /// Shared active tab index — both TabbedContent and TabBar read/write this.
    pub active: std::rc::Rc<Cell<usize>>,
}

impl TabbedContent {
    /// Create a new TabbedContent with the given labels and panes, initially showing pane 0.
    pub fn new(labels: Vec<String>, panes: Vec<Box<dyn Widget>>) -> Self {
        let active = std::rc::Rc::new(Cell::new(0));
        Self {
            labels,
            panes,
            active,
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
        "TabbedContent { min-height: 3; layout-direction: vertical; }"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let mut children: Vec<Box<dyn Widget>> = Vec::new();

        // Tab bar — shares active index via Rc<Cell>
        children.push(Box::new(TabBar {
            labels: self.labels.clone(),
            active: self.active.clone(),
            own_id: Cell::new(None),
            last_area_x: Cell::new(0),
        }));

        // Active pane's children
        let active_idx = self.active.get();
        if let Some(pane) = self.panes.get(active_idx) {
            children.extend(pane.compose());
        }

        children
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Children (TabBar + pane widgets) are rendered by the framework.
    }
}
