---
phase: 05-screen-stack
plan: 02
subsystem: navigation
tags: [screen-stack, modal, async, oneshot, tokio-sync]

dependency_graph:
  requires:
    - phase: 05-screen-stack/01
      provides: push_screen_deferred/pop_screen_deferred, process_deferred_screens, screen_stack tests
  provides:
    - push_screen_wait() returning oneshot Receiver for typed modal results
    - pop_screen_with<T>() delivering result through oneshot on pop
    - process_deferred_screens result delivery pipeline
  affects: [tutorials, examples using screen stack with typed results]

tech_stack:
  added:
    - tokio sync feature (for oneshot channel)
  patterns:
    - "push_screen_wait returns Receiver<Box<dyn Any + Send>>; caller downcasts after await"
    - "pending_pop_result single-slot: one result per event cycle (pop_screen_with always pops top)"
    - "screen_result_senders HashMap<WidgetId, Sender> keyed by pushed screen's WidgetId"

key_files:
  created: []
  modified:
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/tests/screen_stack.rs

key_decisions:
  - "Single-slot pending_pop_result (not HashMap): pop_screen_with always pops the TOP screen so at most one result per event cycle"
  - "tokio sync feature added to Cargo.toml for oneshot channel; flume was considered but tokio oneshot maps better to the push_screen_wait API shape"
  - "pop_screen_with on non-wait screen silently discards result — no panic, clean no-op"

requirements-completed: [NAV-01, NAV-02, NAV-03]

duration: 4min
completed: "2026-03-28"
---

# Phase 05 Plan 02: Screen Stack — push_screen_wait / pop_screen_with

**Typed async modal result API: push_screen_wait() returns a oneshot Receiver; pop_screen_with(value) delivers the typed result when the modal dismisses**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-28T07:09:26Z
- **Completed:** 2026-03-28T07:13:43Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- `AppContext::push_screen_wait()` — pushes a modal via the deferred mechanism and returns a `tokio::sync::oneshot::Receiver<Box<dyn Any + Send>>` for typed result delivery
- `AppContext::pop_screen_with<T>()` — stores the result in `pending_pop_result` and triggers a deferred pop; `process_deferred_screens` completes the oneshot after pop
- 3 new integration tests covering OK path, Cancel path, and no-wait no-panic safety

## Task Commits

1. **Task 1: push_screen_wait / pop_screen_with API and event loop integration** - `75c908d` (feat)
2. **Task 2: Integration tests for push_screen_wait / pop_screen_with** - `713bc0e` (test)

## Files Created/Modified

- `crates/textual-rs/src/widget/context.rs` — Added 3 new fields (`pending_screen_wait_pushes`, `screen_result_senders`, `pending_pop_result`), `push_screen_wait()` and `pop_screen_with()` methods; added `std::collections::HashMap` import
- `crates/textual-rs/src/app.rs` — Modified `process_deferred_screens` to deliver results through oneshot after pop; added wait-push processing with sender registration
- `crates/textual-rs/Cargo.toml` — Added `sync` to tokio features
- `crates/textual-rs/tests/screen_stack.rs` — Added `ResultDialog` helper widget and 3 new tests

## Decisions Made

- **Single-slot `pending_pop_result`**: Since `pop_screen_with` always acts on the top screen, at most one result can be pending per event cycle. A `RefCell<Option<Box<dyn Any + Send>>>` is sufficient.
- **tokio sync feature**: Added `sync` to tokio features for `oneshot` channel. Flume was considered but the tokio oneshot matches the `push_screen_wait` API shape (caller `.await`s the Receiver directly).
- **Silent discard on no-wait pop**: If `pop_screen_with` fires on a screen that wasn't pushed via `push_screen_wait`, the result is silently dropped and the pop completes normally — no error, no panic.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tokio sync feature missing**
- **Found during:** Task 1 (initial compile)
- **Issue:** `tokio::sync::oneshot` requires the `sync` feature which was not in `Cargo.toml`
- **Fix:** Added `"sync"` to tokio features in `crates/textual-rs/Cargo.toml`
- **Files modified:** `crates/textual-rs/Cargo.toml`
- **Verification:** `cargo build` succeeds after addition
- **Committed in:** 75c908d (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (blocking dependency feature)
**Impact on plan:** Necessary fix, no scope change.

## Issues Encountered

None — implementation followed plan cleanly once the tokio sync feature was added.

## Known Stubs

None — `push_screen_wait` and `pop_screen_with` are fully wired and tested end-to-end.

## Next Phase Readiness

- All three NAV requirements (NAV-01, NAV-02, NAV-03) are now fully implemented and tested
- `push_screen_wait` API ready for use in tutorials and user code
- Plan 05-03 (tutorial_06_screens demo) can proceed immediately

## Self-Check: PASSED

- FOUND: .planning/phases/05-screen-stack/05-02-SUMMARY.md
- FOUND: crates/textual-rs/src/widget/context.rs (modified)
- FOUND: crates/textual-rs/src/app.rs (modified)
- FOUND: crates/textual-rs/tests/screen_stack.rs (modified)
- FOUND: commit 75c908d (Task 1)
- FOUND: commit 713bc0e (Task 2)

---
*Phase: 05-screen-stack*
*Completed: 2026-03-28*
