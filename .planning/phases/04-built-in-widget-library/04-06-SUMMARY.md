---
phase: 04-built-in-widget-library
plan: 06
subsystem: ui
tags: [rust, ratatui, widgets, datatable, tree, reactive, unicode-width]

# Dependency graph
requires:
  - phase: 04-01
    provides: Widget trait, Reactive<T>, KeyBinding, AppContext, TestApp/Pilot harness, on_mount/on_action/post_message patterns

provides:
  - DataTable widget with columnar data, sorting, two-axis scrolling, RowSelected/SortChanged messages
  - Tree widget with hierarchical nodes, guide chars, expand/collapse, NodeSelected/Expanded/Collapsed messages
  - ColumnDef and TreeNode supporting types
  - 11 new widget tests (5 DataTable, 6 Tree) plus 2 snapshot tests

affects: [05-layout-engine, any phase building data-heavy UIs, IRC log viewer]

# Tech tracking
tech-stack:
  added: [unicode-width 0.2]
  patterns:
    - "RefCell<Vec<T>> for rows enables sort_by_column(&self) without unsafe"
    - "RefCell<TreeNode> for root enables expand/collapse mutation from on_action(&self)"
    - "FlatEntry pre-order traversal with ancestor_is_last vec for guide char rendering"
    - "inject_key_event + check message_queue (before drain) for message assertion tests"
    - "INSTA_UPDATE=always for accepting new snapshot baselines"

key-files:
  created:
    - crates/textual-rs/src/widget/data_table.rs
    - crates/textual-rs/src/widget/tree_view.rs
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_data_table_3x3.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_tree_collapsed.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_tree_expanded.snap
  modified:
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/tests/widget_tests.rs

key-decisions:
  - "DataTable uses RefCell<Vec<Vec<String>>> for rows — enables sort_by_column(&self) without unsafe raw pointer casting"
  - "Tree named tree_view.rs (not tree.rs) to avoid conflict with existing widget/tree.rs module"
  - "Tree uses FlatEntry with ancestor_is_last Vec<bool> for O(n) guide char rendering without tree re-traversal"
  - "Tree root stored as RefCell<TreeNode> for interior mutability during expand/collapse from on_action(&self)"
  - "Message assertion tests use inject_key_event (not pilot.press) to inspect queue before drain"

patterns-established:
  - "RefCell interior mutability pattern for mutable state accessible from Widget trait's &self methods"
  - "FlatEntry pre-order flattening: rebuild on expand/collapse, render from flat list for O(1) row access"
  - "Guide char rendering: ancestor_is_last Vec tracks vertical line continuations at each depth"

requirements-completed: [WIDGET-10, WIDGET-11]

# Metrics
duration: 25min
completed: 2026-03-25
---

# Phase 4 Plan 6: DataTable and Tree Summary

**DataTable with columnar sorting/scrolling and Tree with guide-char hierarchy — the two most complex Wave 2 widgets, both with comprehensive interaction tests**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-03-25T00:00:00Z
- **Completed:** 2026-03-25T00:25:00Z
- **Tasks:** 2
- **Files modified:** 8 (including 5 new files)

## Accomplishments

- DataTable: header row with sort indicator (▲/▼), separator, data rows with cursor highlighting, auto-computed column widths via unicode_width, two-axis scrolling with vertical scrollbar, `s` key sorting with toggle ascending/descending
- Tree: pre-order FlatEntry flattening, guide chars (├── └── │ ), expand/collapse via Space key, Left/Right navigation, vertical scrollbar, NodeSelected/Expanded/Collapsed message emission
- 11 new widget tests plus 2 insta snapshot baselines (27 widget_tests total, all passing)
- 99 lib unit tests remain green (no regressions)

## Task Commits

1. **Task 1: DataTable widget** - `1cf2943` (feat)
2. **Task 2: Tree hierarchical view widget** - `e62a741` (feat)

## Files Created/Modified

- `crates/textual-rs/src/widget/data_table.rs` - DataTable widget with ColumnDef, RefCell rows, sorting, scrolling
- `crates/textual-rs/src/widget/tree_view.rs` - Tree widget with TreeNode, FlatEntry, guide chars, expand/collapse
- `crates/textual-rs/Cargo.toml` - Added unicode-width 0.2 dependency
- `crates/textual-rs/src/widget/mod.rs` - Added pub mod data_table; pub mod tree_view;
- `crates/textual-rs/src/lib.rs` - Added pub use for DataTable, ColumnDef, Tree, TreeNode
- `crates/textual-rs/tests/widget_tests.rs` - Added 11 data_table/tree tests, updated imports
- `crates/textual-rs/tests/snapshots/widget_tests__snapshot_data_table_3x3.snap` - Snapshot baseline
- `crates/textual-rs/tests/snapshots/widget_tests__snapshot_tree_collapsed.snap` - Snapshot baseline
- `crates/textual-rs/tests/snapshots/widget_tests__snapshot_tree_expanded.snap` - Snapshot baseline

## Decisions Made

- **DataTable rows in RefCell**: Plan called for `pub rows: Vec<Vec<String>>` but `sort_by_column` takes `&self` (Widget trait constraint). Used `rows: RefCell<Vec<Vec<String>>>` (private with `row_count()` accessor) instead of unsafe raw pointer casting. Added `pub fn row_count(&self) -> usize`.
- **Tree root as RefCell**: Plan specified `pub root: RefCell<TreeNode>` — implemented exactly, enabling `node_at_path_in_children` mutation during expand/collapse from `on_action(&self)`.
- **tree_view.rs naming**: Implemented as `tree_view.rs` per plan to avoid conflict with existing `widget/tree.rs` module (Pitfall 5 noted in plan).
- **FlatEntry with ancestor_is_last**: Extended `FlatEntry` with `ancestor_is_last: Vec<bool>` beyond the plan spec. This avoids tree re-traversal during render for guide char computation.
- **Test pattern for message assertions**: Toggle tests (navigate_and_expand, collapse_node) use `inject_key_event` (not `pilot.press`) to inspect the message queue before `drain_message_queue()` is called, following the same pattern as `button_press_enter_emits_pressed_message`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Changed rows from pub Vec to RefCell for sort_by_column(&self)**
- **Found during:** Task 1 (DataTable implementation)
- **Issue:** Plan specified `pub rows: Vec<Vec<String>>` but `sort_by_column(&self)` called from `on_action(&self)` needs mutation. Raw pointer cast is undefined behavior (denied by `#[deny(invalid_reference_casting)]`).
- **Fix:** Changed to `rows: RefCell<Vec<Vec<String>>>` (private field). Added `pub fn row_count(&self) -> usize` for external access. Tests use the widget via TestApp so this is transparent.
- **Files modified:** crates/textual-rs/src/widget/data_table.rs
- **Verification:** `cargo build` passes, all 5 DataTable tests pass
- **Committed in:** 1cf2943 (Task 1 commit)

**2. [Rule 1 - Bug] Used inject_key_event instead of pilot.press for expand/collapse message tests**
- **Found during:** Task 2 (Tree tests)
- **Issue:** `tree_navigate_and_expand` and `tree_collapse_node` used `pilot.press()` which calls `settle()` which drains the message queue via `drain_message_queue()`. Queue is empty by the time assertions run.
- **Fix:** Changed to `test_app.inject_key_event()` which dispatches the action without draining — consistent with `button_press_enter_emits_pressed_message` test pattern.
- **Files modified:** crates/textual-rs/tests/widget_tests.rs
- **Verification:** Both tests now pass
- **Committed in:** e62a741 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 — bugs caught during implementation)
**Impact on plan:** Both fixes essential for correctness. No scope creep.

## Issues Encountered

- `unicode_width` was not in Cargo.toml — added as explicit dependency (it was already a transitive dep via ratatui, making it available immediately after adding)
- Removed unused `node_at_path_mut` method to eliminate dead_code warning; replaced by `node_at_path_in_children` that navigates into `root.children` directly (the Tree never needs to navigate to root itself)

## Next Phase Readiness

- DataTable and Tree are ready for use in data-heavy application screens (IRC log, file browsers)
- Both widgets follow established patterns: Cell<Option<WidgetId>>, get_untracked() in render(), static &[KeyBinding], post_message from on_action
- 27 widget tests all passing, ready for Phase 5 layout engine integration

---
*Phase: 04-built-in-widget-library*
*Completed: 2026-03-25*
