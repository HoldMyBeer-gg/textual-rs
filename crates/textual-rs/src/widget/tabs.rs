use std::cell::{Cell, RefCell};
use std::time::Duration;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use crossterm::event::{KeyCode, KeyModifiers};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::animation::{Tween, ease_out_cubic};
use crate::css::render_style::fill_background;
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
    /// Tween for the underline x-position when switching tabs.
    underline_tween: RefCell<Option<Tween>>,
}

impl Tabs {
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
            ctx.post_message(id, messages::TabChanged { index: new_idx, label });
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = self.own_id.get()
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
    fn widget_type_name(&self) -> &'static str { "TabBar" }
    fn can_focus(&self) -> bool { true }
    fn default_css() -> &'static str where Self: Sized { "TabBar { height: 1; }" }

    fn on_mount(&self, id: WidgetId) { self.own_id.set(Some(id)); }
    fn on_unmount(&self, _id: WidgetId) { self.own_id.set(None); }
    fn key_bindings(&self) -> &[KeyBinding] { TAB_BAR_BINDINGS }

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
        if area.height == 0 || area.width == 0 { return; }
        self.last_area_x.set(area.x);
        let base_style = ratatui::style::Style::default();
        let active_idx = self.active.get();
        let separator = " | ";
        let mut x = area.x;
        let y = area.y;
        for (i, label) in self.labels.iter().enumerate() {
            if x >= area.x + area.width { break; }
            if i > 0 {
                for ch in separator.chars() {
                    if x >= area.x + area.width { break; }
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(ch).set_style(base_style);
                    }
                    x += 1;
                }
            }
            if x < area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) { cell.set_char(' ').set_style(base_style); }
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
                if x >= area.x + area.width { break; }
                if let Some(cell) = buf.cell_mut((x, y)) { cell.set_char(ch).set_style(style); }
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
    pub labels: Vec<String>,
    pub panes: Vec<Box<dyn Widget>>,
    /// Shared active tab index — both TabbedContent and TabBar read/write this.
    pub active: std::rc::Rc<Cell<usize>>,
}

impl TabbedContent {
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

/// Recursively render a pane widget and its compose() children.
/// Since panes are not in the arena, we manually lay out children vertically/horizontally
/// and apply the parent background to give them styled rendering.
fn render_pane_tree(widget: &dyn Widget, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Fill pane background from parent (inherit from buffer)
    let parent_style = buf.cell((area.x, area.y)).map(|c| c.style()).unwrap_or_default();
    if let Some(bg) = parent_style.bg {
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(bg);
                }
            }
        }
    }

    let children = widget.compose();
    if children.is_empty() {
        // Leaf pane — just render it directly
        widget.render(ctx, area, buf);
        return;
    }

    // Detect layout direction from widget type name
    // Convention: panes with "horizontal" in CSS or type names with known patterns
    let is_horizontal = widget.widget_type_name().contains("Lists")
        || widget.widget_type_name().contains("Main")
        || widget.widget_type_name().contains("Horizontal");

    if is_horizontal {
        // Split area equally among children
        let child_count = children.len() as u16;
        if child_count == 0 { return; }
        let child_width = area.width / child_count;
        let mut x = area.x;

        for (i, child) in children.iter().enumerate() {
            let w = if i as u16 == child_count - 1 {
                area.width - (x - area.x) // last child gets remaining width
            } else {
                child_width
            };
            let child_area = Rect { x, y: area.y, width: w, height: area.height };
            render_child_styled(child.as_ref(), ctx, child_area, buf);
            x += w;
        }
    } else {
        // Vertical layout — give each child its natural height, flex remaining
        let child_count = children.len();
        let mut heights: Vec<u16> = Vec::with_capacity(child_count);
        let mut flex_children: Vec<usize> = Vec::new();
        let mut used_height: u16 = 0;

        // First pass: assign fixed heights
        for (i, child) in children.iter().enumerate() {
            let name = child.widget_type_name();
            // Widgets that want flex-grow
            let wants_flex = matches!(name,
                "DataTable" | "ListView" | "Log" | "TreeView" | "ScrollView" | "TextArea"
            );
            if wants_flex {
                heights.push(0); // will be assigned later
                flex_children.push(i);
            } else {
                // Give fixed-height widgets their natural height
                let h = match name {
                    "Button" => 3u16,
                    "Input" | "Select" => 3,
                    "Label" | "Checkbox" | "Switch" | "ProgressBar" | "Sparkline" | "RadioButton" => 1,
                    "Collapsible" => 2,
                    _ => {
                        // For containers, count composed children
                        let child_count = child.compose().len() as u16;
                        if child_count > 0 { child_count } else { 1 }
                    }
                };
                heights.push(h);
                used_height += h;
            }
        }

        // Second pass: distribute remaining height to flex children
        let remaining = area.height.saturating_sub(used_height);
        if !flex_children.is_empty() {
            let per_flex = remaining / flex_children.len() as u16;
            let extra = remaining % flex_children.len() as u16;
            for (j, &idx) in flex_children.iter().enumerate() {
                heights[idx] = per_flex + if (j as u16) < extra { 1 } else { 0 };
            }
        }

        // Third pass: render children
        let mut y = area.y;
        for (i, child) in children.iter().enumerate() {
            let h = heights[i];
            if h == 0 || y >= area.y + area.height {
                continue;
            }
            let h = h.min(area.y + area.height - y);
            let child_area = Rect { x: area.x, y, width: area.width, height: h };
            render_child_styled(child.as_ref(), ctx, child_area, buf);
            y += h;
        }
    }
}

/// Render a single child widget with CSS-like chrome (background, border).
/// Since pane children aren't in the arena, we build a synthetic ComputedStyle
/// by first checking the arena for a matching type, then falling back to parsing
/// the widget's default_css() and any user stylesheet rules.
fn render_child_styled(widget: &dyn Widget, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
    use crate::css::render_style::draw_border;
    use crate::css::types::ComputedStyle;

    if area.width == 0 || area.height == 0 {
        return;
    }

    let type_name = widget.widget_type_name();

    // Try arena first (works if another widget of same type exists in the tree)
    let mut matched_style: Option<ComputedStyle> = None;
    for (id, w) in ctx.arena.iter() {
        if w.widget_type_name() == type_name {
            if let Some(cs) = ctx.computed_styles.get(id) {
                matched_style = Some(cs.clone());
                break;
            }
        }
    }

    // Fallback: build style from default_css() + user stylesheets
    if matched_style.is_none() {
        let mut cs = ComputedStyle::default();

        // Apply widget's default CSS
        let default_css = get_default_css_for_type(type_name);
        if !default_css.is_empty() {
            let (stylesheet, _) = crate::css::cascade::Stylesheet::parse(default_css);
            for rule in &stylesheet.rules {
                cs.apply_declarations(&rule.declarations);
            }
        }

        // Apply user stylesheets (higher priority — overrides defaults)
        for stylesheet in &ctx.stylesheets {
            for rule in &stylesheet.rules {
                for sel in &rule.selectors {
                    // Simple type selector match
                    if matches!(sel, crate::css::selector::Selector::Type(name) if name == type_name) {
                        cs.apply_declarations(&rule.declarations);
                    }
                }
            }
        }

        // Inherit fg/bg from parent buffer if not set by CSS
        if cs.color == crate::css::types::TcssColor::Reset {
            if let Some(cell) = buf.cell((area.x, area.y)) {
                if let Some(fg) = cell.style().fg {
                    cs.color = match fg {
                        ratatui::style::Color::Rgb(r, g, b) => crate::css::types::TcssColor::Rgb(r, g, b),
                        _ => crate::css::types::TcssColor::Reset,
                    };
                }
            }
        }
        if cs.background == crate::css::types::TcssColor::Reset {
            if let Some(cell) = buf.cell((area.x, area.y)) {
                if let Some(bg) = cell.style().bg {
                    cs.background = match bg {
                        ratatui::style::Color::Rgb(r, g, b) => crate::css::types::TcssColor::Rgb(r, g, b),
                        _ => crate::css::types::TcssColor::Reset,
                    };
                }
            }
        }

        matched_style = Some(cs);
    }

    let content_area = if let Some(ref cs) = matched_style {
        fill_background(cs, area, buf);
        draw_border(cs, area, buf)
    } else {
        area
    };

    // Check if this widget has compose() children
    let children = widget.compose();
    if children.is_empty() {
        // Leaf widget — render with inherited style from buffer
        widget.render(ctx, content_area, buf);
    } else {
        // Container — recurse
        render_pane_tree(widget, ctx, content_area, buf);
    }
}

/// Get default CSS string for known widget types.
/// This is used for ad-hoc pane children that aren't in the arena.
fn get_default_css_for_type(type_name: &str) -> &'static str {
    match type_name {
        "Button" => "Button { border: heavy; min-width: 16; height: 3; text-align: center; }",
        "Input" => "Input { border: rounded; height: 3; }",
        "Label" => "Label { min-height: 1; }",
        "Checkbox" => "Checkbox { height: 1; }",
        "Switch" => "Switch { height: 1; width: 8; }",
        "RadioSet" => "",
        "DataTable" => "DataTable { border: rounded; min-height: 3; }",
        "ListView" => "ListView { min-height: 3; flex-grow: 1; }",
        "Log" => "Log { min-height: 3; flex-grow: 1; }",
        "ProgressBar" => "ProgressBar { height: 1; }",
        "Sparkline" => "Sparkline { height: 1; }",
        "Placeholder" => "Placeholder { border: rounded; min-height: 3; min-width: 10; }",
        "Collapsible" => "",
        _ => "",
    }
}
