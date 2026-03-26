---
phase: 02-interactive-states-rendering
plan: 01
subsystem: widget-interactive-states
tags: [hover, pressed, invalid-border, pseudo-class, visual-feedback]
dependency_graph:
  requires: []
  provides: [hover-tracking, button-pressed-state, input-invalid-border, border-color-override-trait]
  affects: [app-event-loop, render-pipeline, widget-trait]
tech_stack:
  added: []
  patterns: [border-color-override-trait-method, pseudo-class-hover-tracking, single-frame-flash-pattern]
key_files:
  created:
    - crates/textual-rs/tests/interactive_states.rs
  modified:
    - crates/textual-rs/src/app.rs
    - crates/textual-rs/src/widget/context.rs
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/widget/button.rs
    - crates/textual-rs/src/widget/input.rs
decisions:
  - "border_color_override() trait method chosen over pseudo-class approach for Input invalid border -- avoids needing &mut AppContext from render"
  - "Hover shows light blue tint (100, 180, 255) on border, focus takes priority over hover"
  - "Button pressed uses single-frame REVERSED modifier flash, reset in render()"
  - "Invalid border only shows when value is non-empty (empty field stays normal)"
metrics:
  duration: ~8 minutes
  completed: 2026-03-26
---

# Phase 02 Plan 01: Interactive Visual States Summary

Hover tracking via MouseMove events, button pressed/active state with REVERSED flash, and input invalid border color override using a new Widget trait method.

## What Was Built

### Hover Tracking (STATE-02)
- Added `hovered_widget: Option<WidgetId>` to AppContext
- MouseEventKind::Moved handled in both `run_async` (production) and `handle_mouse_event` (TestApp)
- Hit-test determines target widget; PseudoClass::Hover set/cleared on old/new
- render_widget_tree applies light blue tint (100, 180, 255) to hovered widget borders
- Focus indicator takes visual priority over hover

### Button Pressed State (STATE-03)
- Added `pressed: Cell<bool>` to Button struct
- on_action("press") sets pressed flag before posting Pressed message
- render() checks pressed flag: applies BOLD | REVERSED modifier for single-frame flash
- Flag reset to false after render consumes it

### Input Invalid Border (STATE-05)
- Added `border_color_override(&self) -> Option<(u8, u8, u8)>` to Widget trait (default None)
- Input overrides: returns (186, 60, 91) when !valid && !empty
- render_widget_tree checks border_color_override before focus/hover checks (highest priority)
- Invalid border color only shown for non-empty invalid values

### Render Pipeline Priority
Border color resolution in render_widget_tree:
1. Widget border_color_override (invalid state) -- highest
2. Focus indicator (accent green 0, 255, 163)
3. Hover tint (light blue 100, 180, 255)
4. Default computed CSS style

## Integration Tests (6 passing)
1. `button_press_shows_reversed_modifier` -- verifies REVERSED on label cells during press
2. `input_invalid_shows_red_border_override` -- digit input triggers red border
3. `input_valid_has_no_border_override` -- letter input returns None
4. `hover_sets_pseudo_class_on_hovered_widget` -- MouseMove sets/clears Hover pseudo-class
5. `focused_widget_shows_accent_border` -- Tab focuses widget, accent green in buffer
6. `listview_selected_item_accent_bold` -- selected item renders accent+BOLD

## Deviations from Plan

None -- plan executed exactly as written.

## Decisions Made

1. **border_color_override trait method** -- Chosen over pseudo-class or inline-style approaches because render() has &self not &mut AppContext, and the trait method lets render_widget_tree query widget state cleanly.
2. **Priority order in render** -- Widget override > Focus > Hover > Default. Invalid state is more important than focus indication.
3. **Single-frame flash** -- Button pressed state is consumed during render, giving a one-frame visual pulse. This matches Textual's behavior.

## Known Stubs

None -- all features are fully wired.

## Self-Check: PASSED

- All 6 key files verified present on disk
- Commit 65ea626 (Task 1 implementation) verified in git log
- Commit 617a9a4 (Task 2 integration tests) verified in git log
- 151 lib tests pass, 6 integration tests pass
- 2 pre-existing snapshot failures (snapshot_placeholder_default, snapshot_placeholder_labeled) unrelated to this plan
