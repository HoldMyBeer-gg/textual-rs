---
phase: 09-complex-widgets
plan: "03"
subsystem: widget/toast
tags: [toast, notifications, ui, overlay, animation]
dependency_graph:
  requires: []
  provides: [WIDGET-13]
  affects: [AppContext, render_widget_tree, full_render_pass]
tech_stack:
  added: []
  patterns: [direct-buffer-rendering, RefCell-borrow-isolation, severity-coloring]
key_files:
  created:
    - crates/textual-rs/src/widget/toast.rs
  modified:
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/widget/mod.rs
decisions:
  - Toast uses Vec<ToastEntry> on AppContext (not active_overlay — that slot is single-instance only)
  - render_toasts() inserted before active_overlay paint so CommandPalette wins z-order
  - tick_toasts() inserted after spinner_tick.set in a separate borrow block to avoid RefCell double-borrow
  - 33ms per tick at ~30fps for countdown arithmetic
  - Persistent toasts (timeout_ms==0) never advance elapsed_ticks and are never removed
metrics:
  duration: "~5 minutes"
  completed_date: "2026-03-27"
  tasks_completed: 2
  files_created: 1
  files_modified: 3
---

# Phase 9 Plan 3: Toast Notification System Summary

Toast notification system with `Vec<ToastEntry>` on AppContext, `ctx.toast()` API, visual rendering in the bottom-right corner, severity-based coloring, auto-dismiss countdown, and correct z-order under CommandPalette.

## Tasks Completed

| # | Task | Commit | Status |
|---|------|--------|--------|
| 1 | Create toast types, render function, and unit tests | ddef204 | Done |
| 2 | Wire toast into AppContext, render pipeline, and module registry | 71a75dd | Done |

## What Was Built

### crates/textual-rs/src/widget/toast.rs (new)

- `ToastSeverity` enum: `Info`, `Warning`, `Error`
- `ToastEntry` struct: `message`, `severity`, `timeout_ms`, `elapsed_ticks`
- `push_toast()`: enforces max-5 cap by dropping oldest when full
- `tick_toasts()`: increments elapsed_ticks and removes expired toasts; persistent toasts (timeout_ms=0) never tick or expire
- `render_toasts()`: renders toasts in bottom-right corner with rounded borders, severity-colored borders (theme.primary/warning/error), dark background tint, and severity symbol (i/!/x)
- 9 unit tests covering all behaviors

### crates/textual-rs/src/widget/context.rs (modified)

- Added `toast_entries: RefCell<Vec<ToastEntry>>` field
- Added `toast(message, severity, timeout_ms)` method
- Added `toast_info(message)` convenience method (3000ms default)
- Initialized `toast_entries: RefCell::new(Vec::new())` in `AppContext::new()`

### crates/textual-rs/src/app.rs (modified)

- `render_widget_tree`: calls `render_toasts()` before `active_overlay` paint (correct z-order)
- `full_render_pass`: calls `tick_toasts()` after `spinner_tick.set()`, in isolated borrow block

### crates/textual-rs/src/widget/mod.rs (modified)

- Added `pub mod toast;` (alphabetical order between text_area and tree)

## Decisions Made

1. **Toast uses Vec on AppContext, not active_overlay**: `active_overlay` is single-instance only; toast stack requires multiple simultaneous entries.
2. **render_toasts before active_overlay**: Ensures CommandPalette (which uses active_overlay) always renders on top of toasts.
3. **Separate borrow blocks for toasts**: The `ctx.toast_entries.borrow()` in `render_widget_tree` is in its own block to drop before any other ctx borrows in that scope, avoiding RefCell panic.
4. **tick_toasts after terminal.draw()**: Countdown runs after the frame is drawn, not before, so toasts don't disappear one tick early.
5. **33ms per tick**: Consistent with the existing spinner_tick timing (30fps app loop).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all functionality is fully wired. `ctx.toast("msg", ToastSeverity::Info, 3000)` immediately adds to the stack and renders on the next frame.

## Self-Check: PASSED

- FOUND: crates/textual-rs/src/widget/toast.rs
- FOUND: .planning/phases/09-complex-widgets/09-03-SUMMARY.md
- FOUND: commit ddef204 (task 1)
- FOUND: commit 71a75dd (task 2)
