---
phase: 09-complex-widgets
plan: "02"
subsystem: widget-library
tags: [widget, filesystem, tree, lazy-loading, workers]
dependency_graph:
  requires: [tree_view.rs, worker.rs, context.rs]
  provides: [DirectoryTree widget]
  affects: [widget/mod.rs, app.rs BUILTIN_CSS]
tech_stack:
  added: [walkdir = "2"]
  patterns: [inner-field delegation, lazy worker dispatch, canonical-path cycle detection]
key_files:
  created:
    - crates/textual-rs/src/widget/directory_tree.rs
  modified:
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/widget/tree_view.rs
decisions:
  - TreeNode derives Clone to allow Vec<TreeNode> cloning in recursive child replacement
  - tree_key_bindings() pub fn added to tree_view.rs to expose static bindings without borrow issues
  - DirectoryTree.key_bindings() returns tree_key_bindings() directly (avoids RefCell lifetime error)
  - trigger_root_load() kept but #[allow(dead_code)] since on_mount lacks ctx; initial load triggered by user's first Space/Right press
  - Tree.mark_dirty(), cursor_path(), is_expanded_at() added as pub helpers for DirectoryTree delegation
metrics:
  duration_minutes: 25
  completed: "2026-03-27"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 5
---

# Phase 9 Plan 02: DirectoryTree Widget Summary

DirectoryTree widget with lazy-loaded filesystem browsing via workers, symlink detection, cycle safety, and hidden file filtering.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add walkdir dep and implement DirectoryTree | 5d64549 | directory_tree.rs, Cargo.toml, tree_view.rs |
| 2 | Register module and add BUILTIN_CSS entry | bc47d0d | widget/mod.rs, app.rs |

## What Was Built

A `DirectoryTree` widget that wraps the existing `Tree` widget and provides:

- **Lazy loading**: Directory children are loaded on first expand via `ctx.run_worker`, never blocking on_event. The root node shows a "Loading..." placeholder until the first Space/Right keypress triggers expansion and a worker fires.
- **Worker dispatch**: `on_event` receives `WorkerResult<Vec<DirEntryInfo>>` and calls `apply_worker_result()` which walks the tree by node.data path string to find and replace children.
- **Caching**: `loaded_paths: RefCell<HashSet<PathBuf>>` prevents re-reading on re-expand. `loading_paths` prevents double-spawning the same directory.
- **Symlink detection**: Uses `symlink_metadata()` (not `metadata()`) per D-14. Symlinked entries show with `@` suffix and `is_dir = false` so they cannot be expanded.
- **Hidden files**: Filtered by default. Unix: dot-prefix. Windows: dot-prefix OR `FILE_ATTRIBUTE_HIDDEN` attribute check.
- **Cycle detection**: `visited_canonical: RefCell<HashSet<PathBuf>>` tracks canonicalized paths. Root canonical added on construction. On expand, canonicalize and check; if already visited, label becomes "X (cycle)" and no worker is spawned. NTFS junctions (which `is_symlink()` doesn't catch) are safe because canonicalize resolves them.
- **Messages**: `messages::FileSelected { path }` and `messages::DirectorySelected { path }` emitted on Enter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing API] Added pub helpers to Tree for DirectoryTree delegation**
- **Found during:** Task 1
- **Issue:** `Tree.dirty`, `flat_entries`, `viewport_height` are private fields; DirectoryTree needed to mark tree dirty after external child mutations, and needed cursor path for on_action.
- **Fix:** Added `Tree::mark_dirty()`, `Tree::cursor_path()`, `Tree::is_expanded_at()` as pub methods. Added `tree_key_bindings() -> &'static [KeyBinding]` pub fn to fix E0515 (can't return ref to temporary borrow).
- **Files modified:** crates/textual-rs/src/widget/tree_view.rs
- **Commit:** 5d64549

**2. [Rule 2 - Missing trait] Added #[derive(Clone)] to TreeNode**
- **Found during:** Task 1 compile
- **Issue:** `replace_children_by_data` recursion needed to clone `Vec<TreeNode>` when calling into subtrees; TreeNode had no Clone.
- **Fix:** Added `#[derive(Clone)]` to `TreeNode` in tree_view.rs.
- **Files modified:** crates/textual-rs/src/widget/tree_view.rs
- **Commit:** 5d64549

## Known Stubs

- Initial load of root directory only happens on first user-triggered expand (Space/Right key), not automatically on mount. `trigger_root_load()` is implemented but requires `ctx` which is not available in `on_mount()`. The root node shows a "Loading..." placeholder child so it appears expandable. This is consistent with the Tree widget model and does not block the plan's goal.

## Self-Check: PASSED

- directory_tree.rs: FOUND
- Commit 5d64549: FOUND
- Commit bc47d0d: FOUND
