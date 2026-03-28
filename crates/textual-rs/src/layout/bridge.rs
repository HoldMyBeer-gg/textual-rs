//! Taffy layout bridge: synchronizes the widget tree into Taffy and maps results back to screen Rects.

use super::style_map::taffy_style_from_computed;
use crate::widget::context::AppContext;
use crate::widget::WidgetId;
use ratatui::layout::Rect;
use std::collections::HashMap;
use taffy::geometry::Size;
use taffy::style::AvailableSpace;
use taffy::{NodeId, TaffyTree};

/// TaffyBridge synchronizes the widget tree into Taffy's node tree,
/// computes layout, and exposes the resulting ratatui `Rect` for each widget.
///
/// Two sync modes:
/// - `sync_subtree`: full sync, ignores dirty flags
/// - `sync_dirty_subtree`: only re-syncs widgets where `ctx.dirty[id] == true`
pub struct TaffyBridge {
    /// The underlying Taffy layout tree
    tree: TaffyTree<()>,
    /// Maps WidgetId → Taffy NodeId
    node_map: HashMap<WidgetId, NodeId>,
    /// Maps WidgetId → computed screen Rect (output of last `compute_layout`)
    layout_cache: HashMap<WidgetId, Rect>,
}

impl TaffyBridge {
    /// Create a new empty TaffyBridge with no nodes in its layout tree.
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            node_map: HashMap::new(),
            layout_cache: HashMap::new(),
        }
    }

    /// Perform a full DFS sync of the subtree rooted at `root`.
    /// Creates new Taffy nodes or updates existing ones. Rewires children.
    pub fn sync_subtree(&mut self, root: WidgetId, ctx: &AppContext) {
        // Clean up stale entries: remove node_map entries for WidgetIds no longer in the arena.
        // This is needed after recompose which unmounts widgets but doesn't notify the bridge.
        self.node_map.retain(|wid, nid| {
            if ctx.arena.contains_key(*wid) {
                true
            } else {
                // Remove the orphaned Taffy node
                let _ = self.tree.remove(*nid);
                self.layout_cache.remove(wid);
                false
            }
        });
        self.sync_node_dfs(root, ctx, false);
    }

    /// Perform an incremental DFS sync, skipping subtrees where dirty == false.
    /// Only widgets where `ctx.dirty[id] == true` are re-synced to Taffy.
    pub fn sync_dirty_subtree(&mut self, root: WidgetId, ctx: &AppContext) {
        self.sync_node_dfs(root, ctx, true);
    }

    /// Internal DFS sync.
    /// `dirty_only`: if true, skip widgets that are not marked dirty.
    fn sync_node_dfs(&mut self, id: WidgetId, ctx: &AppContext, dirty_only: bool) {
        let is_dirty = ctx.dirty.get(id).copied().unwrap_or(true);

        if dirty_only && !is_dirty {
            // Skip this subtree entirely — node_map entry is assumed to still be valid
            return;
        }

        let style = if let Some(cs) = ctx.computed_styles.get(id) {
            taffy_style_from_computed(cs)
        } else {
            Default::default()
        };

        // Create or update the Taffy node
        let node_id = if let Some(&existing) = self.node_map.get(&id) {
            // Update style on existing node (marks it dirty in Taffy's cache)
            self.tree.set_style(existing, style).unwrap();
            existing
        } else {
            // Create a new leaf node (children wired below)
            let nid = self.tree.new_leaf(style).unwrap();
            self.node_map.insert(id, nid);
            nid
        };

        // Recursively sync children
        let children = ctx.children.get(id).cloned().unwrap_or_default();
        for &child_id in &children {
            self.sync_node_dfs(child_id, ctx, dirty_only);
        }

        // Rewire children in Taffy to match ctx.children
        let child_node_ids: Vec<NodeId> = children
            .iter()
            .filter_map(|cid| self.node_map.get(cid).copied())
            .collect();
        self.tree.set_children(node_id, &child_node_ids).unwrap();
    }

    /// Compute layout for the entire tree rooted at `screen_id`.
    /// The available space is `cols` × `rows` terminal cells.
    /// After this call, `rect_for()` returns valid `Rect` values.
    ///
    /// Only the `screen_id` subtree is recalculated. Layout cache entries for
    /// other screens (background screens in the stack) are preserved so that
    /// layered multi-screen rendering works correctly.
    pub fn compute_layout(&mut self, screen_id: WidgetId, cols: u16, rows: u16, ctx: &AppContext) {
        let root = match self.node_map.get(&screen_id) {
            Some(&nid) => nid,
            None => return, // not synced yet
        };

        // Force the root screen node to fill the entire terminal.
        // Screen widgets have `size: Auto` by default (shrinks to content).
        // We override to Definite so the flex children get the full available space.
        {
            use taffy::prelude::Dimension;
            let mut root_style = self.tree.style(root).cloned().unwrap_or_default();
            root_style.size = taffy::geometry::Size {
                width: Dimension::length(cols as f32),
                height: Dimension::length(rows as f32),
            };
            self.tree.set_style(root, root_style).unwrap();
        }

        let available_space = Size {
            width: AvailableSpace::Definite(cols as f32),
            height: AvailableSpace::Definite(rows as f32),
        };

        self.tree.compute_layout(root, available_space).unwrap();

        // Update layout cache with absolute positions (Taffy positions are parent-relative).
        // Only remove entries for widgets in this screen's subtree before repopulating —
        // background screen entries are kept so layered rendering can paint them.
        let subtree = collect_subtree_dfs(screen_id, ctx);
        for &id in &subtree {
            self.layout_cache.remove(&id);
        }
        self.collect_absolute_rects(screen_id, 0.0, 0.0, ctx);
    }

    /// Walk the widget tree DFS, accumulating absolute x/y offsets from parent positions.
    fn collect_absolute_rects(
        &mut self,
        wid: WidgetId,
        offset_x: f32,
        offset_y: f32,
        ctx: &AppContext,
    ) {
        let nid = match self.node_map.get(&wid) {
            Some(&n) => n,
            None => return,
        };
        let layout = match self.tree.layout(nid) {
            Ok(l) => *l,
            Err(_) => return,
        };
        let abs_x = offset_x + layout.location.x;
        let abs_y = offset_y + layout.location.y;
        self.layout_cache.insert(
            wid,
            Rect {
                x: abs_x.floor() as u16,
                y: abs_y.floor() as u16,
                width: layout.size.width.round() as u16,
                height: layout.size.height.round() as u16,
            },
        );
        let children = ctx.children.get(wid).cloned().unwrap_or_default();
        for child in children {
            self.collect_absolute_rects(child, abs_x, abs_y, ctx);
        }
    }

    /// Return the computed screen `Rect` for a widget, if available.
    pub fn rect_for(&self, id: WidgetId) -> Option<Rect> {
        self.layout_cache.get(&id).copied()
    }

    /// Remove a widget subtree from the Taffy tree (bottom-up removal).
    /// Taffy requires children to be detached before parent removal.
    pub fn remove_subtree(&mut self, root: WidgetId, ctx: &AppContext) {
        let all = collect_subtree_dfs(root, ctx);
        // Bottom-up: remove children before parents
        for &id in all.iter().rev() {
            if let Some(nid) = self.node_map.remove(&id) {
                // Detach children first
                let _ = self.tree.set_children(nid, &[]);
                let _ = self.tree.remove(nid);
                self.layout_cache.remove(&id);
            }
        }
    }

    /// Access the raw layout cache (WidgetId → Rect).
    pub fn layout_cache(&self) -> &HashMap<WidgetId, Rect> {
        &self.layout_cache
    }
}

impl Default for TaffyBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// DFS traversal of widget subtree, returns IDs in pre-order.
fn collect_subtree_dfs(root: WidgetId, ctx: &AppContext) -> Vec<WidgetId> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        result.push(id);
        if let Some(children) = ctx.children.get(id) {
            for &child in children.iter().rev() {
                stack.push(child);
            }
        }
    }
    result
}
