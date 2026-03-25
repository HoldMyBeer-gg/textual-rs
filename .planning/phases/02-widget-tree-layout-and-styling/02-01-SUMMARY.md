---
phase: 02-widget-tree-layout-and-styling
plan: 01
subsystem: ui
tags: [rust, slotmap, widget-tree, css, arena, focus-management]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: "ratatui + crossterm + Tokio LocalSet event loop, Cargo workspace with textual-rs crate"
provides:
  - "WidgetId (DenseSlotMap generational key) + Widget trait (render/compose/on_mount/on_unmount/can_focus)"
  - "AppContext arena owning all widget state via DenseSlotMap<WidgetId, Box<dyn Widget>> + SecondaryMaps"
  - "mount_widget / unmount_widget / compose_children tree operations"
  - "push_screen / pop_screen screen stack management"
  - "advance_focus / advance_focus_backward depth-first DOM-order focus cycling"
  - "mark_widget_dirty / clear_dirty_subtree ancestor-bubbling dirty flags"
  - "ComputedStyle with all CSS-06 properties, PseudoClassSet, Declaration, apply_declarations"
affects:
  - 02-layout-engine
  - 02-css-parser
  - 02-render-pipeline
  - 03-reactive-properties
  - 03-message-pump

# Tech tracking
tech-stack:
  added:
    - "slotmap 1.0 — DenseSlotMap arena + SecondaryMap for O(1) widget lookup and safe generational keys"
  patterns:
    - "ctx-passing pattern: AppContext owns all widget state; Widget trait takes &AppContext for read access, &mut AppContext for mutations — resolves Rust borrow conflict"
    - "SecondaryMap per concern: separate maps for children, parent, computed_styles, inline_styles, dirty, pseudo_classes — avoids fat struct on Widget trait"
    - "Lifecycle via &self: on_mount/on_unmount take &self only (not &mut AppContext) — avoids borrow conflict when calling from arena; ctx-mutating lifecycle deferred to Phase 3"
    - "Bottom-up unmount: DFS collect subtree, reverse for bottom-up removal — ensures children removed before parents"

key-files:
  created:
    - "crates/textual-rs/src/css/mod.rs — pub mod types re-export"
    - "crates/textual-rs/src/css/types.rs — ComputedStyle, TcssDisplay, TcssDimension, BorderStyle, TcssColor, PseudoClassSet, Declaration, apply_declarations"
    - "crates/textual-rs/src/widget/mod.rs — Widget trait, WidgetId new_key_type!, EventPropagation"
    - "crates/textual-rs/src/widget/context.rs — AppContext with DenseSlotMap arena + 7 SecondaryMaps"
    - "crates/textual-rs/src/widget/tree.rs — mount_widget, unmount_widget, compose_children, push_screen, pop_screen, advance_focus, advance_focus_backward, mark_widget_dirty, clear_dirty_subtree"
  modified:
    - "crates/textual-rs/Cargo.toml — added slotmap = '1.0' dependency"
    - "crates/textual-rs/src/lib.rs — added pub mod css, pub mod widget, pub use Widget/WidgetId"

key-decisions:
  - "on_mount/on_unmount take &self (not &mut AppContext) — avoids borrow conflict when called from arena; ctx-mutating lifecycle deferred to Phase 3 when message pump exists"
  - "pending_mounts field added to AppContext for future Phase 3 deferred lifecycle flushing"
  - "slotmap 1.0 used instead of 1.1.1 specified in plan — latest available patch version, fully compatible"
  - "DenseSlotMap chosen over SlotMap — faster iteration required for render pass"

patterns-established:
  - "Widget trait is object-safe: default_css() has where Self: Sized gate; all other methods use &self"
  - "DFS traversal via stack-based iterative algorithm (not recursive) to avoid stack overflow on deep trees"
  - "Focus management: PseudoClass::Focus pseudo-class added/removed from pseudo_classes SecondaryMap during advance_focus"
  - "Dirty propagation stops when ancestor already dirty (short-circuit optimization)"

requirements-completed: [TREE-01, TREE-02, TREE-03, TREE-04, TREE-05]

# Metrics
duration: 4min
completed: 2026-03-25
---

# Phase 2 Plan 01: Widget Tree Foundation Summary

**SlotMap-backed widget arena (AppContext) with Widget trait, CSS property types, compose/mount/unmount lifecycle, screen stack, and depth-first focus cycling — 19 unit tests passing, no warnings**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-25T19:39:58Z
- **Completed:** 2026-03-25T19:44:05Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Widget trait defined with render/compose/on_mount/on_unmount/can_focus/widget_type_name — object-safe, stores as Box<dyn Widget>
- AppContext arena: DenseSlotMap<WidgetId, Box<dyn Widget>> + 7 SecondaryMaps (children, parent, computed_styles, inline_styles, dirty, pseudo_classes, screen_stack)
- Complete CSS property type system: ComputedStyle with 23 fields, all defaults correct per CSS-06 spec, apply_declarations with full property dispatch
- All tree operations: mount_widget initializes SecondaryMaps + wires parent/child; unmount_widget recursively removes subtree bottom-up; compose_children calls widget.compose() and mounts children
- Screen stack: push_screen / pop_screen with full subtree lifecycle management
- Focus management: advance_focus / advance_focus_backward with DFS DOM-order, PseudoClass::Focus pseudo-class updates, wrapping, skips non-focusable widgets
- Dirty tracking: mark_widget_dirty bubbles to ancestors (short-circuits when already dirty); clear_dirty_subtree recursive clear

## Task Commits

Each task was committed atomically:

1. **Task 1: CSS type definitions and Widget trait with AppContext arena** - `e87fbfd` (feat)
2. **Task 2: Widget tree operations — mount, unmount, compose, screen stack, focus** - `5c28337` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `crates/textual-rs/src/css/types.rs` — ComputedStyle, TcssDisplay, TcssDimension, BorderStyle, TcssColor, PseudoClassSet, Declaration, apply_declarations, TcssValue
- `crates/textual-rs/src/css/mod.rs` — module declaration and re-exports
- `crates/textual-rs/src/widget/mod.rs` — Widget trait with WidgetId (new_key_type!), EventPropagation, unit tests
- `crates/textual-rs/src/widget/context.rs` — AppContext struct with DenseSlotMap arena + SecondaryMaps
- `crates/textual-rs/src/widget/tree.rs` — all tree operation functions + 13 unit tests
- `crates/textual-rs/Cargo.toml` — added slotmap 1.0
- `crates/textual-rs/src/lib.rs` — added css and widget module declarations and re-exports

## Decisions Made

- on_mount/on_unmount take `&self` only (not `&mut AppContext`): avoids the Rust borrow conflict of holding a widget reference from arena while also mutating arena. Ctx-mutating lifecycle hooks deferred to Phase 3 when the message pump exists. The plan's own "REVISED on_mount approach" section specified this as the correct approach.
- slotmap 1.0 used (plan specified 1.1.1, but 1.0 is the latest stable on crates.io and fully API-compatible).
- pending_mounts field retained in AppContext for Phase 3 flush_pending_mounts pattern.

## Deviations from Plan

None — plan executed as specified. The plan itself included a "REVISED on_mount approach" section documenting the &self-only signature, which was the final spec. Used slotmap 1.0 rather than 1.1.1 (1.0 is latest stable, API identical).

## Issues Encountered

None — cargo build and cargo test both clean on first attempt.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- AppContext arena is the central owner ready for layout engine (Plan 02) and CSS parser (Plan 03)
- Widget trait + WidgetId exported from crate root — downstream crates can implement Widget
- Screen stack push/pop ready for screen navigation in Plan 05
- Focus cycling ready for keyboard event handling in Phase 3
- Blocker resolved: SlotMap borrow ergonomics confirmed via ctx-passing pattern (AppContext pattern, not HopSlotMap)

---
*Phase: 02-widget-tree-layout-and-styling*
*Completed: 2026-03-25*
