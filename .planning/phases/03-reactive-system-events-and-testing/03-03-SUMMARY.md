---
phase: 03-reactive-system-events-and-testing
plan: "03"
subsystem: testing
tags: [ratatui, testbackend, insta, proptest, async, tokio, headless-testing]

# Dependency graph
requires:
  - phase: 03-01
    provides: reactive signals, AppContext with event_tx
  - phase: 03-02
    provides: App event dispatch, key/mouse handlers, drain_message_queue

provides:
  - TestApp: headless App wrapper with TestBackend for automated testing
  - Pilot: async press/type_text/click/settle input simulation
  - assert_buffer_lines / assert_cell: row-level buffer assertion helpers
  - Integration tests (9 async) proving TestApp+Pilot end-to-end
  - Insta snapshot baselines for GreetingWidget, EmptyWidget, multi-size
  - Proptest CSS fuzzing (5 property-based tests)

affects:
  - 04-built-in-widgets (uses TestApp+Pilot+snapshots for all widget tests)

# Tech tracking
tech-stack:
  added:
    - "insta 1.46.3 (snapshot testing, already in dev-deps)"
    - "proptest 1.11.0 (property-based testing, already in dev-deps)"
    - "tokio rt-multi-thread feature (dev-dep, needed for #[tokio::test])"
  patterns:
    - "TestApp::new(cols, rows, factory) for headless widget testing"
    - "pilot.press(KeyCode).await / pilot.settle().await test pattern"
    - "assert_snapshot!(format!({}, backend())) for visual regression"
    - "proptest! macro for parser fuzz testing"

key-files:
  created:
    - crates/textual-rs/src/testing/mod.rs
    - crates/textual-rs/src/testing/pilot.rs
    - crates/textual-rs/src/testing/assertions.rs
    - crates/textual-rs/tests/test_harness.rs
    - crates/textual-rs/tests/snapshot_tests.rs
    - crates/textual-rs/tests/proptest_css.rs
    - crates/textual-rs/tests/snapshots/ (4 .snap files)
  modified:
    - crates/textual-rs/src/app.rs (set_event_tx, mount_root_screen, render_to_terminal, handle_key_event, handle_mouse_event, drain_message_queue now pub)
    - crates/textual-rs/src/lib.rs (pub mod testing, re-exports TestApp + Pilot)
    - crates/textual-rs/Cargo.toml (tokio rt-multi-thread in dev-deps)

key-decisions:
  - "TestApp processes events synchronously via process_event — no async event loop, tests control timing precisely"
  - "settle() uses yield_now loop (max 100) to drain reactive effects then checks empty queues for quiescence"
  - "Doc examples use ignore (not no_run) to avoid compile failures referencing undefined identifiers"
  - "Snapshot format uses format!(\"{}\", backend()) — TestBackend implements Display as grid of cells"

patterns-established:
  - "TestApp::new + pilot().press().await + assert_buffer_lines: standard widget test pattern"
  - "proptest! for parser/parser-adjacent code; assert_snapshot! for rendered output regression"

requirements-completed: [TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06]

# Metrics
duration: 5min
completed: 2026-03-25
---

# Phase 03 Plan 03: Testing Infrastructure Summary

**TestApp headless harness with TestBackend, async Pilot input simulation, insta snapshot baselines, and proptest CSS fuzzer — complete test foundation for Phase 4 widget development**

## Performance

- **Duration:** 5min
- **Started:** 2026-03-25T23:39:33Z
- **Completed:** 2026-03-25T23:45:25Z
- **Tasks:** 2
- **Files modified:** 13 (6 created in src/testing, 3 integration test files, 4 snapshot files, 3 modified)

## Accomplishments

- TestApp wraps App with TestBackend — headless testing without a real terminal, initial render on construction
- Pilot provides async press/type_text/click/settle methods that simulate user input and drain to quiescence
- assert_buffer_lines and assert_cell helpers for row-level buffer assertions
- 9 async integration tests prove TestApp+Pilot work end-to-end (focus cycling, buffer dimensions, settle determinism, type_text, click)
- Insta snapshot tests with 4 committed baseline .snap files
- Proptest CSS fuzzing confirms parser never panics on arbitrary/structured/empty input; valid RGB colors parse without errors

## Task Commits

1. **Task 1: Create testing module — TestApp, Pilot, settle(), assertions** - `0c96481` (feat)
2. **Task 2: Integration tests — TestApp harness, insta snapshots, proptest CSS** - `105b443` (feat)

## Files Created/Modified

- `crates/textual-rs/src/testing/mod.rs` — TestApp struct with new/pilot/ctx/buffer/backend/process_event
- `crates/textual-rs/src/testing/pilot.rs` — Pilot with async press/press_with_modifiers/type_text/click/settle
- `crates/textual-rs/src/testing/assertions.rs` — assert_buffer_lines, assert_cell helpers
- `crates/textual-rs/src/app.rs` — Added set_event_tx, mount_root_screen, render_to_terminal, handle_key_event, handle_mouse_event; drain_message_queue made pub
- `crates/textual-rs/src/lib.rs` — Added pub mod testing, re-exports TestApp and Pilot
- `crates/textual-rs/Cargo.toml` — tokio rt-multi-thread feature in dev-dependencies
- `crates/textual-rs/tests/test_harness.rs` — 9 async integration tests
- `crates/textual-rs/tests/snapshot_tests.rs` — 3 insta snapshot tests
- `crates/textual-rs/tests/proptest_css.rs` — 5 property-based CSS parser tests
- `crates/textual-rs/tests/snapshots/*.snap` — 4 baseline snapshot files

## Decisions Made

- **Sync process_event pattern**: TestApp processes events synchronously rather than running the async event loop. This gives tests precise control over timing and avoids flakiness.
- **settle() design**: Uses `tokio::task::yield_now().await` in a bounded loop (max 100 iterations) to drain reactive effects, then checks empty rx channel and message queue for quiescence. Simple and effective.
- **Doc examples use `ignore`**: The `no_run` attribute still attempts to compile the example, which fails for snippets referencing undefined types. Switched to `ignore` to skip compilation entirely.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Doc examples used `no_run` instead of `ignore`**
- **Found during:** Task 2 (full test suite run)
- **Issue:** `no_run` examples in testing module still compiled and failed due to references to `MyScreen` / `test_app` (undefined in doc context)
- **Fix:** Changed `no_run` to `ignore` in testing/mod.rs, testing/pilot.rs, testing/assertions.rs
- **Files modified:** crates/textual-rs/src/testing/mod.rs, pilot.rs, assertions.rs
- **Verification:** `cargo test -p textual-rs` shows doctests as `ignored` not `FAILED`
- **Committed in:** `105b443` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug in doc example annotations)
**Impact on plan:** Minor fix, no scope change. All planned artifacts delivered as specified.

## Issues Encountered

- Insta produced `.snap.new` files on first run — resolved by running with `INSTA_UPDATE=always INSTA_FORCE_UPDATE_SNAPSHOTS=1` to merge into `.snap` files (no `cargo-insta` CLI installed)

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- TestApp + Pilot + assert_buffer_lines + assert_snapshot! are ready for Phase 4 widget development
- Every Phase 4 widget test can follow: `TestApp::new(cols, rows, factory)` → `pilot.press(Key)` → `assert_buffer_lines()`
- No blockers — full test suite (99 unit + 9 integration + 5 proptest + 3 snapshot) passes

## Self-Check: PASSED

All required files confirmed present:
- crates/textual-rs/src/testing/mod.rs — FOUND
- crates/textual-rs/src/testing/pilot.rs — FOUND
- crates/textual-rs/src/testing/assertions.rs — FOUND
- crates/textual-rs/tests/test_harness.rs — FOUND
- crates/textual-rs/tests/snapshot_tests.rs — FOUND
- crates/textual-rs/tests/proptest_css.rs — FOUND

All commits confirmed present:
- 0c96481 — FOUND
- 105b443 — FOUND

---
*Phase: 03-reactive-system-events-and-testing*
*Completed: 2026-03-25*
