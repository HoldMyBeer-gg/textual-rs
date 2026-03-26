---
phase: 04-production-readiness
plan: 02
subsystem: overlays, css-parser, canvas
tags: [bugfix, overlay, css-variables, unicode-fallback]
dependency_graph:
  requires: []
  provides: [active-overlay-select, active-overlay-palette, border-variable-parsing, mcgugan-fallback]
  affects: [app-event-loop, css-cascade, canvas-rendering]
tech_stack:
  added: []
  patterns: [active_overlay for all floating overlays, variable resolution in border shorthand]
key_files:
  created: []
  modified:
    - crates/textual-rs/src/widget/select.rs
    - crates/textual-rs/src/command/palette.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/css/property.rs
    - crates/textual-rs/src/css/types.rs
    - crates/textual-rs/src/css/cascade.rs
    - crates/textual-rs/src/canvas.rs
decisions:
  - "Used active_overlay pattern for Select and CommandPalette instead of push_screen_deferred"
  - "Route both key_bindings and on_event to active overlay for full input handling"
  - "Used U+2595 (RIGHT ONE EIGHTH BLOCK) as McGugan Box right border fallback"
  - "BorderWithVariable resolved in cascade alongside Variable"
metrics:
  duration: ~8 min
  completed: 2026-03-26T23:24:38Z
  tasks_completed: 2
  tasks_total: 2
  files_modified: 7
---

# Phase 04 Plan 02: Fix Select/CommandPalette overlays, CSS border+variable, McGugan Box fallback Summary

Select and CommandPalette now use active_overlay pattern for proper overlay rendering without screen blanking; CSS `border: tall $primary` parses and resolves via theme; McGugan Box right border uses U+2595 for broad terminal compatibility.

## Completed Tasks

| Task | Name | Commit | Key Changes |
|------|------|--------|-------------|
| 1 | Convert Select and CommandPalette to active_overlay | 03a856a | select.rs, palette.rs, app.rs - overlay pattern + key routing |
| 2 | CSS border+variable and McGugan Box fallback (TDD) | 02bb722 (RED), e01ea82 (GREEN) | property.rs, cascade.rs, types.rs, canvas.rs |

## Changes Made

### Task 1: Active Overlay Pattern
- **select.rs**: `Select::on_action("open")` now sets `active_overlay` instead of `push_screen_deferred`; `SelectOverlay` calls `dismiss_overlay()` instead of `pop_screen_deferred()`; removed `is_overlay()` override
- **palette.rs**: `CommandPalette` calls `dismiss_overlay()` for Esc/Enter; removed `is_overlay()` override
- **app.rs** (main event loop): Added overlay key routing before Ctrl+P check - routes key bindings then `on_event` to active overlay; Ctrl+P now sets `active_overlay` directly
- **app.rs** (`handle_key_event`): Updated to route both `key_bindings()` and `on_event()` to overlay (was only routing bindings, missing char input for CommandPalette); removed "any unhandled key dismisses" behavior

### Task 2: CSS Border+Variable and McGugan Fallback
- **types.rs**: Added `BorderWithVariable(BorderStyle, String)` variant to `TcssValue` enum
- **property.rs**: In border parsing, tries `try_parse_variable(input)` before `parse_color()` - produces `BorderWithVariable` when `$variable` follows border style
- **cascade.rs**: `resolve_variables` now matches `BorderWithVariable` and resolves to `BorderWithColor` via theme, alongside existing `Variable` -> `Color` resolution
- **canvas.rs**: Added `RIGHT_BORDER_FALLBACK` constant (U+2595); `mcgugan_box()` uses fallback instead of `RIGHT_ONE_QUARTER` (U+1FB87) for right edge; updated existing test assertion

## Decisions Made

1. **Active overlay for all overlays**: Both Select and CommandPalette now follow the same pattern as ContextMenuOverlay - set `active_overlay` directly. This avoids the screen blanking caused by `push_screen_deferred` which replaces the entire screen.
2. **Full key routing to overlay**: The overlay key routing now calls both `key_bindings()` and `on_event()`. This is critical for CommandPalette which handles all input (chars, arrows, Esc, Enter) via `on_event` with empty `key_bindings`.
3. **U+2595 for McGugan right border**: U+1FB87 (RIGHT ONE QUARTER BLOCK) is from Unicode 13 Legacy Computing Supplement and is missing from many terminal fonts. U+2595 (RIGHT ONE EIGHTH BLOCK) is thinner but has been available since Unicode 1.1. Python Textual also uses this approach for compatibility.

## Deviations from Plan

None - plan executed exactly as written.

## Test Results

- 192 lib tests pass (including 6 new tests)
- New tests: `parse_border_tall_variable`, `parse_border_rounded_variable_with_suffix`, `parse_border_solid_hex_still_works`, `parse_border_heavy_no_color`, `resolve_cascade_border_with_variable_resolves_to_color`, `mcgugan_box_right_border_uses_fallback`
- All existing CSS, cascade, and canvas tests still pass
- Clippy clean (no new warnings)

## Known Stubs

None.

## Self-Check: PASSED

All 7 modified files exist. All 3 commits (03a856a, 02bb722, e01ea82) verified in git log.
