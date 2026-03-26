---
phase: 04-built-in-widget-library
plan: 08
subsystem: ui
tags: [ratatui, input-widget, validation, tdd]

requires:
  - phase: 04-built-in-widget-library
    provides: Input widget with placeholder, password mode, cursor navigation, and Changed/Submitted messages

provides:
  - Input widget validator callback via with_validator() builder method
  - is_valid() query method on Input
  - valid: bool field on messages::Changed
  - Red-foreground rendering when input fails validation
  - 4 passing tests covering validation scenarios

affects:
  - Any plan using Input widget (Changed message now has valid field)
  - 04-VERIFICATION.md Gap 1 closure

tech-stack:
  added: []
  patterns:
    - "Cell<bool> for interior-mutable validation state in Widget (no &mut required)"
    - "Builder method with_validator() takes impl Fn(&str) -> bool + 'static, boxed into Option"
    - "run_validation() called from emit_changed() to keep state fresh before message emit"
    - "inject_key_event in tests to leave messages in queue for inspection (vs type_text which drains)"

key-files:
  created: []
  modified:
    - crates/textual-rs/src/widget/input.rs
    - crates/textual-rs/tests/widget_tests.rs

key-decisions:
  - "Used Cell<bool> for valid state — consistent with existing Cell<usize> cursor_pos pattern, avoids &mut in on_event/render"
  - "run_validation() called inside emit_changed() — single call site ensures state is always current before message emission"
  - "Tests use inject_key_event() for character typing — pilot.type_text() drains message queue via settle(), leaving nothing to inspect"

patterns-established:
  - "Validation state pattern: Cell<bool> + Option<Box<dyn Fn>> + run_validation() called from emit_changed"

requirements-completed: [WIDGET-03]

duration: 10min
completed: 2026-03-25
---

# Phase 04 Plan 08: Input Validation Summary

**Input widget with optional validator callback, valid: bool in Changed messages, and red-foreground error rendering — 4 new TDD tests, 97 total tests passing**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-03-25T00:00:00Z
- **Completed:** 2026-03-25T00:10:00Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Added `validator: Option<Box<dyn Fn(&str) -> bool>>` and `valid: Cell<bool>` fields to `Input` struct
- Added `with_validator()` builder method and `is_valid()` query method
- Updated `messages::Changed` to include `pub valid: bool`
- `emit_changed()` now runs validation and includes validity in each Change message
- `render()` uses `Color::Red` foreground for text and cursor when input is invalid and non-empty
- 4 tests added covering valid input, invalid input, no-validator default, and Changed message valid field

## Task Commits

1. **Task 1: Add validator field, validation logic, and error state rendering** - `080368e` (feat)

**Plan metadata:** _(docs commit follows)_

## Files Created/Modified

- `crates/textual-rs/src/widget/input.rs` — Added validator/valid fields, with_validator(), is_valid(), run_validation(), updated emit_changed and render
- `crates/textual-rs/tests/widget_tests.rs` — Added 4 validation tests plus inject_char() helper function

## Decisions Made

- Used `Cell<bool>` for validity state — matches existing `Cell<usize>` cursor pattern, avoids &mut in event handlers
- `run_validation()` called inside `emit_changed()` — ensures validity is always fresh before emitting
- Tests use `inject_key_event()` for character typing instead of `pilot.type_text()`, because `type_text` calls `settle()` which drains the message queue, leaving nothing to inspect

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Initial test implementation used `pilot.type_text()` which calls `settle()` and drains the message queue before assertions. Switched to `inject_key_event()` (same pattern as `input_submit_emits_message` test) to leave messages in queue for inspection. No code change needed — only test approach correction.

## Known Stubs

None — validation is fully wired.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Input validation is complete and WIDGET-03 requirement is satisfied
- Gap 1 from 04-VERIFICATION.md is closed
- All 97 tests pass with no regressions

---
*Phase: 04-built-in-widget-library*
*Completed: 2026-03-25*
