---
phase: 01-foundation
plan: 02
subsystem: testing
tags: [ratatui, TestBackend, crossterm, panic-hook, terminal-guard, integration-test]

requires:
  - phase: 01-foundation/01-01
    provides: App, TerminalGuard, init_panic_hook, demo-example, Cargo workspace

provides:
  - App::render_frame<B: Backend> public method for TestBackend testing
  - 5 integration tests covering render pipeline, panic hook, and resize behavior
  - Headless render pipeline verification (TestBackend)

affects: [phase-02-widget-tree, phase-03-reactive]

tech-stack:
  added:
    - ratatui TestBackend (already available via ratatui dep, now exercised in tests)
    - crossterm::terminal::disable_raw_mode (exercised in idempotent drop test)
  patterns:
    - "render_frame<B: Backend> where B::Error: Send + Sync + 'static — generic backend abstraction enabling TestBackend in CI"
    - "Integration tests in crates/textual-rs/tests/ using TestBackend::new(cols, rows) + terminal.draw()"
    - "Collect buffer cells via buffer.content().iter().map(|c| c.symbol()).collect::<String>() for content assertion"

key-files:
  created:
    - crates/textual-rs/tests/integration_test.rs
  modified:
    - crates/textual-rs/src/app.rs

key-decisions:
  - "render_frame requires B::Error: Send + Sync + 'static to satisfy anyhow::Error From conversion — discovered during compilation, not pre-planned"
  - "run_async refactored to use render_frame for both initial render and resize redraw, keeping render path DRY"

patterns-established:
  - "TDD: write failing test, commit RED, implement, verify GREEN, commit feat"
  - "TestBackend integration tests prove render pipeline works without a real terminal — safe for CI"

requirements-completed: [FOUND-05, FOUND-06]

duration: 4min
completed: 2026-03-25
---

# Phase 01 Plan 02: Hardened Terminal Management Summary

**App::render_frame public API with Send+Sync bounds enables TestBackend integration tests that prove the ratatui render pipeline, panic hook installation, and resize re-layout all work headlessly in CI.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-25T06:13:28Z
- **Completed:** 2026-03-25T06:16:41Z
- **Tasks:** 2 auto + 1 auto-approved checkpoint
- **Files modified:** 2

## Accomplishments

- Added `App::render_frame<B: Backend>` public method with correct trait bounds, enabling TestBackend usage
- Refactored `run_async` to call `render_frame` for both initial render and resize redraws (DRY)
- 5 integration tests pass headlessly: render content, border title, panic hook, Drop safety, resize layout

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Failing TestBackend tests** - `242509f` (test)
2. **Task 1 GREEN: App::render_frame implementation** - `0804a9d` (feat)
3. **Task 2: Panic hook and resize tests** - `23762bf` (feat)

**Plan metadata:** (docs commit below)

_Note: TDD tasks have multiple commits (test RED → feat GREEN)_

## Files Created/Modified

- `crates/textual-rs/tests/integration_test.rs` - 5 integration tests: test_render_hello, test_render_has_title, test_panic_hook_is_installed, test_terminal_guard_drop_is_idempotent, test_render_at_different_sizes
- `crates/textual-rs/src/app.rs` - Added `render_frame<B: Backend>` method; refactored run_async to use it

## Decisions Made

- `render_frame` needs `where B::Error: Send + Sync + 'static` to satisfy `anyhow::Error`'s `From` impl. This is a Rust trait bound constraint, not a design decision — anyhow requires `std::error::Error + Send + Sync + 'static` for conversion.
- Refactored `run_async` to use `render_frame` instead of inlining `terminal.draw(|f| Self::render(f))` — keeps the render path in one place.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `B::Error: Send + Sync + 'static` where clause to render_frame**
- **Found during:** Task 1 GREEN phase (compilation)
- **Issue:** `anyhow::Result<()>` requires the error type to implement `Send + Sync + 'static`. The plan's suggested signature omitted this bound, causing compile error E0277.
- **Fix:** Added `where B::Error: Send + Sync + 'static` to the `render_frame` signature.
- **Files modified:** crates/textual-rs/src/app.rs
- **Verification:** `cargo test -p textual-rs` exits 0, all 2 tests pass in GREEN phase.
- **Committed in:** 0804a9d (Task 1 feat commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - compiler type bound)
**Impact on plan:** Required for correctness. The plan's suggested signature was missing a necessary trait constraint; the fix is minimal and idiomatic Rust.

## Issues Encountered

None beyond the trait bound fix above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 1 foundation complete: workspace, event loop, terminal guard, panic hook, demo, and headless test coverage
- Phase 2 can attach the widget tree to `App` — the `render_frame` API provides the test harness entry point
- Phase 3 can build on `render_frame` for `TestApp`/`Pilot` snapshot testing
- Blocker noted in STATE.md: SlotMap borrow ergonomics spike required before Phase 2 planning

## Known Stubs

None — all functionality is fully wired and tested.

## Self-Check: PASSED

Files verified present:
- crates/textual-rs/tests/integration_test.rs: FOUND
- crates/textual-rs/src/app.rs: FOUND

Commits verified:
- 242509f: FOUND (test(01-02): add failing TestBackend integration tests for render_frame)
- 0804a9d: FOUND (feat(01-02): add App::render_frame public API for TestBackend testing)
- 23762bf: FOUND (feat(01-02): add panic hook and resize integration tests)

---
*Phase: 01-foundation*
*Completed: 2026-03-25*
