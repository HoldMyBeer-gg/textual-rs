---
phase: 08-enhanced-display-widgets
plan: "02"
subsystem: loading-spinner
tags: [widget, loading, animation, overlay, spinner]
dependency_graph:
  requires: []
  provides: [LoadingIndicator, ctx.set_loading, draw_loading_spinner_overlay]
  affects: [AppContext, render_widget_tree, full_render_pass]
tech_stack:
  added: []
  patterns: [SecondaryMap-per-widget-state, Cell-tick-counter, overlay-after-render]
key_files:
  created:
    - crates/textual-rs/src/widget/loading_indicator.rs
  modified:
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/tests/widget_tests.rs
decisions:
  - LoadingIndicator owns no tick state — uses ctx.spinner_tick for synchronized animation across all overlay instances
  - Overlay is drawn using full rect (including borders), not content_area, for visual continuity
  - spinner_tick is incremented in both full_render_pass and render_to_test_backend for test determinism
metrics:
  duration_minutes: 20
  completed_date: "2026-03-27"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 6
---

# Phase 08 Plan 02: LoadingIndicator and Loading Overlay Summary

**One-liner:** Per-widget loading overlay via ctx.set_loading() using SecondaryMap + standalone LoadingIndicator with synchronized braille spinner animation.

## What Was Built

### LoadingIndicator widget (src/widget/loading_indicator.rs)
- Standalone widget that renders a braille spinner animation centered in its area
- Uses 8 braille characters (U+28FE through U+28B7) cycling at ~15fps (tick/2 index)
- When skip_animations=true (tests): renders static "Loading..." text for deterministic snapshots
- Uses ctx.spinner_tick for synchronized animation — all loading overlays animate in lock-step
- pub fn draw_loading_spinner_overlay() fills area with dark bg (Rgb 20,20,28) and centers spinner

### AppContext additions (src/widget/context.rs)
- loading_widgets: RefCell<SecondaryMap<WidgetId, bool>> — per-widget loading state, parallel to existing computed_styles/dirty maps
- spinner_tick: Cell<u8> — global tick counter, wrapping, incremented once per render pass
- set_loading(id: WidgetId, loading: bool) — inserts/removes from SecondaryMap; uses &self for ergonomic call from on_action

### Render integration (src/app.rs)
- render_widget_tree: after widget.render(), checks loading_widgets.contains_key(id) and calls draw_loading_spinner_overlay with full rect
- full_render_pass: increments spinner_tick after terminal.draw()
- render_to_test_backend: also increments spinner_tick for consistent behavior in tests

### Tests (tests/widget_tests.rs)
- loading_indicator_skip_animations: TestApp renders LoadingIndicator, verifies "Loading..." appears
- loading_indicator_animated_spinner: direct render with skip_animations=false, tick=0, verifies U+28FE (⣾) appears
- loading_overlay_set_and_clear: set_loading(true) → overlay appears; set_loading(false) → original content returns
- loading_overlay_multiple_widgets: two Labels in loading state simultaneously, both IDs in map, overlay renders

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] crates/textual-rs/src/widget/loading_indicator.rs exists
- [x] crates/textual-rs/src/widget/context.rs has loading_widgets and spinner_tick fields
- [x] crates/textual-rs/src/app.rs has loading overlay check and spinner_tick increment
- [x] cargo test -p textual-rs loading: 4 passed
- [x] cargo test -p textual-rs --tests: 100 passed, 0 failed

## Self-Check: PASSED
