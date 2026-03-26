---
phase: 04-built-in-widget-library
plan: 03
subsystem: widget-library
tags: [TextArea, Select, overlay, clipboard, cursor, reactive]
dependency_graph:
  requires: [04-01]
  provides: [TextArea, Select, SelectOverlay, pending_screen_pops]
  affects: [widget/mod.rs, lib.rs, widget/context.rs]
tech_stack:
  added: [arboard clipboard integration]
  patterns: [Reactive<Vec<String>>, Cell cursor state, pending_screen_pushes overlay pattern, pending_screen_pops deferred pop]
key_files:
  created:
    - crates/textual-rs/src/widget/text_area.rs
    - crates/textual-rs/src/widget/select.rs
    - crates/textual-rs/tests/snapshots/widget_tests__text_area_line_numbers.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_select_initial.snap
  modified:
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/tests/widget_tests.rs
decisions:
  - "TextArea tests verify state via rendered buffer rows rather than message queue — message queue is drained by process_event before assertions can inspect it"
  - "SelectOverlay keeps current field with #[allow(dead_code)] for future selected-item highlighting use"
  - "Select tests verify overlay push via pending_screen_pushes rather than full mount — overlay screen lifecycle requires event loop integration not yet in TestApp"
  - "pending_screen_pops added to AppContext with Cell<usize> for zero-borrow deferred pop from on_action(&self)"
metrics:
  duration: 8min
  completed: "2026-03-26T00:49:11Z"
  tasks_completed: 2
  files_created: 4
  files_modified: 4
---

# Phase 04 Plan 03: TextArea and Select Widgets Summary

TextArea multi-line editor with Reactive<Vec<String>> state, full cursor model, optional line numbers, and arboard clipboard; Select dropdown with SelectOverlay screen via pending_screen_pushes deferred mechanism and pending_screen_pops for overlay dismissal.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | TextArea multi-line editor widget | 5ccab0d | text_area.rs, mod.rs, lib.rs, widget_tests.rs, snapshot |
| 2 | Select dropdown widget with overlay screen | 3decf86 | select.rs, context.rs, mod.rs, lib.rs, widget_tests.rs, snapshot |

## What Was Built

### TextArea (WIDGET-04)

Multi-line text editor with:
- `Reactive<Vec<String>>` for line storage — reactive update triggers re-renders
- `Cell<usize>` for cursor_row, cursor_col, scroll_offset — zero borrow pressure in `on_action(&self)`
- Key bindings: Up/Down/Left/Right/Home/End/Ctrl+Left/Ctrl+Right/Backspace/Delete/Enter/Ctrl+C/Ctrl+V/Ctrl+A
- Character insertion via `on_event` — matches `KeyCode::Char(c)` with NONE/SHIFT modifiers, delegates to `on_action` for key bindings
- `delete_back`: removes char before cursor or joins with previous line
- `delete_forward`: removes char at cursor or joins with next line
- `newline`: splits current line at cursor position
- `copy`/`paste`: arboard::Clipboard integration, fail-silently on unavailable clipboard
- `show_line_numbers`: reserves left margin with right-aligned line number + space
- Auto-scroll: adjusts scroll_offset so cursor remains visible
- `messages::Changed { value: String }` posted after every text modification

### Select (WIDGET-08)

Dropdown widget with:
- `Reactive<usize>` for selected index, `Vec<String>` options
- Renders `"▼ {option}"` using `get_untracked()` in render()
- Enter binding calls `ctx.push_screen_deferred(Box::new(SelectOverlay {...}))`
- Private `SelectOverlay` struct with `Cell<usize>` cursor
- Overlay key bindings: Up/Down (wrapping), Enter (select + pop), Esc (cancel + pop)
- `on_action("select")`: posts `messages::Changed { value, index }` to originating Select via source_id, then calls `ctx.pop_screen_deferred()`
- `messages::Changed { value: String, index: usize }` defined in `pub mod messages`

### AppContext extensions

Added `pending_screen_pops: Cell<usize>` and `pop_screen_deferred()` method alongside the existing `pending_screen_pushes` mechanism.

## Verification

- `cargo test --test widget_tests` — 25 tests pass (0 failures)
- `cargo test --lib -q` — 99 tests pass (0 regressions)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tests checked message_queue after settle() drains it**
- **Found during:** Task 1
- **Issue:** Tests using `message_queue.borrow()` after `pilot` methods found empty queue — `process_event` drains via `drain_message_queue()` before test assertions run
- **Fix:** Changed text_area tests to verify state via rendered buffer rows (checking visible text in ratatui TestBackend buffer) instead of message queue inspection
- **Files modified:** crates/textual-rs/tests/widget_tests.rs

**2. [Rule 2 - Missing] pending_screen_pops mechanism missing from AppContext**
- **Found during:** Task 2
- **Issue:** Plan specified `pending_screen_pops: Cell<usize>` and `pop_screen_deferred()` but these were not yet in context.rs
- **Fix:** Added `pending_screen_pops: Cell<usize>` to AppContext struct, initialized in `new()`, added `pop_screen_deferred()` method
- **Files modified:** crates/textual-rs/src/widget/context.rs

**3. [Rule 1 - Test] select_choose_option test accessed private TestApp fields**
- **Found during:** Task 2
- **Issue:** Test used `test_app.app` and `test_app.terminal` directly — both are `pub(crate)` not `pub`
- **Fix:** Rewrote test to use public API (`inject_key_event`, `drain_messages`, `buffer()`) and verify overlay push via `pending_screen_pushes` borrow
- **Files modified:** crates/textual-rs/tests/widget_tests.rs

## Known Stubs

None — TextArea and Select are fully functional within the scope of this plan. The SelectOverlay is pushed to `pending_screen_pushes` but the event loop draining of that queue (mounting it as an active screen) is handled by the app event loop and was already in place from Plan 04-01.

## Self-Check: PASSED

Files exist:
- `crates/textual-rs/src/widget/text_area.rs` — FOUND
- `crates/textual-rs/src/widget/select.rs` — FOUND
- `crates/textual-rs/tests/snapshots/widget_tests__text_area_line_numbers.snap` — FOUND
- `crates/textual-rs/tests/snapshots/widget_tests__snapshot_select_initial.snap` — FOUND

Commits exist:
- 5ccab0d (TextArea) — FOUND
- 3decf86 (Select) — FOUND
