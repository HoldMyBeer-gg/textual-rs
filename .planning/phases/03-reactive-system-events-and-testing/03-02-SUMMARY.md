---
phase: 03-reactive-system-events-and-testing
plan: "02"
subsystem: event-dispatch
tags: [event-system, dispatch, message-passing, key-bindings, mouse-events, tdd]
dependency_graph:
  requires: ["03-01"]
  provides: [event-dispatch, message-bubbling, key-bindings, mouse-routing, timer]
  affects: [app-event-loop, widget-trait, app-context]
tech_stack:
  added: []
  patterns: [message-bubbling, parent-chain-dispatch, refcell-interior-mutability, hit-test-mouse-routing]
key_files:
  created:
    - crates/textual-rs/src/event/mod.rs
    - crates/textual-rs/src/event/message.rs
    - crates/textual-rs/src/event/dispatch.rs
    - crates/textual-rs/src/event/keybinding.rs
    - crates/textual-rs/src/event/timer.rs
  modified:
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/terminal.rs
decisions:
  - "message_queue uses RefCell<Vec<...>> on AppContext — allows post_message(&self) from on_event/on_action without borrow conflict"
  - "AppEvent::Message variant rejected — Box<dyn Any> breaks Clone/Debug on AppEvent; message_queue field is cleaner"
  - "drain_message_queue bounded at 100 iterations — prevents infinite message loops while supporting recursive dispatch"
  - "Mouse Down/ScrollUp/ScrollDown dispatch via hit_test; drag/hover/move ignored for now"
  - "Old input_buffer.push/pop removed from event loop — key events flow through proper dispatch system now"
metrics:
  duration: 4min
  completed: "2026-03-25"
  tasks: 2
  files: 9
---

# Phase 03 Plan 02: Event Dispatch System Summary

**One-liner:** Event dispatch system with Message trait, parent-chain bubbling via dyn Any downcasting, KeyBinding struct + action dispatch, timer spawning, and App event loop wired to route key/mouse events through focused widget and MouseHitMap.

## Tasks Completed

| # | Name | Commit | Key Files |
|---|------|--------|-----------|
| 1 | Create event module — Message trait, dispatch_message bubbling, KeyBinding, timer | `a936e16` | event/mod.rs, message.rs, dispatch.rs, keybinding.rs, timer.rs, widget/mod.rs, widget/context.rs |
| 2 | Wire event dispatch into App event loop — key routing, mouse routing, message draining | `caa4895` | app.rs, terminal.rs |

## What Was Built

### Event Module (`crates/textual-rs/src/event/`)

**`message.rs`** — `Message` trait: marker over `Any + 'static` with optional `bubbles()` flag. Plain structs implement this to become dispatchable messages.

**`dispatch.rs`** — Two functions:
- `collect_parent_chain(start, ctx)` — returns `[start, parent, grandparent, ...]` in bottom-up order by walking `ctx.parent` chain
- `dispatch_message(target, message: &dyn Any, ctx)` — walks parent chain, calls `widget.on_event()` on each, stops on `EventPropagation::Stop`

**`keybinding.rs`** — `KeyBinding` struct with `key: KeyCode`, `modifiers: KeyModifiers`, `action: &'static str`, `description: &'static str`, `show: bool`. `matches()` method checks exact key+modifier equality.

**`timer.rs`** — `spawn_tick_timer(tx, interval)` spawns a Tokio LocalSet task that posts `AppEvent::Tick` at configurable intervals.

### Widget Trait Extensions (`widget/mod.rs`)

Three default methods added to `Widget`:
- `on_event(&self, event: &dyn Any, ctx: &AppContext) -> EventPropagation` — handles dispatched messages via downcasting
- `key_bindings(&self) -> &[KeyBinding]` — declares keyboard shortcuts
- `on_action(&self, action: &str, ctx: &AppContext)` — called when a declared key binding fires

### AppContext Extensions (`widget/context.rs`)

- `message_queue: RefCell<Vec<(WidgetId, Box<dyn Any>)>>` — interior-mutable queue allows `post_message(&self)` from widget handlers
- `post_message(&self, source: WidgetId, message: impl Any + 'static)` — enqueues a message for dispatch in next event loop iteration

### App Event Loop (`app.rs`)

Key event flow:
1. Non-press events (release/repeat) ignored
2. Global quit (`q`, `Ctrl+C`) checked first
3. Focused widget's `key_bindings()` checked; matching binding fires `on_action`
4. If no binding matched, `dispatch_message(focused_id, &k, &ctx)` called (bubbles up)
5. Tab/Shift+Tab handled for focus cycling
6. `drain_message_queue()` called after key handling

Mouse event flow:
1. `Down`, `ScrollDown`, `ScrollUp` hit-tested via `MouseHitMap`
2. `dispatch_message(target_id, &m, &ctx)` called on hit widget (bubbles up)
3. `drain_message_queue()` called after mouse handling
4. Drag/hover/move events ignored for now

### Terminal (`terminal.rs`)

`TerminalGuard::new()` now enables mouse capture (`EnableMouseCapture`) and `Drop` disables it (`DisableMouseCapture`). Panic hook also restored to disable mouse capture on crash.

## Deviations from Plan

### Auto-fixed Issues

None — plan executed exactly as written.

### Structural Notes

The plan correctly anticipated the `RefCell` approach for `message_queue`. The alternative of adding `AppEvent::Message(WidgetId, Box<dyn Any>)` was evaluated and rejected (noted in plan) because `Box<dyn Any>` is neither `Clone` nor `Debug`, which breaks `#[derive(Debug, Clone)]` on `AppEvent`. The `RefCell<Vec<...>>` on `AppContext` is the cleaner solution and was implemented as specified.

The `drain_message_queue` Tab key handling needed one small adjustment: the Tab case needed to run focus cycling even when `handled = true` (since a focused widget's `on_event` might return Stop on a Tab key but we still want Tab to cycle focus). The implementation uses `if !handled || matches!(k.code, KeyCode::Tab)` for Tab handling, which ensures Tab always reaches focus cycling.

## Known Stubs

None — all event dispatch machinery is fully wired. The `input_buffer` field on `AppContext` remains (Phase 4 cleanup when Input widget replaces it) but is no longer written to in the event loop.

## Self-Check: PASSED

- FOUND: crates/textual-rs/src/event/mod.rs
- FOUND: crates/textual-rs/src/event/message.rs
- FOUND: crates/textual-rs/src/event/dispatch.rs
- FOUND: crates/textual-rs/src/event/keybinding.rs
- FOUND: crates/textual-rs/src/event/timer.rs
- FOUND: commit a936e16 (Task 1)
- FOUND: commit caa4895 (Task 2)
- 99 tests passing (0 failures)
- `cargo build -p textual-rs` exits 0
