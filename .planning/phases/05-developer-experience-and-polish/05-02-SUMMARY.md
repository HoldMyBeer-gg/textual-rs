---
phase: 05-developer-experience-and-polish
plan: 02
subsystem: worker-api
tags: [worker, async, cancellation, notify, message-passing]
dependency_graph:
  requires: []
  provides: [worker-api, notify-convenience]
  affects: [app-event-loop, widget-context, widget-tree]
tech_stack:
  added: []
  patterns:
    - tokio::task::spawn_local for worker tasks on LocalSet
    - Dedicated flume channel (worker_tx/worker_rx) for worker results
    - tokio::select! integrating app events and worker results
    - AbortHandle tracking in SecondaryMap for auto-cancellation on unmount
key_files:
  created:
    - crates/textual-rs/src/worker.rs
    - crates/textual-rs/tests/worker_tests.rs
  modified:
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/widget/tree.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/lib.rs
decisions:
  - Worker results use a dedicated flume channel (worker_tx/worker_rx) not an AppEvent variant — avoids Clone/Debug requirement on Box<dyn Any + Send>
  - process_deferred_screens() extracted as App helper — called after worker result handling for consistency with key event handling
  - WorkerResult<T> includes source_id field — available to on_event handlers for multi-worker disambiguation
metrics:
  duration: "8min"
  completed_date: "2026-03-26T05:59:37Z"
  tasks_completed: 2
  files_created: 2
  files_modified: 4
---

# Phase 5 Plan 02: Worker API and notify() Summary

## One-Liner

Worker API using spawn_local + dedicated flume channel delivers WorkerResult<T> to widgets, with AbortHandle auto-cancellation on unmount and notify() as a post_message alias.

## What Was Built

### Worker API (DX-02)

`ctx.run_worker(widget_id, async_future)` spawns a task on the Tokio LocalSet. On completion, the result is boxed as `WorkerResult<T>` and sent through a dedicated `worker_tx` flume channel to the App event loop. The event loop uses `tokio::select!` to await both the main `AppEvent` channel and the `worker_rx` channel — when a worker result arrives, it is pushed to the `message_queue` and dispatched via the normal bubbling mechanism.

Workers are automatically cancelled when their owning widget is unmounted: `unmount_widget` calls `ctx.cancel_workers(id)` which aborts all `AbortHandle`s tracked in `worker_handles: RefCell<SecondaryMap<WidgetId, Vec<AbortHandle>>>`.

### notify() Convenience (DX-03)

`ctx.notify(source_id, message)` is a direct alias for `ctx.post_message(source_id, message)`, added for API symmetry with Python Textual. The existing post_message dispatch (bubbling up the parent chain) provides the intended semantic.

### AppEvent Unchanged

The `AppEvent` enum still derives `Clone` and `Debug`. Worker results bypass AppEvent entirely via their own channel.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Worker API implementation | 0bdf157 | worker.rs, context.rs, tree.rs, app.rs, lib.rs |
| 2 | Worker API and notify tests | 06a13ef | tests/worker_tests.rs |

## Tests

6 tests in `worker_tests.rs`, all passing:
- `worker_result_delivered` — WorkerResult arrives on worker_rx channel
- `worker_result_dispatched_via_message_queue` — full pipeline to on_event via dispatch_message
- `worker_cancelled_on_unmount` — AbortHandle aborted, handles map cleared after unmount
- `cancel_workers_targets_correct_widget` — cancellation is per-widget, not global
- `notify_bubbles_to_parent` — notify() posts to message_queue, bubbles up parent chain
- `post_message_to_target` — post_message dispatches to specified target widget

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] Added process_deferred_screens() helper**

- **Found during:** Task 1 — the refactored event loop called `process_deferred_screens()` after worker result handling (for consistency with key event handling), but no such method existed
- **Fix:** Extracted `process_deferred_screens()` as an `App` method that drains `pending_screen_pops` then `pending_screen_pushes`. Called after key events, mouse events, and worker results.
- **Files modified:** `crates/textual-rs/src/app.rs`
- **Commit:** 0bdf157

**2. [Rule 2 - Extra field] Added source_id field to WorkerResult<T>**

- **Found during:** Task 1 — when a widget spawns multiple workers, it needs to know which worker completed. The plan showed only `value` on WorkerResult, but the source_id is already in the message queue tuple. Added it directly on WorkerResult for convenience in on_event handlers.
- **Fix:** Added `pub source_id: WidgetId` field to `WorkerResult<T>`.
- **Files modified:** `crates/textual-rs/src/worker.rs`
- **Commit:** 0bdf157

## Pre-existing Failures (Out of Scope)

20 widget_tests failures were pre-existing before this plan (visible in commit `0d9d292` from focus highlighting work). These are visual rendering assertion failures unrelated to the Worker API. Logged to deferred items.

## Known Stubs

None — all Worker API paths are fully wired.

## Self-Check: PASSED
