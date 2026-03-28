//! Mouse hit testing: maps screen coordinates to the widget under the cursor.

use crate::widget::WidgetId;
use ratatui::layout::Rect;
use std::collections::HashMap;

/// Sparse cell-to-widget map for mouse hit testing.
///
/// Built from the layout cache after `TaffyBridge::compute_layout()`.
/// For overlapping widgets (e.g., modals), the widget appearing later in DFS order wins
/// (it overwrites earlier entries), which corresponds to higher z-order / rendered on top.
pub struct MouseHitMap {
    /// Sparse map: (col, row) → WidgetId
    cells: HashMap<(u16, u16), WidgetId>,
}

impl MouseHitMap {
    /// Build a hit map from the layout cache.
    ///
    /// `widgets_dfs` is the DFS order of widget IDs from the screen root.
    /// Later entries in `widgets_dfs` overwrite earlier ones for the same cell,
    /// providing correct z-ordering (later in DFS order = on top).
    pub fn build(widgets_dfs: &[WidgetId], layout_cache: &HashMap<WidgetId, Rect>) -> Self {
        let mut cells = HashMap::new();
        for &wid in widgets_dfs {
            if let Some(&rect) = layout_cache.get(&wid) {
                for row in rect.y..rect.y.saturating_add(rect.height) {
                    for col in rect.x..rect.x.saturating_add(rect.width) {
                        cells.insert((col, row), wid);
                    }
                }
            }
        }
        MouseHitMap { cells }
    }

    /// Return the `WidgetId` at the given terminal cell `(col, row)`, if any.
    pub fn hit_test(&self, col: u16, row: u16) -> Option<WidgetId> {
        self.cells.get(&(col, row)).copied()
    }
}
