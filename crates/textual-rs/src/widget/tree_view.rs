//! Hierarchical tree view widget with expand/collapse and guide characters.
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use std::cell::{Cell, RefCell};

use super::context::AppContext;
use super::{Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// A single node in the tree hierarchy.
#[derive(Clone)]
pub struct TreeNode {
    /// Display label shown in the tree.
    pub label: String,
    /// Optional application-defined data string associated with this node.
    pub data: Option<String>,
    /// Child nodes nested under this node.
    pub children: Vec<TreeNode>,
    /// Whether this node's children are currently visible.
    pub expanded: bool,
}

impl TreeNode {
    /// Create a leaf node (no children) with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            data: None,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Create a branch node with the given label and children.
    pub fn with_children(label: &str, children: Vec<TreeNode>) -> Self {
        Self {
            label: label.to_string(),
            data: None,
            children,
            expanded: false,
        }
    }
}

/// A flattened view entry used for rendering.
struct FlatEntry {
    label: String,
    depth: usize,
    is_last_sibling: bool,
    has_children: bool,
    expanded: bool,
    path: Vec<usize>,
    /// For each ancestor, whether that ancestor was the last sibling.
    /// Used for guide char rendering (vertical line vs blank).
    ancestor_is_last: Vec<bool>,
}

/// Messages emitted by a Tree.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted when a tree node is selected (Enter key).
    pub struct NodeSelected {
        /// Index path from the root to the selected node.
        pub path: Vec<usize>,
    }

    impl Message for NodeSelected {}

    /// Emitted when a tree node is expanded.
    pub struct NodeExpanded {
        /// Index path from the root to the expanded node.
        pub path: Vec<usize>,
    }

    impl Message for NodeExpanded {}

    /// Emitted when a tree node is collapsed.
    pub struct NodeCollapsed {
        /// Index path from the root to the collapsed node.
        pub path: Vec<usize>,
    }

    impl Message for NodeCollapsed {}
}

/// Hierarchical tree view widget with expand/collapse and guide characters.
///
/// Renders nodes with guide characters (`├── `, `└── `, `│   `) for visual hierarchy.
/// Space toggles expand/collapse. Enter emits NodeSelected.
pub struct Tree {
    /// Root node of the tree.
    pub root: RefCell<TreeNode>,
    /// Zero-based index of the highlighted node in the flattened visible list.
    pub cursor: Reactive<usize>,
    /// Vertical scroll offset in rows.
    pub scroll_offset: Reactive<usize>,
    flat_entries: RefCell<Vec<FlatEntry>>,
    viewport_height: Cell<u16>,
    own_id: Cell<Option<WidgetId>>,
    /// Dirty flag — true when flat_entries need rebuilding.
    dirty: Cell<bool>,
    last_area_y: Cell<u16>,
}

impl Tree {
    /// Create a new Tree with the given root node.
    pub fn new(root: TreeNode) -> Self {
        let tree = Self {
            root: RefCell::new(root),
            cursor: Reactive::new(0),
            scroll_offset: Reactive::new(0),
            flat_entries: RefCell::new(Vec::new()),
            viewport_height: Cell::new(0),
            own_id: Cell::new(None),
            dirty: Cell::new(true),
            last_area_y: Cell::new(0),
        };
        // Initial flatten
        tree.rebuild_flat_entries();
        tree
    }

    /// Walk the tree in pre-order, building the flat entry list.
    /// Collapsed nodes' subtrees are skipped.
    fn rebuild_flat_entries(&self) {
        let mut entries = Vec::new();
        let root = self.root.borrow();
        // Process the root node's children (root itself is not shown)
        flatten_children(&root.children, &[], &[], &mut entries);
        drop(root);
        *self.flat_entries.borrow_mut() = entries;
        self.dirty.set(false);
    }

    /// Return the index path of the currently-focused entry, if any.
    /// Used by wrapper widgets (e.g. DirectoryTree) to identify the cursor node.
    pub fn cursor_path(&self) -> Option<Vec<usize>> {
        let cursor = self.cursor.get_untracked();
        let flat = self.flat_entries.borrow();
        flat.get(cursor).map(|e| e.path.clone())
    }

    /// Return whether the entry at `path` is currently expanded.
    pub fn is_expanded_at(&self, path: &[usize]) -> bool {
        let flat = self.flat_entries.borrow();
        flat.iter()
            .find(|e| e.path.as_slice() == path)
            .map(|e| e.expanded)
            .unwrap_or(false)
    }

    /// Mark the flat entry cache as stale. Call this after externally modifying
    /// `root` children (e.g. from DirectoryTree after a worker delivers results).
    /// The next `render()` call will rebuild the cache automatically.
    pub fn mark_dirty(&self) {
        self.dirty.set(true);
    }

    fn adjust_scroll(&self) {
        let cursor = self.cursor.get_untracked();
        let vp = self.viewport_height.get() as usize;
        if vp == 0 {
            return;
        }
        let offset = self.scroll_offset.get_untracked();
        if cursor < offset {
            self.scroll_offset.set(cursor);
        } else if cursor >= offset + vp {
            self.scroll_offset.set(cursor + 1 - vp);
        }
    }
}

/// Recursively flatten tree nodes into FlatEntry list.
/// `path` is the index path to reach the current level's parent.
/// `ancestor_is_last` tracks for each depth level whether the ancestor was the last sibling.
fn flatten_children(
    children: &[TreeNode],
    path: &[usize],
    ancestor_is_last: &[bool],
    entries: &mut Vec<FlatEntry>,
) {
    let n = children.len();
    for (i, node) in children.iter().enumerate() {
        let is_last = i + 1 == n;
        let mut node_path = path.to_vec();
        node_path.push(i);

        entries.push(FlatEntry {
            label: node.label.clone(),
            depth: path.len(),
            is_last_sibling: is_last,
            has_children: !node.children.is_empty(),
            expanded: node.expanded,
            path: node_path.clone(),
            ancestor_is_last: ancestor_is_last.to_vec(),
        });

        if node.expanded && !node.children.is_empty() {
            let mut child_ancestor_is_last = ancestor_is_last.to_vec();
            child_ancestor_is_last.push(is_last);
            flatten_children(&node.children, &node_path, &child_ancestor_is_last, entries);
        }
    }
}

/// Returns the static key bindings slice for Tree / DirectoryTree.
pub fn tree_key_bindings() -> &'static [KeyBinding] {
    TREE_BINDINGS
}

static TREE_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        action: "cursor_up",
        description: "Up",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        action: "cursor_down",
        description: "Down",
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
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        action: "toggle",
        description: "Expand/Collapse",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        action: "expand",
        description: "Expand",
        show: false,
    },
    KeyBinding {
        key: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        action: "collapse",
        description: "Collapse",
        show: false,
    },
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
];

impl Widget for Tree {
    fn widget_type_name(&self) -> &'static str {
        "Tree"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Tree { border: inner; min-height: 5; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        TREE_BINDINGS
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> super::EventPropagation {
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        if let Some(m) = event.downcast_ref::<MouseEvent>() {
            if matches!(m.kind, MouseEventKind::Down(MouseButton::Left)) {
                let local_row = m.row.saturating_sub(self.last_area_y.get()) as usize;
                let scroll = self.scroll_offset.get_untracked();
                let entry_idx = scroll + local_row;
                let flat = self.flat_entries.borrow();
                if entry_idx < flat.len() {
                    drop(flat);
                    self.cursor.set(entry_idx);
                    // Toggle expand/collapse on click
                    self.on_action("toggle", ctx);
                    return super::EventPropagation::Stop;
                }
            }
        }
        super::EventPropagation::Continue
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "cursor_up" => {
                let current = self.cursor.get_untracked();
                if current > 0 {
                    self.cursor.set(current - 1);
                    self.adjust_scroll();
                }
            }
            "cursor_down" => {
                let current = self.cursor.get_untracked();
                let flat = self.flat_entries.borrow();
                if current + 1 < flat.len() {
                    drop(flat);
                    self.cursor.set(current + 1);
                    self.adjust_scroll();
                }
            }
            "toggle" => {
                let cursor = self.cursor.get_untracked();
                let (path, has_children, currently_expanded) = {
                    let flat = self.flat_entries.borrow();
                    if let Some(entry) = flat.get(cursor) {
                        (entry.path.clone(), entry.has_children, entry.expanded)
                    } else {
                        return;
                    }
                };
                if !has_children {
                    return;
                }
                let mut root = self.root.borrow_mut();
                // Path into root's children: path[0] is index into root.children
                if let Some(node) = Self::node_at_path_in_children(&mut root.children, &path) {
                    node.expanded = !currently_expanded;
                    let new_expanded = node.expanded;
                    drop(root);
                    self.rebuild_flat_entries();
                    if let Some(id) = self.own_id.get() {
                        if new_expanded {
                            ctx.post_message(id, messages::NodeExpanded { path });
                        } else {
                            ctx.post_message(id, messages::NodeCollapsed { path });
                        }
                    }
                }
            }
            "expand" => {
                let cursor = self.cursor.get_untracked();
                let (path, has_children, currently_expanded) = {
                    let flat = self.flat_entries.borrow();
                    if let Some(entry) = flat.get(cursor) {
                        (entry.path.clone(), entry.has_children, entry.expanded)
                    } else {
                        return;
                    }
                };
                if has_children && !currently_expanded {
                    // Expand
                    let mut root = self.root.borrow_mut();
                    if let Some(node) = Self::node_at_path_in_children(&mut root.children, &path) {
                        node.expanded = true;
                        drop(root);
                        self.rebuild_flat_entries();
                        if let Some(id) = self.own_id.get() {
                            ctx.post_message(id, messages::NodeExpanded { path });
                        }
                    }
                } else if currently_expanded {
                    // Move to first child (next entry in flat list)
                    let flat = self.flat_entries.borrow();
                    if cursor + 1 < flat.len() {
                        drop(flat);
                        self.cursor.set(cursor + 1);
                        self.adjust_scroll();
                    }
                }
            }
            "collapse" => {
                let cursor = self.cursor.get_untracked();
                let (path, has_children, currently_expanded, depth) = {
                    let flat = self.flat_entries.borrow();
                    if let Some(entry) = flat.get(cursor) {
                        (
                            entry.path.clone(),
                            entry.has_children,
                            entry.expanded,
                            entry.depth,
                        )
                    } else {
                        return;
                    }
                };
                if has_children && currently_expanded {
                    // Collapse
                    let mut root = self.root.borrow_mut();
                    if let Some(node) = Self::node_at_path_in_children(&mut root.children, &path) {
                        node.expanded = false;
                        drop(root);
                        self.rebuild_flat_entries();
                        if let Some(id) = self.own_id.get() {
                            ctx.post_message(id, messages::NodeCollapsed { path });
                        }
                    }
                } else if depth > 0 {
                    // Move to parent
                    let flat = self.flat_entries.borrow();
                    // Find the parent by path (depth - 1)
                    let parent_path = &path[..path.len() - 1];
                    if let Some(parent_idx) =
                        flat.iter().position(|e| e.path.as_slice() == parent_path)
                    {
                        drop(flat);
                        self.cursor.set(parent_idx);
                        self.adjust_scroll();
                    }
                }
            }
            "select" => {
                let cursor = self.cursor.get_untracked();
                let path = {
                    let flat = self.flat_entries.borrow();
                    flat.get(cursor).map(|e| e.path.clone())
                };
                if let Some(path) = path {
                    if let Some(id) = self.own_id.get() {
                        ctx.post_message(id, messages::NodeSelected { path });
                    }
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
                let viewport = self.viewport_height.get() as usize;
                let total = self.flat_entries.borrow().len();
                if viewport > 0 && total > viewport && offset < total - viewport {
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

        let base_style = self
            .own_id
            .get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        if self.dirty.get() {
            self.rebuild_flat_entries();
        }

        self.viewport_height.set(area.height);

        let cursor = self.cursor.get_untracked();
        let scroll = self.scroll_offset.get_untracked();

        let flat = self.flat_entries.borrow();
        let total = flat.len();
        let visible = area.height as usize;

        // Scrollbar needed?
        let has_scrollbar = total > visible;
        let content_width = if has_scrollbar {
            area.width.saturating_sub(1)
        } else {
            area.width
        } as usize;

        for row_offset in 0..visible {
            let entry_idx = scroll + row_offset;
            let row_y = area.y + row_offset as u16;

            if entry_idx >= total {
                break;
            }

            let entry = &flat[entry_idx];
            let is_cursor = entry_idx == cursor;

            // Build guide string
            // For each ancestor depth, draw "│   " if that ancestor was NOT the last sibling,
            // or "    " if it was.
            let mut guide = String::new();
            for &anc_is_last in &entry.ancestor_is_last {
                if anc_is_last {
                    guide.push_str("    ");
                } else {
                    guide.push_str("│   ");
                }
            }
            // Connector for current node
            if entry.is_last_sibling {
                guide.push_str("└── ");
            } else {
                guide.push_str("├── ");
            }
            // Expand indicator
            if entry.has_children {
                if entry.expanded {
                    guide.push_str("▼ ");
                } else {
                    guide.push_str("▶ ");
                }
            }
            guide.push_str(&entry.label);

            // Split guide chars from label for separate styling
            let guide_len = guide.len() - entry.label.len();
            let guide_part: String = guide.chars().take(guide_len.min(content_width)).collect();
            let label_part: String = entry
                .label
                .chars()
                .take(content_width.saturating_sub(guide_part.chars().count()))
                .collect();

            let guide_style = if is_cursor {
                base_style.fg(ratatui::style::Color::Rgb(0, 255, 163))
            } else {
                base_style.fg(ratatui::style::Color::Rgb(74, 74, 90))
            };
            let label_style = if is_cursor {
                base_style
                    .fg(ratatui::style::Color::Rgb(0, 255, 163))
                    .add_modifier(Modifier::BOLD)
            } else {
                base_style
            };

            buf.set_string(area.x, row_y, &guide_part, guide_style);
            let lx = area.x + guide_part.chars().count() as u16;
            buf.set_string(lx, row_y, &label_part, label_style);
            let rendered = guide_part.chars().count() + label_part.chars().count();
            if rendered < content_width {
                let padding = " ".repeat(content_width - rendered);
                let pad_style = if is_cursor { label_style } else { base_style };
                buf.set_string(area.x + rendered as u16, row_y, &padding, pad_style);
            }
        }

        // Vertical scrollbar using canvas eighth-block rendering
        if has_scrollbar {
            let sb_x = area.x + area.width - 1;
            let bar_color = ratatui::style::Color::Rgb(0, 255, 163);
            let track_color = ratatui::style::Color::Rgb(30, 30, 40);
            crate::canvas::vertical_scrollbar(
                buf,
                sb_x,
                area.y,
                area.height,
                total,
                visible,
                scroll,
                bar_color,
                track_color,
            );
        }
    }
}

impl Tree {
    /// Helper to navigate into root.children by path (path[0] indexes root.children,
    /// path[1] indexes that node's children, etc.).
    fn node_at_path_in_children<'a>(
        children: &'a mut [TreeNode],
        path: &[usize],
    ) -> Option<&'a mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let idx = path[0];
        if idx >= children.len() {
            return None;
        }
        if path.len() == 1 {
            return Some(&mut children[idx]);
        }
        Self::node_at_path_in_children(&mut children[idx].children, &path[1..])
    }
}
