---
phase: 05-screen-stack
plan: 03
subsystem: ui
tags: [screens, navigation, modal, push_screen_wait, pop_screen_with, tutorial]

# Dependency graph
requires:
  - phase: 05-02
    provides: push_screen_wait / pop_screen_with API and screen stack with focus save/restore
provides:
  - tutorial_06_screens.rs — end-to-end tutorial demonstrating push navigation, modal dialog, typed result delivery

affects: [phase-06, docs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "push_screen_wait + run_worker + WorkerResult<T> pattern for async modal result bridging"
    - "MainContent owns own_id to bridge push_screen_wait receiver into run_worker"

key-files:
  created:
    - crates/textual-rs/examples/tutorial_06_screens.rs
  modified:
    - crates/textual-rs/Cargo.toml

key-decisions:
  - "Key bindings for push_nav/push_modal placed on MainContent (which owns own_id) not the root ScreenDemoScreen"
  - "NavScreen uses key_bindings (b/Esc) plus Button for go_back — consistent with screen_stack.rs pattern"
  - "Quit handled by the framework's global 'q' handler — no explicit quit action needed in tutorial"
  - "pop_screen_with(bool) delivers typed result; WorkerResult<bool> received in MainContent.on_event"

patterns-established:
  - "Modal result pattern: push_screen_wait -> run_worker(async rx.await) -> WorkerResult<T> in on_event"

requirements-completed: [NAV-01, NAV-02, NAV-03]

# Metrics
duration: 3min
completed: 2026-03-28
---

# Phase 5 Plan 3: Screen Stack Tutorial Summary

**tutorial_06_screens demonstrates push navigation, modal ConfirmDialog with pop_screen_with(bool), and WorkerResult<bool> delivery to the main screen**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-28T07:01:47Z
- **Completed:** 2026-03-28T07:04:51Z
- **Tasks:** 2/2 (Task 1 auto, Task 2 auto-approved human-verify)
- **Files modified:** 2

## Accomplishments

1. Created `crates/textual-rs/examples/tutorial_06_screens.rs` (~300 lines):
   - `ScreenDemoScreen` root screen with Header/MainContent/Footer
   - `MainContent` with `Reactive<String>` result label, key bindings (n/m), `push_screen_wait` + `run_worker` bridge
   - `NavScreen` with `push_screen_deferred` push and b/Esc to `pop_screen_deferred`
   - `NavContent` button group
   - `ConfirmDialog` inner modal widget using `pop_screen_with(true/false)` for typed result delivery
   - CSS: all widget types styled, ModalScreen transparent background, ConfirmDialog centered via margin
2. Added `[[example]]` entry for `tutorial_06_screens` in `crates/textual-rs/Cargo.toml`

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1    | 94a51b7 | feat(05-03): add tutorial_06_screens example with push/pop/modal/result |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed invalid AppEvent::Quit usage from root screen on_action**
- **Found during:** Task 1
- **Issue:** `AppEvent::Quit` doesn't exist in the framework; the app loop handles `q` globally already
- **Fix:** Removed quit action from ScreenDemoScreen entirely; framework's built-in `q` handler suffices
- **Files modified:** crates/textual-rs/examples/tutorial_06_screens.rs
- **Commit:** 94a51b7

**2. [Rule 1 - Design] Moved push_modal key binding to MainContent (which has own_id)**
- **Found during:** Task 1
- **Issue:** Root ScreenDemoScreen has no own_id, making push_screen_wait unusable there
- **Fix:** Key bindings n/m placed on MainContent which owns the own_id needed for run_worker
- **Files modified:** crates/textual-rs/examples/tutorial_06_screens.rs
- **Commit:** 94a51b7

## Verification

```
cargo build --example tutorial_06_screens  → Finished (no errors, 1 warning pre-existing)
```

## Self-Check: PASSED

- [x] crates/textual-rs/examples/tutorial_06_screens.rs exists
- [x] crates/textual-rs/Cargo.toml contains tutorial_06_screens entry
- [x] Commit 94a51b7 exists in git log
