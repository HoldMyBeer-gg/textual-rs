---
phase: 04-production-readiness
plan: 03
subsystem: terminal
tags: [terminal-detection, resize, capability-detection]
dependency_graph:
  requires: []
  provides: [terminal-caps, resize-reflow]
  affects: [app, widget-context, rendering]
tech_stack:
  added: []
  patterns: [environment-variable-detection, platform-conditional-compilation]
key_files:
  created:
    - crates/textual-rs/src/terminal.rs (TerminalCaps, ColorDepth, detect functions)
  modified:
    - crates/textual-rs/src/widget/context.rs (terminal_caps field on AppContext)
    - crates/textual-rs/src/app.rs (resize sets needs_full_sync, debug caps logging)
decisions:
  - "Windows assumed TrueColor if WT_SESSION present, EightBit otherwise"
  - "Unicode assumed true on Windows (modern conhost + Windows Terminal)"
  - "Mouse always true since crossterm manages mouse capture"
  - "Title disabled for TERM=dumb and TERM=linux only"
metrics:
  duration: 266s
  completed: "2026-03-26T23:18:31Z"
  tasks: 2
  files: 3
---

# Phase 04 Plan 03: Terminal Capability Detection & Resize Reflow Summary

TerminalCaps struct detects color depth (TrueColor/256/16/none), Unicode, mouse, and title support from env vars with Windows-specific heuristics; resize triggers full layout sync.

## Task Completion

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Terminal capability detection | ab8a749 | terminal.rs |
| 2 | Wire caps into App + resize reflow | 956e11b | context.rs, app.rs |

## What Was Built

### TerminalCaps struct (terminal.rs)
- `ColorDepth` enum: NoColor, Standard, EightBit, TrueColor
- `TerminalCaps` struct with color_depth, unicode, mouse, title fields
- `detect()` method probes COLORTERM, TERM, WT_SESSION, LC_ALL/LANG/LC_CTYPE
- Platform-conditional compilation for Windows vs Unix detection paths
- 6 unit tests covering detection, equality, clone/debug, Windows-specific assertions

### AppContext integration (context.rs)
- `terminal_caps: TerminalCaps` field added to AppContext
- Initialized automatically via `detect_capabilities()` in `AppContext::new()`
- Widgets can now read `ctx.terminal_caps` to degrade gracefully

### Resize reflow (app.rs)
- Resize handler now sets `self.needs_full_sync = true` before `full_render_pass()`
- This forces `bridge.sync_subtree()` (full tree walk) instead of `sync_dirty_subtree()`
- Ensures all widgets get proper layout recomputation after terminal resize
- Debug builds log detected capabilities to stderr on startup

## Decisions Made

1. **Color detection priority:** COLORTERM > TERM 256color > WT_SESSION (Windows) > Standard fallback
2. **Windows defaults:** TrueColor if Windows Terminal detected, EightBit for conhost, Unicode always true
3. **Informational-only for now:** Caps stored but no rendering downgrade yet; future plans use it for ASCII borders, 256-color palette fallback

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all detection logic is fully implemented and functional.

## Verification

- 174 lib unit tests pass (including 6 new terminal tests)
- cargo clippy clean (only pre-existing warnings)
- cargo build succeeds

## Self-Check: PASSED

- All 3 key files exist on disk
- Both commit hashes (ab8a749, 956e11b) verified in git log
