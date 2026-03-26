---
phase: 04-built-in-widget-library
plan: 01
subsystem: ui
tags: [rust, ratatui, widget, reactive, tui, label, button, checkbox, switch, insta, snapshots]

# Dependency graph
requires:
  - phase: 03-reactive-system-events-and-testing
    provides: Reactive<T>, Message trait, TestApp/Pilot harness, key binding dispatch, snapshot testing

provides:
  - Label widget (static text, no focus, render to buffer)
  - Button widget (focusable, ButtonVariant enum, Pressed message, Enter/Space bindings)
  - Checkbox widget (Reactive<bool> checked state, Changed message, Space/Enter toggle)
  - Switch widget (Reactive<bool> value, Changed message, on=━━━◉ off=◉━━━ indicator)
  - AppContext.pending_screen_pushes and push_screen_deferred() for deferred screen pushes
  - TestApp.inject_key_event and drain_messages for low-level test control
  - 16 widget tests (4 snapshot + 12 render/interaction) all passing

affects:
  - 04-built-in-widget-library (all subsequent plans use the same widget implementation pattern)

# Tech tracking
tech-stack:
  added:
    - pulldown-cmark 0.13 (for future Markdown widget — added now per plan)
    - arboard 3.6 (for clipboard access in future TextArea — added now per plan)
  patterns:
    - Widget struct + impl Widget with default_css, render, on_mount, on_unmount, key_bindings, on_action
    - Reactive<bool> for interactive widget state with get_untracked() in render()
    - Cell<Option<WidgetId>> set in on_mount for own_id to enable post_message from on_action
    - Static &[KeyBinding] slices for zero-allocation key binding declarations
    - pub mod messages { pub struct Foo; impl Message for Foo {} } inside widget module
    - TestApp.inject_key_event for pre-drain message queue inspection in tests

key-files:
  created:
    - crates/textual-rs/src/widget/label.rs
    - crates/textual-rs/src/widget/button.rs
    - crates/textual-rs/src/widget/checkbox.rs
    - crates/textual-rs/src/widget/switch.rs
    - crates/textual-rs/tests/widget_tests.rs
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_label_default.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_button_default.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_checkbox_checked.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_switch_on.snap
  modified:
    - crates/textual-rs/Cargo.toml (pulldown-cmark, arboard deps)
    - crates/textual-rs/src/widget/mod.rs (pub mod label/button/checkbox/switch)
    - crates/textual-rs/src/widget/context.rs (pending_screen_pushes, push_screen_deferred)
    - crates/textual-rs/src/lib.rs (pub use Label, Button, ButtonVariant, Checkbox, Switch)
    - crates/textual-rs/src/testing/mod.rs (inject_key_event, drain_messages)

key-decisions:
  - "Use get_untracked() in all render() methods — avoids reactive tracking loops since render runs outside reactive owner context"
  - "Cell<Option<WidgetId>> set in on_mount enables post_message from on_action(&self) without &mut"
  - "Static &[KeyBinding] slices (const-sized) avoid allocation and satisfy lifetime requirements for key_bindings() -> &[KeyBinding]"
  - "TestApp.inject_key_event added (deviation Rule 2) — required for button press verification without accessing private app field"
  - "pending_screen_pushes on AppContext (RefCell) — deferred screen push from on_action(&self) for future Select widget overlay pattern"

patterns-established:
  - "Widget pattern: struct fields → impl Widget trait → default_css() → on_mount sets own_id → key_bindings static slice → on_action posts message → render uses get_untracked()"
  - "Message pattern: pub mod messages { pub struct Changed { pub field: T } impl Message for Changed {} }"
  - "Test pattern: TestApp::new(cols, rows, factory) → pilot.press(key).await → assert_buffer_lines or snapshot"
  - "Interaction verification: inject_key_event before drain to inspect raw message queue"

requirements-completed: [WIDGET-01, WIDGET-02, WIDGET-05, WIDGET-06]

# Metrics
duration: 5min
completed: 2026-03-26
---

# Phase 4 Plan 1: Label, Button, Checkbox, Switch widgets with snapshot and interaction tests

**Four reactive widgets (Label, Button, Checkbox, Switch) with typed message dispatch, static key bindings, and 16 passing tests establishing the widget implementation pattern for all subsequent plans.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-26T00:32:07Z
- **Completed:** 2026-03-26T00:37:00Z
- **Tasks:** 2
- **Files modified:** 14 (5 created, 9 modified)

## Accomplishments

- Label, Button, Checkbox, Switch widgets implemented following the established pattern
- AppContext extended with `pending_screen_pushes` and `push_screen_deferred()` for the future Select widget overlay
- 16 tests: 4 insta snapshots + 8 render assertions + 4 keyboard interaction tests — all pass
- TestApp extended with `inject_key_event` / `drain_messages` for precise message queue inspection
- pulldown-cmark and arboard dependencies added for future Markdown/TextArea widgets
- Zero regressions in 99 existing lib tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Scaffold widget module infrastructure** - `89038c8` (feat)
2. **Task 2: Widget tests with snapshot and interaction** - `1fe0e8b` (test)

## Files Created/Modified

- `crates/textual-rs/src/widget/label.rs` — Label: static text, no focus, truncated render
- `crates/textual-rs/src/widget/button.rs` — Button: ButtonVariant, Pressed message, Enter/Space bindings
- `crates/textual-rs/src/widget/checkbox.rs` — Checkbox: Reactive<bool> checked, Changed message, [X]/[ ] render
- `crates/textual-rs/src/widget/switch.rs` — Switch: Reactive<bool> value, Changed message, ━━━◉/◉━━━ render
- `crates/textual-rs/tests/widget_tests.rs` — 16 tests: snapshots, render assertions, keyboard interactions
- `crates/textual-rs/src/widget/context.rs` — Added pending_screen_pushes, push_screen_deferred
- `crates/textual-rs/src/testing/mod.rs` — Added inject_key_event, drain_messages
- `crates/textual-rs/src/lib.rs` — Re-exports Label, Button, ButtonVariant, Checkbox, Switch
- `crates/textual-rs/src/widget/mod.rs` — pub mod declarations for all four widgets
- `crates/textual-rs/Cargo.toml` — pulldown-cmark 0.13, arboard 3.6

## Decisions Made

- Used `Cell<Option<WidgetId>>` (not `RefCell`) for `own_id` — only needs set/get, not interior mutation of complex type; simpler than RefCell
- Static `&[KeyBinding]` slices for zero-allocation key binding declarations matching the `key_bindings() -> &[KeyBinding]` signature
- `pending_screen_pushes` as `RefCell<Vec<Box<dyn Widget>>>` on AppContext — enables deferred screen push from `on_action(&self)` without needing `&mut AppContext`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added inject_key_event and drain_messages to TestApp**
- **Found during:** Task 2 (button_press_enter_emits_pressed_message test)
- **Issue:** `TestApp.app` field is `pub(crate)` — not accessible from integration tests. The test needed to call `handle_key_event` without the auto-drain that `process_event` performs, to inspect the raw message queue.
- **Fix:** Added `pub fn inject_key_event` and `pub fn drain_messages` to `TestApp` — minimal public API for low-level test control.
- **Files modified:** `crates/textual-rs/src/testing/mod.rs`
- **Verification:** `button_press_enter_emits_pressed_message` test passes, verifying ButtonPressed in queue
- **Committed in:** `1fe0e8b` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2 - missing critical test infrastructure)
**Impact on plan:** The fix is strictly additive (new public methods on TestApp). No plan scope creep.

## Issues Encountered

- Insta snapshot tests require `INSTA_UPDATE=always` env var on first run (no `cargo insta` binary installed) — resolved by re-running tests with the env var set.

## Next Phase Readiness

- Widget implementation pattern validated end-to-end: struct, Widget trait, default_css, reactive state, messages, key bindings, tests
- Plan 04-02 can proceed with Input and TextArea widgets using the same pattern
- `pending_screen_pushes` and `push_screen_deferred()` ready for Select widget overlay (Plan 04-04)

---
*Phase: 04-built-in-widget-library*
*Completed: 2026-03-26*
