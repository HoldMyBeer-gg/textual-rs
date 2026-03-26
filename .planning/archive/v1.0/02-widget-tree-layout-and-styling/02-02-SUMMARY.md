---
phase: 02-widget-tree-layout-and-styling
plan: 02
subsystem: layout
tags: [taffy, layout, flexbox, grid, dock, hit-map, ratatui]

# Dependency graph
requires:
  - phase: 02-01
    provides: "Widget trait, WidgetId, AppContext with arena/children/computed_styles/dirty maps"

provides:
  - TaffyBridge — syncs widget arena to Taffy, computes layout, exposes ratatui Rect per widget
  - taffy_style_from_computed — converts ComputedStyle to taffy::Style with flex/grid/dock/absolute support
  - MouseHitMap — sparse (col,row) → WidgetId map for mouse event routing

affects: [renderer, event-dispatch, 02-03, 02-04, phase-3]

# Tech tracking
tech-stack:
  added: ["taffy 0.9.2 — Flexbox/Grid/Block layout engine"]
  patterns:
    - "TDD: tests written first (RED), then implementation (GREEN) — 15 failing tests → 15 passing"
    - "Dirty-flag incremental sync: sync_dirty_subtree skips clean subtrees to avoid redundant Taffy updates"
    - "Absolute positioning for dock layouts: DockEdge maps to Position::Absolute with edge insets=0, opposite=auto"
    - "Z-ordering via DFS insertion order: later DFS widgets overwrite earlier ones in MouseHitMap cells"
    - "Percent TCSS values stored as 0..100, converted to 0.0..1.0 range for Taffy"

key-files:
  created:
    - crates/textual-rs/src/layout/mod.rs
    - crates/textual-rs/src/layout/bridge.rs
    - crates/textual-rs/src/layout/style_map.rs
    - crates/textual-rs/src/layout/hit_map.rs
    - crates/textual-rs/src/layout/tests.rs
  modified:
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/src/lib.rs

key-decisions:
  - "taffy 0.9.2 GridTemplateComponent::Single wraps TrackSizingFunction — not a bare Vec<TrackSizingFunction>"
  - "Dock layout uses Position::Absolute with pinned insets; absolute widget width auto-fills parent in Taffy flex containers"
  - "TaffyBridge stores TaffyTree<()> (no node context needed — WidgetId tracked in separate HashMap)"
  - "Percent dimensions divided by 100.0 at conversion boundary (TCSS stores 0-100, Taffy expects 0.0-1.0)"

patterns-established:
  - "Layout consumed by renderer: bridge.rect_for(id) returns Option<Rect> for each widget"
  - "Hit testing: build MouseHitMap from DFS-ordered widget list + layout cache after each compute_layout"
  - "Incremental relayout: sync_dirty_subtree + compute_layout called per frame; clear_dirty_subtree after render"

requirements-completed: [LAYOUT-01, LAYOUT-02, LAYOUT-03, LAYOUT-04, LAYOUT-05, LAYOUT-06, LAYOUT-07]

# Metrics
duration: 8min
completed: 2026-03-25
---

# Phase 2 Plan 02: Layout Engine — TaffyBridge + MouseHitMap Summary

**Taffy 0.9.2 layout engine integrated with widget arena: TaffyBridge computes per-widget ratatui Rects for flex/grid/dock layouts, MouseHitMap provides O(1) cell-to-widget hit testing, dirty-flag incremental sync avoids redundant re-layouts**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-25T19:09:41Z
- **Completed:** 2026-03-25T19:17:41Z
- **Tasks:** 2 (TDD: 2 RED commits + 2 GREEN commits)
- **Files modified:** 7

## Accomplishments

- TaffyBridge syncs widget arena (AppContext.children + computed_styles) into TaffyTree, computes layout, exposes ratatui Rect per widget
- All layout modes produce correct geometry: flex vertical/horizontal, grid (fr tracks), dock (absolute positioning), fractional (flex_grow), fixed/percent/auto
- MouseHitMap builds sparse cell map from layout cache — (col, row) → WidgetId with DFS z-order for overlapping widgets
- Dirty-flag driven incremental sync: `sync_dirty_subtree` skips clean subtrees so only changed widgets trigger Taffy re-sync

## Task Commits

Each task was committed atomically:

1. **RED phase (both tasks)** - `bb36529` (test: 15 failing tests for TaffyBridge, style_map, MouseHitMap)
2. **GREEN phase (both tasks)** - `520ce7a` (feat: implement TaffyBridge, style_map, and MouseHitMap)

_Note: TDD tasks have RED commit (failing tests) followed by GREEN commit (passing implementation)_

## Files Created/Modified

- `crates/textual-rs/src/layout/mod.rs` — Public layout API (re-exports TaffyBridge, taffy_style_from_computed, MouseHitMap)
- `crates/textual-rs/src/layout/bridge.rs` — TaffyBridge: sync_subtree, sync_dirty_subtree, compute_layout, rect_for, remove_subtree
- `crates/textual-rs/src/layout/style_map.rs` — taffy_style_from_computed: ComputedStyle → taffy::Style with all dimension/track helpers
- `crates/textual-rs/src/layout/hit_map.rs` — MouseHitMap: build from layout cache, hit_test(col, row) → Option<WidgetId>
- `crates/textual-rs/src/layout/tests.rs` — 15 unit tests covering all required behaviors
- `crates/textual-rs/Cargo.toml` — Added taffy = "0.9.2" dependency
- `crates/textual-rs/src/lib.rs` — Added pub mod layout

## Decisions Made

- `GridTemplateComponent::Single` wraps `TrackSizingFunction` — Taffy 0.9.2 uses this enum wrapper, not bare Vec<TrackSizingFunction>
- Dock layout via `Position::Absolute` with 3 insets=0 and opposite=auto; width auto-fills to parent width in flex container
- `TaffyTree<()>` — no node context type needed since WidgetId mapping is maintained in a separate HashMap
- Percent TCSS values (0-100) divided by 100.0 when converting to Taffy (expects 0.0-1.0 range)

## Deviations from Plan

None — plan executed exactly as written. All API nuances (GridTemplateComponent wrapping, TaffyTree type parameter) were noted in plan as "check actual API during implementation" — no structural surprises.

## Issues Encountered

One minor issue resolved during GREEN phase: `taffy::style::AvailableSpace` unused import warning (leftover from initial draft) — removed immediately before final test run.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Layout engine complete: TaffyBridge and MouseHitMap ready for use by the renderer (Phase 2 Plan 03)
- Pattern for renderer: call `bridge.sync_dirty_subtree(screen_id, &ctx)` → `bridge.compute_layout(screen_id, cols, rows)` → use `bridge.rect_for(id)` for each widget's render area
- Pattern for mouse events: `MouseHitMap::build(&dfs_order, bridge.layout_cache())` → `hit_map.hit_test(col, row)`
- No blockers for Phase 2 Plan 03

## Self-Check: PASSED

- bridge.rs: FOUND
- style_map.rs: FOUND
- hit_map.rs: FOUND
- mod.rs: FOUND
- SUMMARY.md: FOUND
- Commit bb36529 (RED phase): FOUND
- Commit 520ce7a (GREEN phase): FOUND

---
*Phase: 02-widget-tree-layout-and-styling*
*Completed: 2026-03-25*
