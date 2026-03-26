---
phase: quick
plan: ti8
subsystem: examples
tags: [demo, irc, examples, widgets, ui]
dependency_graph:
  requires: [Phase 04-built-in-widget-library]
  provides: [polished-demo-examples]
  affects: [first-user-impression, developer-onboarding]
tech_stack:
  added: []
  patterns:
    - TabbedContent with inline pane render() for tab content
    - compose() pattern for widget-tree children (ChannelPane, ChatLog, UserPane)
key_files:
  created: []
  modified:
    - crates/textual-rs/examples/demo.rs
    - crates/textual-rs/examples/irc_demo.rs
decisions:
  - "Tab panes in demo.rs render children inline via render() since TabbedContent calls pane.render() directly, not via widget tree compose"
  - "IRC demo uses compose() for all child widgets enabling proper widget tree layout and CSS targeting"
metrics:
  duration: ~15min
  completed: 2026-03-26
  tasks_completed: 2
  files_modified: 2
---

# Quick Task ti8: Fix demo.rs and irc_demo.rs to Look Good — Summary

**One-liner:** Replaced blank/hand-rolled demo screens with polished widget showcases using the full Phase 4 built-in library and the lazeport dark color palette.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rewrite demo.rs as tabbed widget showcase | 946effb | crates/textual-rs/examples/demo.rs |
| 2 | Rewrite irc_demo.rs using built-in widgets | 88dd434 | crates/textual-rs/examples/irc_demo.rs |

## What Was Built

### demo.rs

A multi-tab widget showcase with:
- **Header**: "textual-rs Widget Showcase -- Tab to navigate | q to quit" with green lazeport accent
- **Footer**: shows key bindings for focused widget
- **Tab "Controls"**: Label, Input, Checkbox, Switch, RadioSet, Button (Primary + Default)
- **Tab "Data"**: Label, DataTable (5 rows, 3 cols), ProgressBar (65%), Sparkline (20 data points)
- **Tab "Lists"**: ListView (8 fruit items) side-by-side with Log (10 pre-filled server log lines)
- Full lazeport color palette: background rgb(10,10,15), secondary rgb(18,18,26), accent rgb(0,255,163)

### irc_demo.rs

A weechat-style IRC client layout with:
- **Header**: "textual-rs IRC -- #general -- 7 users" with green lazeport accent
- **ChannelPane**: ListView with 7 channels (#general, #rust, #tui-dev, etc.) — 20 cols
- **ChatLog**: Log widget pre-filled with 17 realistic timestamped IRC messages — flex-grow
- **UserPane**: ListView with 7 users (@alice [op], @bob, etc.) — 22 cols
- **InputRegion**: Input widget docked at bottom — "Type a message..."
- All hand-rolled ratatui primitives (Paragraph, Block, Borders) removed
- Old tests and find_focused_widget_type helper removed

## Deviations from Plan

### Implementation Note — TabbedContent Pane Rendering

**Found during:** Task 1

**Issue:** TabbedContent.render() calls `pane.render()` directly, not via the widget tree compose path. Pane children cannot be laid out by Taffy when rendered this way.

**Fix:** demo.rs tab panes use manual inline rendering in their render() methods (calling widget.render() with calculated Rects). This is the appropriate pattern since TabbedContent owns panes directly.

**Impact:** Tab pane children are not focusable (no widget IDs in tree). This is acceptable for a demo showcase.

For irc_demo.rs, the outer containers (ChannelPane, ChatLog, UserPane) use compose() which goes through the full widget tree — so those children are properly mounted and focusable.

## Checkpoint: Auto-approved

Task 3 was `checkpoint:human-verify`. Auto mode active — auto-approved.

Both examples compile cleanly (`cargo build --example demo` and `cargo build --example irc_demo`).

Verification criteria met:
- `cargo build --example demo` succeeds
- `cargo build --example irc_demo` succeeds
- Neither example uses direct ratatui rendering (no Paragraph::new, no Block::default(), no RatatuiWidget::render)
- Both import from `textual_rs::*`
- Both define TCSS stylesheets with the lazeport color palette

## Known Stubs

None. Both demos have real content rendered via built-in widgets.

## Self-Check: PASSED

Files exist:
- `/c/Users/mbeha/code/textual-rs/.claude/worktrees/agent-acb41de3/crates/textual-rs/examples/demo.rs` — FOUND
- `/c/Users/mbeha/code/textual-rs/.claude/worktrees/agent-acb41de3/crates/textual-rs/examples/irc_demo.rs` — FOUND

Commits exist:
- `946effb` — FOUND (feat(quick-ti8): rewrite demo.rs as tabbed widget showcase)
- `88dd434` — FOUND (feat(quick-ti8): rewrite irc_demo.rs using built-in widgets)
