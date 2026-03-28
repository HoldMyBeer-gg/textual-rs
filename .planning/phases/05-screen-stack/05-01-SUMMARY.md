---
phase: 05
plan: 01
subsystem: screen-stack
tags: [navigation, modal, rendering, focus, integration-tests]
one_liner: "Multi-screen layered rendering, modal focus scoping, and pop-last-screen no-op fix"

dependency_graph:
  requires: []
  provides: [multi-screen-rendering, modal-input-blocking, screen-stack-tests, screen-stack-demo]
  affects: [app.rs, layout/bridge.rs]

tech_stack:
  added: []
  patterns:
    - "Render all screens bottom-to-top in terminal.draw() for correct modal layering"
    - "layout_cache partial clear: only evict top screen entries, preserve background screen positions"
    - "Guard screen_stack.len() <= 1 before pop to enforce no-op contract on last screen"

key_files:
  created:
    - crates/textual-rs/tests/screen_stack.rs
    - crates/textual-rs/examples/screen_stack.rs
    - .planning/phases/05-screen-stack/05-01-PLAN.md
  modified:
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/layout/bridge.rs

decisions:
  - "compute_layout clears only top-screen subtree entries from layout_cache; background entries preserved for layered render"
  - "full_render_pass renders all screens bottom-to-top; CSS/layout/dirty-clear remain top-screen only (perf)"
  - "pop_screen_deferred no-op on last screen enforced in process_deferred_screens (len <= 1 guard)"

metrics:
  duration_minutes: 23
  completed: "2026-03-28"
  tasks_completed: 3
  files_changed: 4
---

# Phase 05 Plan 01: Screen Stack — Multi-Screen Rendering and Demo

## Summary

Multi-screen layered rendering so that modal screens visually overlay their background screens,
plus integration tests and a demo example covering the full push/pop/modal lifecycle.

## What Was Built

### Task 1: Multi-Screen Layered Rendering (b3eafe9)

**Problem:** `full_render_pass` only rendered the top screen. When a `ModalScreen` was pushed,
the background was blank — the modal appeared on an empty frame.

**Fix 1 — bridge.rs:** `compute_layout` previously called `self.layout_cache.clear()`, wiping
all widget positions including background screens. Changed to collect the top screen's subtree
IDs first, remove only those entries, then repopulate — background screen positions are preserved.

**Fix 2 — app.rs:** `full_render_pass` now iterates `screen_stack` and calls `render_widget_tree`
for each screen from bottom to top inside a single `terminal.draw()` closure. CSS cascade, layout
sync, dirty clearing, and the mouse hit map remain top-screen-only (frozen background = no
re-layout cost).

### Task 2: Integration Tests + Bug Fix (7e64228)

Added `tests/screen_stack.rs` with 8 integration tests:
- `screen_stack_initial_state` — push mounts widget, auto-focuses first focusable
- `screen_stack_keyboard_scoped_to_top_screen` — base screen receives no keys when overlay is on top
- `screen_stack_focus_history_tracks_pushes_and_pops` — focus_history len equals stack depth
- `screen_stack_pop_last_screen_is_noop` — stack stays at 1 when only screen is popped
- `screen_stack_focus_restored_after_pop` — exact focus widget restored after multi-pop
- `screen_stack_modal_dismissed_via_pop` — Esc in modal calls pop_screen_deferred correctly
- `screen_stack_multi_screen_renders_all_screens` — top screen 'M' overwrites background 'B'
- `screen_stack_is_modal_returns_true_for_modal_screen` — ModalScreen.is_modal() == true

**Bug fixed (Rule 1):** `process_deferred_screens` did not guard against popping the last screen,
despite `pop_screen_deferred` being documented as a no-op on the last screen. Added
`if self.ctx.screen_stack.len() <= 1 { break; }` guard.

### Task 3: Screen Stack Demo (841e180)

Created `examples/screen_stack.rs` demonstrating:
- `MainScreen` with 'n' (push second screen) and 'm' (open modal) key bindings
- `SecondScreen` with 'b'/'Esc' to pop back and 'm' for a nested modal
- `ModalDialog` — focusable widget with Enter/Esc dismiss via `pop_screen_deferred()`
- `ModalScreen` wraps `ModalDialog` — background screen visible underneath
- `NavButtons` — reusable button group component

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] pop_screen_deferred no-op on last screen not enforced**
- **Found during:** Task 2 (test `screen_stack_pop_last_screen_is_noop` failed)
- **Issue:** `process_deferred_screens` looped `pops` times calling `pop_screen()` without
  checking if the stack was already at 1. The last screen could be unmounted, leaving an empty
  stack and blank terminal.
- **Fix:** Added `if self.ctx.screen_stack.len() <= 1 { break; }` guard before each `pop_screen()` call.
- **Files modified:** `crates/textual-rs/src/app.rs`
- **Commit:** 7e64228

## Requirements Satisfied

- **NAV-01**: `ctx.push_screen_deferred()` pushes screen, focuses first focusable widget
- **NAV-02**: `ctx.pop_screen_deferred()` removes top screen, restores exact prior focus
- **NAV-03**: `ModalScreen` blocks all keyboard/mouse from screens below (focus scoped to top,
  hit map built from top screen only)

## Known Stubs

None — all three requirements are fully wired and tested.

## Self-Check: PASSED
