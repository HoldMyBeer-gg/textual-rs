---
phase: 09-complex-widgets
plan: 01
subsystem: ui
tags: [ratatui, widget, mask, input, cursor, tdd]

# Dependency graph
requires: []
provides:
  - MaskedInput widget with mask enforcement (##/##/#### date format and variants)
  - Cursor tracking in raw-value space with O(1) display column lookup
  - Case modifier support (> uppercase, < lowercase) per mask slot
  - messages::Changed and messages::Submitted for widget-to-widget communication
affects: [phase-09-complex-widgets, examples, demos]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - raw-value cursor space: cursor tracked as index into user chars only; separators skipped automatically at render
    - slot_display_cols lookup table: precomputed at ::new() for O(1) raw-pos-to-display-col mapping
    - MaskSlot enum with accepts_char per slot type and case transform via slot_case vec

key-files:
  created:
    - crates/textual-rs/src/widget/masked_input.rs
  modified:
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/app.rs

key-decisions:
  - "MaskedInput cursor tracked in raw-value space only; display cursor derived per render via precomputed slot_display_cols table (O(1) lookup, no separator-skip logic in cursor movement)"
  - "Case modifiers (>/< in mask string) are not slots — they set current_case mode for subsequent input slots during parse"
  - "raw_value stores only user-typed chars; separators exist only in display string built at render time"
  - "BUILTIN_CSS entry matches Input: border rounded, height 3 for visual consistency"

patterns-established:
  - "Raw-space cursor: MaskedInput pattern where cursor index refers to raw user input only, display col derived via lookup"
  - "Precomputed display cols: slot_display_cols built once in ::new() avoids per-render O(n) scan"

requirements-completed: [WIDGET-11]

# Metrics
duration: 15min
completed: 2026-03-27
---

# Phase 09 Plan 01: MaskedInput Summary

**Format-constrained text input with mask enforcement (##/##/####), raw-space cursor tracking, case modifiers, and 15 passing unit tests**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-03-27T21:05:45Z
- **Completed:** 2026-03-27T21:20:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- MaskedInput widget implementing full mask enforcement for dates, phone numbers, and custom patterns
- Cursor tracked in raw-value space with precomputed O(1) display column lookup table
- Case transform modifiers (`>` uppercase, `<` lowercase) applied per slot during parse
- 15 unit tests covering all specified behaviors; no regressions (245 lib tests pass)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement MaskedInput widget** - `fc3b855` (feat)
2. **Task 2: Register module and add BUILTIN_CSS** - `d1fa1df` (feat)

**Plan metadata:** (this commit)

## Files Created/Modified
- `crates/textual-rs/src/widget/masked_input.rs` - Full MaskedInput widget: MaskSlot enum, parse_mask, build_display, raw_pos_to_display_col, Widget trait impl, 15 unit tests
- `crates/textual-rs/src/widget/mod.rs` - Added `pub mod masked_input;` (alphabetical order after markdown)
- `crates/textual-rs/src/app.rs` - Added `MaskedInput { border: rounded; height: 3; }` to BUILTIN_CSS

## Decisions Made
- Cursor tracked in raw-value space (not display space) — separator skipping is automatic since separators are never in raw_value
- `slot_display_cols: Vec<usize>` precomputed once at `::new()` via `parse_mask()` — avoids O(n) scan per render or cursor movement
- Case modifiers (`>`, `<`) consumed during mask parsing, not stored as slots — they only influence `slot_case` vec entries
- Module registered in alphabetical position (after `markdown`, before `placeholder`) following codebase convention

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - implementation compiled and all 15 tests passed on first run.

## Next Phase Readiness
- MaskedInput is fully usable as a drop-in widget in any textual-rs application
- No stubs — all mask slot types, cursor movements, backspace, case transforms, and messages are fully implemented
- Phase 09 continues with 09-02 (DirectoryTree) and 09-03 (Toast) in parallel

---
*Phase: 09-complex-widgets*
*Completed: 2026-03-27*
