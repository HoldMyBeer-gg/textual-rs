---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Widget Parity & Ship
status: Phase 05 Plan 02 executed
stopped_at: 05-03 complete — tutorial_06_screens example created and committed
last_updated: "2026-03-28T07:05:57.138Z"
last_activity: 2026-03-28
progress:
  total_phases: 6
  completed_phases: 4
  total_plans: 12
  completed_plans: 12
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-26)

**Core value:** Developers can build Textual-quality TUI applications in Rust -- declare widgets, style with CSS, react to events, get a polished result on any terminal.
**Current focus:** Phase 05 — screen-stack

## Current Position

Phase: 05
Plan: 03 complete
Status: Phase 05 complete (3/3 plans)
Last activity: 2026-03-28

### Progress

```
v1.3: [██        ] 16% (1/6 phases)
```

Phase 5  [x] Screen Stack (3/3 plans complete — completed 2026-03-28)
Phase 6  [ ] Render-Only Foundation Widgets
Phase 7  [ ] List and Selection Widgets
Phase 8  [ ] Enhanced Display Widgets
Phase 9  [ ] Complex Widgets
Phase 10 [x] Platform Verification and Publish (completed 2026-03-28)

### History

- v1.0 MVP: 5 phases, 22 plans — shipped 2026-03-26
- v1.1 Visual Parity: 3 phases, 6 plans — shipped 2026-03-27
- v1.2 Production Readiness: 1 phase, 5 plans — shipped 2026-03-27

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260326-uwc | Mouse capture push/pop stack and Shift modifier bypass | 2026-03-27 | 2674edb | [260326-uwc](./quick/260326-uwc-mouse-capture-push-pop-stack-and-shift-m/) |

## Accumulated Context

### Key Decisions (v1.3)

- push_screen_wait + run_worker + WorkerResult<T> is the canonical pattern for typed modal result delivery (05-03)
- push_screen_wait uses single-slot pending_pop_result (not HashMap): at most one pop-with-result per event cycle (05-02)
- pop_screen_with on non-wait screen silently discards result — safe no-op (05-02)
- tokio sync feature added for oneshot channel support in push_screen_wait (05-02)
- compute_layout clears only top-screen subtree entries from layout_cache; background entries preserved for layered render (05-01)
- full_render_pass renders all screens bottom-to-top; CSS/layout/dirty-clear remain top-screen only for performance (05-01)
- pop_screen_deferred no-op on last screen enforced in process_deferred_screens with len<=1 guard (05-01)
- CI uses dtolnay/rust-toolchain@stable (not dtolnay/rust-action/setup@v1 which returns 404) (10-01)
- Docs CI job added with RUSTDOCFLAGS=-D warnings; will fail until Plan 02 adds rustdoc — expected (10-01)
- #![deny(missing_docs)] belongs in Plan 02 alongside the full doc pass, not Plan 01 (10-01)
- #![allow(missing_docs)] added to widget/mod.rs suppresses all widget subtree lint — allows deny at crate level while deferring widget docs to 10-04 (10-02)
- All core infrastructure (31 files) fully documented; widget docs deferred to plan 10-04 (10-02)
- LoadingIndicator uses ctx.spinner_tick (not own tick) for synchronized overlay animation across all instances (08-02)
- set_loading() uses SecondaryMap<WidgetId,bool> — supports multiple simultaneous loading widgets (08-02)
- Loading overlay drawn over full rect (including borders) for visual continuity (08-02)
- RichLog uses imported `Line` type alias — `Reactive<Vec<Line<'static>>>` is `Reactive<Vec<ratatui::text::Line<'static>>>` (08-01)
- RichLog PageUp/PageDown added beyond minimum 4 bindings for usability (08-01)
- `focus_history: Vec<Option<WidgetId>>` must be added to AppContext before any screen stack consumer code is written
- Toast uses `Vec<ToastEntry>` on AppContext — NOT `active_overlay` (that slot is single-instance only)
- MaskedInput cursor tracked in raw-value space only; display cursor derived per render
- ContentSwitcher uses `recompose_widget` pattern, not CSS display toggling (Taffy has no display:none)
- DirectoryTree filesystem I/O always via `ctx.run_worker`, never in `on_event` or `compose`
- cargo publish --dry-run for textual-rs errors (not warns) when macros not on crates.io yet — expected, resolves after publishing macros first (10-03)
- Publish order mandatory: textual-rs-macros first, ~60s propagation, then textual-rs (10-03)

### Research Flags (resolve during phase planning)

- Phase 5: screen suspend/resume lifecycle events scope (push_screen_wait resolved in 05-02)
- Phase 8: `widget.loading = true` base-class overlay integration scope vs. standalone widget only
- Phase 9: DirectoryTree symlink detection on Windows NTFS; Toast z-order vs. active_overlay

### Dependencies

- walkdir 2 (new dep) — DirectoryTree filesystem traversal
- serde_json 1 with preserve_order feature (new dep) — Pretty widget

## Session Continuity

Last session: 2026-03-28T07:05:57.134Z
Stopped at: 05-03 complete — tutorial_06_screens example created and committed
Next action: Phase 06 — Render-Only Foundation Widgets
