---
phase: 03-widget-visual-polish-demos
plan: 01
subsystem: widget-rendering
tags: [visual-polish, button, data-table, 3d-borders, zebra-stripes]
dependency_graph:
  requires: [canvas::blend_color]
  provides: [button-3d-depth, datatable-zebra-rows]
  affects: [button.rs, data_table.rs]
tech_stack:
  added: []
  patterns: [blend_color for shade computation, per-row background fill]
key_files:
  created: []
  modified:
    - crates/textual-rs/src/widget/button.rs
    - crates/textual-rs/src/widget/data_table.rs
decisions:
  - "Button 3D depth uses 25% lighter top / 35% darker bottom blend ratios"
  - "DataTable zebra stripe is 6% lighter than table background"
  - "Cursor row always overrides zebra stripe (accent highlight takes priority)"
metrics:
  duration: 246s
  completed: "2026-03-26"
  tasks_completed: 2
  tasks_total: 2
  tests_added: 5
  tests_total: 314
---

# Phase 03 Plan 01: Button 3D Depth Borders + DataTable Zebra Rows Summary

Button renders 3D depth via blend_color shading (lighter top, darker bottom, inverted on press); DataTable alternates odd-row backgrounds at 6% lighter for zebra striping.

## What Was Done

### Task 1: Button 3D depth borders
- Extracted background color from buffer cell at render time
- Computed light edge (25% toward white) and dark edge (35% toward black) using `canvas::blend_color`
- Top row of button content area painted with light edge bg, bottom row with dark edge bg
- Pressed state swaps: dark top, light bottom -- creating visual "push in" illusion
- Only applied when `area.height >= 3`; short buttons unaffected
- Existing label centering, BOLD, and REVERSED press flash preserved

### Task 2: DataTable zebra-striped rows
- Extracted table background from buffer, computed 6% lighter alternate shade
- Odd-indexed rows (`row_idx % 2 == 1`) get full-row alt_bg fill before text rendering
- Text style for alt rows also carries `bg(alt_bg)` for consistency
- Column separators on alt rows receive alt_bg as well
- Cursor row always uses accent color (green + BOLD), never receives zebra stripe

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | 4bafe49 | feat(03-01): add 3D depth borders to Button widget |
| 2 | 7356644 | feat(03-01): add zebra-striped rows to DataTable widget |

## Test Results

- 5 new tests added (3 button + 2 data_table)
- Full suite: 314 passed, 0 failed, 4 ignored
- No compiler errors; only pre-existing warnings (unused functions)

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all rendering is fully wired to live buffer data.
