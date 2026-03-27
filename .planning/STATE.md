---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Widget Parity & Ship
status: completed
stopped_at: Completed 09-01-PLAN.md (MaskedInput widget)
last_updated: "2026-03-27T21:20:00Z"
last_activity: 2026-03-27
progress:
  total_phases: 6
  completed_phases: 1
  total_plans: 3
  completed_plans: 1
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-26)

**Core value:** Developers can build Textual-quality TUI applications in Rust -- declare widgets, style with CSS, react to events, get a polished result on any terminal.
**Current focus:** Phase 09 — complex-widgets

## Current Position

Phase: 9
Plan: 1 (complete)
Status: 09-01 complete, 09-02 and 09-03 in progress (parallel)
Last activity: 2026-03-27

### Progress

```
v1.3: [          ] 0% (0/6 phases)
```

Phase 5  [ ] Screen Stack
Phase 6  [ ] Render-Only Foundation Widgets
Phase 7  [ ] List and Selection Widgets
Phase 8  [ ] Enhanced Display Widgets
Phase 9  [ ] Complex Widgets
Phase 10 [ ] Platform Verification and Publish

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

- LoadingIndicator uses ctx.spinner_tick (not own tick) for synchronized overlay animation across all instances (08-02)
- set_loading() uses SecondaryMap<WidgetId,bool> — supports multiple simultaneous loading widgets (08-02)
- Loading overlay drawn over full rect (including borders) for visual continuity (08-02)
- RichLog uses imported `Line` type alias — `Reactive<Vec<Line<'static>>>` is `Reactive<Vec<ratatui::text::Line<'static>>>` (08-01)
- RichLog PageUp/PageDown added beyond minimum 4 bindings for usability (08-01)
- `focus_history: Vec<Option<WidgetId>>` must be added to AppContext before any screen stack consumer code is written
- Toast uses `Vec<ToastEntry>` on AppContext — NOT `active_overlay` (that slot is single-instance only)
- MaskedInput cursor tracked in raw-value space only; display cursor derived per render via precomputed slot_display_cols O(1) lookup (09-01)
- MaskedInput raw_value stores only user-typed chars; separators exist only in display string built at render time (09-01)
- Case modifiers (>/< in mask) consume during parse, not stored as slots — only influence slot_case vec (09-01)
- ContentSwitcher uses `recompose_widget` pattern, not CSS display toggling (Taffy has no display:none)
- DirectoryTree filesystem I/O always via `ctx.run_worker`, never in `on_event` or `compose`

### Research Flags (resolve during phase planning)

- Phase 5: `push_screen_wait` async variant scope decision; screen suspend/resume lifecycle events scope
- Phase 8: `widget.loading = true` base-class overlay integration scope vs. standalone widget only
- Phase 9: DirectoryTree symlink detection on Windows NTFS; Toast z-order vs. active_overlay

### Dependencies

- walkdir 2 (new dep) — DirectoryTree filesystem traversal
- serde_json 1 with preserve_order feature (new dep) — Pretty widget

## Session Continuity

Last session: 2026-03-27
Stopped at: Completed 09-01-PLAN.md (MaskedInput widget)
Resume file: None
Next action: Continue phase 09 with 09-02 (DirectoryTree) and 09-03 (Toast)
