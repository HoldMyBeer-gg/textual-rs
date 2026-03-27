---
phase: quick
plan: 260326-uwc
subsystem: core/mouse-capture
tags: [mouse, capture, stack, shift-bypass, resize-guard]
dependency_graph:
  requires: []
  provides: [mouse-capture-stack, shift-bypass, resize-guard]
  affects: [app-event-loop, widget-context]
tech_stack:
  added: []
  patterns: [push-pop-stack, deferred-drain]
key_files:
  created:
    - crates/textual-rs/tests/mouse_capture.rs
  modified:
    - crates/textual-rs/src/terminal.rs
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/app.rs
decisions:
  - MouseCaptureStack lives in terminal.rs near TerminalGuard (same domain)
  - Deferred push/pop pattern (RefCell<Vec> + Cell<usize>) matches existing screen push/pop pattern
  - Shift bypass placed BEFORE overlay routing so Shift always means native selection
metrics:
  duration: 378s
  completed: "2026-03-26"
  tasks: 2
  files: 4
---

# Quick Task 260326-uwc: Mouse Capture Push/Pop Stack and Shift-M Summary

Push/pop stack-based mouse capture with Shift bypass and resize guard, preventing competing enable/disable calls from clobbering each other.

## What Was Done

### Task 1: MouseCaptureStack type and AppContext integration (TDD)
- Created `MouseCaptureStack` in `terminal.rs` with `push()`, `pop()`, `is_enabled()`, `reset()` API
- Added `mouse_capture_stack`, `pending_mouse_push`, `pending_mouse_pops` fields to `AppContext`
- Added `push_mouse_capture()` and `pop_mouse_capture()` convenience methods on `AppContext` (deferred pattern)
- 10 unit tests covering all stack behaviors
- **Commits:** efb0aa6 (RED), e8f2ac2 (GREEN)

### Task 2: Shift bypass, event loop drain, and resize guard
- Added `mouse_capture_active: bool` field to `App` struct (tracks terminal state)
- Added `drain_mouse_capture_changes()` method that drains deferred pushes/pops and syncs terminal
- Called `drain_mouse_capture_changes()` after every `drain_message_queue()` in the event loop (4 call sites)
- Shift+mouse check at top of Mouse event arm (before overlay routing) -- `continue` to skip
- Capture-disabled check (`!mouse_capture_stack.is_enabled()`) -- `continue` to skip
- Resize handler re-sends current desired capture state to terminal
- Same guards added to `handle_mouse_event()` (TestApp path)
- **Commit:** 5b6a5f5

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all functionality is fully wired.

## Verification

- `cargo test --test mouse_capture` -- 10/10 pass
- `cargo test` -- all existing tests pass (no regressions)
- `cargo clippy` -- no new warnings

## Self-Check: PASSED

- All 4 key files exist on disk
- All 3 task commits verified in git log (efb0aa6, e8f2ac2, 5b6a5f5)
