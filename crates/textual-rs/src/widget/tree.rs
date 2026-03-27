use super::{Widget, WidgetId};
use super::context::AppContext;
use crate::css::types::{ComputedStyle, PseudoClassSet};

// ---- Helper: DFS traversal ----

fn collect_subtree_dfs(root: WidgetId, ctx: &AppContext) -> Vec<WidgetId> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        result.push(id);
        if let Some(children) = ctx.children.get(id) {
            // push in reverse so we process left-to-right
            for &child in children.iter().rev() {
                stack.push(child);
            }
        }
    }
    result
}

fn collect_focusable_dfs(root: WidgetId, ctx: &AppContext) -> Vec<WidgetId> {
    collect_subtree_dfs(root, ctx)
        .into_iter()
        .filter(|&id| ctx.arena.get(id).map(|w| w.can_focus()).unwrap_or(false))
        .collect()
}

// ---- Public API ----

/// Mount a widget into the arena. Initializes all SecondaryMap entries.
/// Calls on_mount via shared reference (no borrow conflict).
/// Returns the new WidgetId.
pub fn mount_widget(widget: Box<dyn Widget>, parent: Option<WidgetId>, ctx: &mut AppContext) -> WidgetId {
    let id = ctx.arena.insert(widget);

    // Initialize SecondaryMap entries
    ctx.children.insert(id, Vec::new());
    ctx.parent.insert(id, parent);
    ctx.computed_styles.insert(id, ComputedStyle::default());
    ctx.inline_styles.insert(id, Vec::new());
    ctx.dirty.insert(id, true);
    ctx.pseudo_classes.insert(id, PseudoClassSet::default());

    // Wire to parent
    if let Some(parent_id) = parent {
        if let Some(siblings) = ctx.children.get_mut(parent_id) {
            siblings.push(id);
        }
    }

    // Call on_mount (takes &self, no borrow conflict)
    ctx.arena[id].on_mount(id);

    id
}

/// Unmount a widget and all its descendants recursively. Removes from arena and all SecondaryMaps.
pub fn unmount_widget(id: WidgetId, ctx: &mut AppContext) {
    if !ctx.arena.contains_key(id) {
        eprintln!("WidgetId {:?} not found in arena -- unmount is a no-op", id);
        return;
    }

    // Collect all descendants (bottom-up: children before parent)
    let all = collect_subtree_dfs(id, ctx);
    // Reverse for bottom-up removal
    let all_bottom_up: Vec<WidgetId> = all.iter().rev().copied().collect();

    // Remove id from parent's children list
    if let Some(Some(parent_id)) = ctx.parent.get(id).copied() {
        if let Some(siblings) = ctx.children.get_mut(parent_id) {
            siblings.retain(|&c| c != id);
        }
    }

    // Remove all descendants bottom-up
    for did in &all_bottom_up {
        let did = *did;
        // call on_unmount
        if let Some(w) = ctx.arena.get(did) {
            w.on_unmount(did);
        }
        // Cancel any active workers for this widget before removing it
        ctx.cancel_workers(did);
        ctx.arena.remove(did);
        ctx.children.remove(did);
        ctx.parent.remove(did);
        ctx.computed_styles.remove(did);
        ctx.inline_styles.remove(did);
        ctx.dirty.remove(did);
        ctx.pseudo_classes.remove(did);
    }

    // Clear focus if focused widget was part of the subtree
    if let Some(focused) = ctx.focused_widget {
        if all_bottom_up.contains(&focused) {
            ctx.focused_widget = None;
        }
    }
}

/// Mount all children returned by widget.compose() under parent_id (one level only).
pub fn compose_children(parent_id: WidgetId, ctx: &mut AppContext) {
    let children = ctx.arena[parent_id].compose();
    for child in children {
        mount_widget(child, Some(parent_id), ctx);
    }
}

/// Recursively compose the entire subtree rooted at `root_id`.
/// Calls compose_children on root, then on each mounted child, depth-first.
pub fn compose_subtree(root_id: WidgetId, ctx: &mut AppContext) {
    compose_children(root_id, ctx);
    // Collect child ids to avoid borrow issues while recursing
    let child_ids: Vec<WidgetId> = ctx.children.get(root_id).cloned().unwrap_or_default();
    for child_id in child_ids {
        compose_subtree(child_id, ctx);
    }
}

/// Recompose a widget: unmount all its children and recompose from scratch.
/// Used when a widget's compose() output changes dynamically (e.g. tab switching).
pub fn recompose_widget(id: WidgetId, ctx: &mut AppContext) {
    // Unmount all existing children
    let children: Vec<WidgetId> = ctx.children.get(id).cloned().unwrap_or_default();
    for child_id in children {
        unmount_widget(child_id, ctx);
    }
    // Recompose
    compose_subtree(id, ctx);
    // Mark the entire subtree dirty so the layout bridge creates new Taffy nodes
    mark_subtree_dirty(id, ctx);
    // Re-focus the first focusable child (e.g. TabBar after tab switch)
    let new_children: Vec<WidgetId> = ctx.children.get(id).cloned().unwrap_or_default();
    for child_id in new_children {
        if ctx.arena.get(child_id).is_some_and(|w| w.can_focus()) {
            ctx.focused_widget = Some(child_id);
            break;
        }
    }
}

/// Mark a widget and all its descendants as dirty.
fn mark_subtree_dirty(id: WidgetId, ctx: &mut AppContext) {
    ctx.dirty.insert(id, true);
    let children: Vec<WidgetId> = ctx.children.get(id).cloned().unwrap_or_default();
    for child_id in children {
        mark_subtree_dirty(child_id, ctx);
    }
}

/// Push a new screen onto the screen stack and compose its entire subtree.
pub fn push_screen(screen: Box<dyn Widget>, ctx: &mut AppContext) -> WidgetId {
    let id = mount_widget(screen, None, ctx);
    ctx.screen_stack.push(id);
    compose_subtree(id, ctx);
    id
}

/// Pop the top screen from the stack and unmount its entire subtree.
pub fn pop_screen(ctx: &mut AppContext) -> Option<WidgetId> {
    if let Some(id) = ctx.screen_stack.pop() {
        unmount_widget(id, ctx);
        Some(id)
    } else {
        None
    }
}

/// Advance focus to the next focusable widget in depth-first DOM order (wrapping).
pub fn advance_focus(ctx: &mut AppContext) {
    let root = match ctx.screen_stack.last().copied() {
        Some(r) => r,
        None => return,
    };

    let focusable = collect_focusable_dfs(root, ctx);
    if focusable.is_empty() {
        return;
    }

    let current = ctx.focused_widget;
    let next_idx = match current.and_then(|f| focusable.iter().position(|&id| id == f)) {
        Some(idx) => (idx + 1) % focusable.len(),
        None => 0,
    };

    // Remove Focus from old
    if let Some(old) = current {
        if let Some(ps) = ctx.pseudo_classes.get_mut(old) {
            ps.remove(&crate::css::types::PseudoClass::Focus);
        }
    }

    let next_id = focusable[next_idx];
    ctx.focused_widget = Some(next_id);
    if let Some(ps) = ctx.pseudo_classes.get_mut(next_id) {
        ps.insert(crate::css::types::PseudoClass::Focus);
    }
}

/// Advance focus backward (Shift+Tab) — previous focusable widget with wrapping.
pub fn advance_focus_backward(ctx: &mut AppContext) {
    let root = match ctx.screen_stack.last().copied() {
        Some(r) => r,
        None => return,
    };

    let focusable = collect_focusable_dfs(root, ctx);
    if focusable.is_empty() {
        return;
    }

    let current = ctx.focused_widget;
    let prev_idx = match current.and_then(|f| focusable.iter().position(|&id| id == f)) {
        Some(0) => focusable.len() - 1,
        Some(idx) => idx - 1,
        None => focusable.len() - 1,
    };

    // Remove Focus from old
    if let Some(old) = current {
        if let Some(ps) = ctx.pseudo_classes.get_mut(old) {
            ps.remove(&crate::css::types::PseudoClass::Focus);
        }
    }

    let prev_id = focusable[prev_idx];
    ctx.focused_widget = Some(prev_id);
    if let Some(ps) = ctx.pseudo_classes.get_mut(prev_id) {
        ps.insert(crate::css::types::PseudoClass::Focus);
    }
}

/// Mark a widget dirty and propagate the dirty flag up to all ancestors.
pub fn mark_widget_dirty(id: WidgetId, ctx: &mut AppContext) {
    let mut current = Some(id);
    while let Some(cid) = current {
        if let Some(dirty) = ctx.dirty.get_mut(cid) {
            if *dirty {
                // Already dirty — ancestors already dirty, stop bubbling
                break;
            }
            *dirty = true;
        }
        current = ctx.parent.get(cid).and_then(|p| *p);
    }
}

/// Clear dirty flag for an entire subtree starting at root.
pub fn clear_dirty_subtree(root: WidgetId, ctx: &mut AppContext) {
    let all = collect_subtree_dfs(root, ctx);
    for id in all {
        if let Some(dirty) = ctx.dirty.get_mut(id) {
            *dirty = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    struct SimpleWidget {
        focusable: bool,
    }

    impl SimpleWidget {
        fn new(focusable: bool) -> Self {
            SimpleWidget { focusable }
        }
    }

    impl Widget for SimpleWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str { "SimpleWidget" }
        fn can_focus(&self) -> bool { self.focusable }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            // Return pre-set children (they've been moved out, so this only works once)
            // For testing compose_children with a static list, we use a separate test widget
            vec![]
        }
    }

    /// Widget that returns 2 children from compose()
    struct ParentWidget;

    impl Widget for ParentWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str { "ParentWidget" }
        fn compose(&self) -> Vec<Box<dyn Widget>> {
            vec![
                Box::new(SimpleWidget::new(false)),
                Box::new(SimpleWidget::new(true)),
            ]
        }
    }

    #[test]
    fn mount_widget_inserts_into_arena() {
        let mut ctx = AppContext::new();
        let widget: Box<dyn Widget> = Box::new(SimpleWidget::new(false));
        let id = mount_widget(widget, None, &mut ctx);

        assert!(ctx.arena.contains_key(id));
        assert_eq!(ctx.arena[id].widget_type_name(), "SimpleWidget");
        assert!(ctx.children.contains_key(id));
        assert_eq!(ctx.parent[id], None);
        assert!(ctx.computed_styles.contains_key(id));
        assert_eq!(ctx.dirty[id], true);
    }

    #[test]
    fn mount_widget_wires_parent_child() {
        let mut ctx = AppContext::new();
        let parent_id = mount_widget(Box::new(SimpleWidget::new(false)), None, &mut ctx);
        let child_id = mount_widget(Box::new(SimpleWidget::new(false)), Some(parent_id), &mut ctx);

        assert_eq!(ctx.parent[child_id], Some(parent_id));
        assert!(ctx.children[parent_id].contains(&child_id));
    }

    #[test]
    fn unmount_widget_removes_widget_and_secondarymaps() {
        let mut ctx = AppContext::new();
        let id = mount_widget(Box::new(SimpleWidget::new(false)), None, &mut ctx);
        assert!(ctx.arena.contains_key(id));

        unmount_widget(id, &mut ctx);

        assert!(!ctx.arena.contains_key(id));
        assert!(!ctx.children.contains_key(id));
        assert!(!ctx.parent.contains_key(id));
        assert!(!ctx.computed_styles.contains_key(id));
        assert!(!ctx.dirty.contains_key(id));
    }

    #[test]
    fn unmount_widget_removes_descendants_recursively() {
        let mut ctx = AppContext::new();
        let root_id = mount_widget(Box::new(SimpleWidget::new(false)), None, &mut ctx);
        let child_id = mount_widget(Box::new(SimpleWidget::new(false)), Some(root_id), &mut ctx);
        let grandchild_id = mount_widget(Box::new(SimpleWidget::new(false)), Some(child_id), &mut ctx);

        unmount_widget(root_id, &mut ctx);

        assert!(!ctx.arena.contains_key(root_id));
        assert!(!ctx.arena.contains_key(child_id));
        assert!(!ctx.arena.contains_key(grandchild_id));
    }

    #[test]
    fn unmount_clears_focus_if_focused_widget_removed() {
        let mut ctx = AppContext::new();
        let id = mount_widget(Box::new(SimpleWidget::new(true)), None, &mut ctx);
        ctx.focused_widget = Some(id);

        unmount_widget(id, &mut ctx);

        assert!(ctx.focused_widget.is_none());
    }

    #[test]
    fn compose_children_mounts_returned_children() {
        let mut ctx = AppContext::new();
        let parent_id = mount_widget(Box::new(ParentWidget), None, &mut ctx);
        compose_children(parent_id, &mut ctx);

        // ParentWidget.compose() returns 2 children
        assert_eq!(ctx.children[parent_id].len(), 2);
        for &child_id in &ctx.children[parent_id] {
            assert_eq!(ctx.parent[child_id], Some(parent_id));
        }
    }

    #[test]
    fn push_screen_adds_to_stack() {
        let mut ctx = AppContext::new();
        assert!(ctx.screen_stack.is_empty());

        let screen_id = push_screen(Box::new(SimpleWidget::new(false)), &mut ctx);

        assert_eq!(ctx.screen_stack.len(), 1);
        assert_eq!(ctx.screen_stack[0], screen_id);
        assert!(ctx.arena.contains_key(screen_id));
    }

    #[test]
    fn pop_screen_removes_subtree() {
        let mut ctx = AppContext::new();
        let screen_id = push_screen(Box::new(ParentWidget), &mut ctx);

        // Should have screen + its 2 children from compose
        let children_count = ctx.children[screen_id].len();
        assert_eq!(children_count, 2);

        let popped = pop_screen(&mut ctx);
        assert_eq!(popped, Some(screen_id));
        assert!(ctx.screen_stack.is_empty());
        assert!(!ctx.arena.contains_key(screen_id));
    }

    #[test]
    fn advance_focus_cycles_in_dfs_order() {
        let mut ctx = AppContext::new();

        // Create: screen -> [non-focusable, focusable1, focusable2, focusable3]
        struct ScreenWithFocusable;
        impl Widget for ScreenWithFocusable {
            fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
            fn widget_type_name(&self) -> &'static str { "Screen" }
            fn compose(&self) -> Vec<Box<dyn Widget>> {
                vec![
                    Box::new(SimpleWidget::new(false)), // non-focusable
                    Box::new(SimpleWidget::new(true)),  // focusable1
                    Box::new(SimpleWidget::new(true)),  // focusable2
                    Box::new(SimpleWidget::new(true)),  // focusable3
                ]
            }
        }

        push_screen(Box::new(ScreenWithFocusable), &mut ctx);
        let screen_id = ctx.screen_stack[0];
        let children = ctx.children[screen_id].clone();

        // children[0] = non-focusable, [1],[2],[3] = focusable
        let f1 = children[1];
        let f2 = children[2];
        let f3 = children[3];

        assert!(ctx.focused_widget.is_none());

        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(f1));

        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(f2));

        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(f3));

        // Wrap around
        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(f1));
    }

    #[test]
    fn advance_focus_backward_wraps_first_to_last() {
        let mut ctx = AppContext::new();

        struct TwoFocusable;
        impl Widget for TwoFocusable {
            fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
            fn widget_type_name(&self) -> &'static str { "Screen" }
            fn compose(&self) -> Vec<Box<dyn Widget>> {
                vec![
                    Box::new(SimpleWidget::new(true)),
                    Box::new(SimpleWidget::new(true)),
                ]
            }
        }

        push_screen(Box::new(TwoFocusable), &mut ctx);
        let screen_id = ctx.screen_stack[0];
        let children = ctx.children[screen_id].clone();
        let f1 = children[0];
        let f2 = children[1];

        // Start at f1
        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(f1));

        // Backward from f1 should wrap to f2
        advance_focus_backward(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(f2));
    }

    #[test]
    fn advance_focus_skips_non_focusable() {
        let mut ctx = AppContext::new();

        struct OneFocusable;
        impl Widget for OneFocusable {
            fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
            fn widget_type_name(&self) -> &'static str { "Screen" }
            fn compose(&self) -> Vec<Box<dyn Widget>> {
                vec![
                    Box::new(SimpleWidget::new(false)),
                    Box::new(SimpleWidget::new(false)),
                    Box::new(SimpleWidget::new(true)),
                ]
            }
        }

        push_screen(Box::new(OneFocusable), &mut ctx);
        let screen_id = ctx.screen_stack[0];
        let children = ctx.children[screen_id].clone();
        let focusable = children[2];

        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(focusable));
        // Second advance wraps back to only focusable
        advance_focus(&mut ctx);
        assert_eq!(ctx.focused_widget, Some(focusable));
    }

    #[test]
    fn mark_widget_dirty_bubbles_to_ancestors() {
        let mut ctx = AppContext::new();
        let root_id = mount_widget(Box::new(SimpleWidget::new(false)), None, &mut ctx);
        let child_id = mount_widget(Box::new(SimpleWidget::new(false)), Some(root_id), &mut ctx);
        let leaf_id = mount_widget(Box::new(SimpleWidget::new(false)), Some(child_id), &mut ctx);

        // Clear all dirty flags first
        clear_dirty_subtree(root_id, &mut ctx);
        assert_eq!(ctx.dirty[root_id], false);
        assert_eq!(ctx.dirty[child_id], false);
        assert_eq!(ctx.dirty[leaf_id], false);

        // Mark leaf dirty
        mark_widget_dirty(leaf_id, &mut ctx);
        assert_eq!(ctx.dirty[leaf_id], true);
        assert_eq!(ctx.dirty[child_id], true);
        assert_eq!(ctx.dirty[root_id], true);
    }

    #[test]
    fn clear_dirty_subtree_clears_entire_subtree() {
        let mut ctx = AppContext::new();
        let root_id = mount_widget(Box::new(SimpleWidget::new(false)), None, &mut ctx);
        let child_id = mount_widget(Box::new(SimpleWidget::new(false)), Some(root_id), &mut ctx);

        // All should be dirty after mount
        assert_eq!(ctx.dirty[root_id], true);
        assert_eq!(ctx.dirty[child_id], true);

        clear_dirty_subtree(root_id, &mut ctx);
        assert_eq!(ctx.dirty[root_id], false);
        assert_eq!(ctx.dirty[child_id], false);
    }
}
