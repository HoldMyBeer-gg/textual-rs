---
phase: 04-production-readiness
plan: 04
subsystem: animation, css, worker, scroll
tags: [animation, tween, text-align, horizontal-scroll, worker-progress, css-hot-reload]
dependency_graph:
  requires: [04-03]
  provides: [animation-system, text-align-rendering, horizontal-scroll, worker-progress, css-hot-reload]
  affects: [widget-switch, widget-tabs, widget-label, widget-button, widget-header, app-event-loop]
tech_stack:
  added: []
  patterns: [tween-animation, easing-functions, css-file-polling, progress-channel]
key_files:
  created:
    - crates/textual-rs/src/animation.rs
  modified:
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/css/render_style.rs
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/src/testing/mod.rs
    - crates/textual-rs/src/widget/button.rs
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/widget/header.rs
    - crates/textual-rs/src/widget/label.rs
    - crates/textual-rs/src/widget/switch.rs
    - crates/textual-rs/src/widget/tabs.rs
    - crates/textual-rs/src/worker.rs
    - crates/textual-rs/tests/worker_tests.rs
decisions:
  - "Tween uses wall-clock time (Instant::now) with skip_animations flag for deterministic tests"
  - "CSS hot-reload uses simple 2-second polling instead of notify crate to avoid new dependency"
  - "Worker progress uses flume channel forwarded via spawn_local to message queue"
  - "Button defaults to text-align: center via BUILTIN_CSS; bare test snapshot updated"
metrics:
  duration: "17m 44s"
  completed: "2026-03-26"
  tasks: 2
  files: 14
---

# Phase 04 Plan 04: Animation, Text-Align, Horizontal Scroll, Worker Progress, CSS Hot-Reload Summary

Tween animation system with easing functions; text-align CSS property wired to Label/Button/Header rendering; horizontal mouse wheel scroll; worker progress channel; CSS file hot-reload via polling.

## Task Completion

| Task | Name | Commit | Status |
|------|------|--------|--------|
| 1 | Animation system + text-align center + horizontal scroll | aee5333 | Done |
| 2 | Worker progress reporting and CSS hot-reload | 7e4c98a | Done |

## What Was Built

### Animation System (animation.rs)
- `Tween` struct: interpolates from/to values over a Duration using an easing function
- Three built-in easing functions: `linear`, `ease_in_out_cubic`, `ease_out_cubic`
- `EasingFn` type alias for `fn(f64) -> f64`
- Tween tracks start time and computes current value based on elapsed wall-clock time
- `is_complete()` and `target()` accessors for animation lifecycle

### Switch Animation
- Switch knob position now animated via Tween (200ms ease-in-out-cubic) on toggle
- Colors interpolated between on/off states based on knob fraction
- `skip_animations` flag on AppContext snaps to target for deterministic tests

### Tabs Underline Animation
- Tabs widget computes per-tab x-offset for underline position
- On tab change, a Tween (200ms ease-out-cubic) slides from old to new tab position
- `underline_tween` field on Tabs struct stores active animation

### text-align CSS Property
- `align_text(text, width, TextAlign)` helper in render_style.rs
- Label reads text-align from computed style, defaults to Left
- Button reads text-align from computed style, defaults to Center
- Header already centered manually (unchanged)
- BUILTIN_CSS updated: `Button { text-align: center; }`

### Horizontal Mouse Scroll
- App event loop handles `MouseEventKind::ScrollLeft` and `MouseEventKind::ScrollRight`
- Maps to `scroll_left` / `scroll_right` actions dispatched through hit_map
- ScrollView already handles these actions via existing key bindings
- Both the async event loop and `handle_mouse_event` method updated

### Worker Progress Reporting
- `WorkerProgress<T>` message type in worker.rs (mirrors WorkerResult)
- `run_worker_with_progress` on AppContext: accepts closure receiving flume::Sender<P>
- Progress messages forwarded via spawn_local to the message queue as WorkerProgress<P>
- Final result still delivered as WorkerResult<T>
- Re-exported as `textual_rs::WorkerProgress`

### CSS Hot-Reload
- `with_css_file(path)` builder on App: reads, parses, and registers a .tcss file
- `WatchedCss` struct tracks path, last_modified time, and stylesheet index
- Event loop polls watched files every 2 seconds via `tokio::time::interval`
- On change: re-read, re-parse, replace stylesheet at stored index, trigger full re-cascade
- Poll branch uses `if !self.watched_css.is_empty()` guard to skip when no files watched

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Button snapshot test broke due to text-align defaulting to Left**
- **Found during:** Task 1
- **Issue:** Adding text-align CSS support to Button caused bare-test (no CSS) to show left-aligned label instead of centered
- **Fix:** Added `text-align: center` to Button's BUILTIN_CSS; updated bare-test snapshot to reflect correct no-CSS behavior
- **Files modified:** app.rs, widget_tests__snapshot_button_default.snap

**2. [Rule 1 - Bug] Switch animation test failures**
- **Found during:** Task 1
- **Issue:** Switch tween animation starts at 0.0 and takes 200ms to reach 1.0, but tests render immediately after toggle, showing intermediate position
- **Fix:** Added `skip_animations: bool` field to AppContext, set true by TestApp. Tween snaps to target when skip_animations is true.
- **Files modified:** widget/context.rs, widget/switch.rs, testing/mod.rs, app.rs

## Decisions Made

1. **Wall-clock tweening with test override** -- Tween uses `Instant::now()` for real-time animation. Tests use `skip_animations` flag rather than mock clocks, keeping the animation API simple while ensuring deterministic tests.

2. **Polling CSS watcher** -- Used 2-second `tokio::time::interval` polling instead of the `notify` crate. Zero new dependencies. 2-second latency is acceptable for development workflow.

3. **Progress channel pattern** -- Worker progress uses flume unbounded channel with a forwarding spawn_local task. This keeps the API ergonomic (closure receives Sender) while integrating with the existing message queue dispatch.

## Known Stubs

None -- all features are fully wired.

## Test Results

All 391 tests pass (0 failures, 6 doc-test ignores).

## Self-Check: PASSED
