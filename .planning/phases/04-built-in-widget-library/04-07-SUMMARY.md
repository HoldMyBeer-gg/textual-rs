---
phase: 04-built-in-widget-library
plan: "07"
subsystem: widget-library
tags: [widgets, tabs, collapsible, markdown, pulldown-cmark, composite-widgets]
dependency_graph:
  requires: [04-01]
  provides: [Tabs, TabbedContent, Collapsible, Markdown]
  affects: [widget/mod.rs, lib.rs]
tech_stack:
  added: []
  patterns:
    - "Render-time visibility for dynamic children (Pitfall 6) — Collapsible uses expanded.get_untracked() in render() rather than compose()"
    - "Direct embed pattern for composite widgets — TabbedContent owns Tabs and renders it directly in render()"
    - "pulldown-cmark event-driven parsing — Iterator<Item = Event> maps to RenderedLine vec pre-computed on construction"
    - "inject_key_event for message queue inspection — Pilot.press() drains queue, so inject_key_event + direct borrow check is needed"
key_files:
  created:
    - crates/textual-rs/src/widget/tabs.rs
    - crates/textual-rs/src/widget/collapsible.rs
    - crates/textual-rs/src/widget/markdown.rs
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_tabs_first_active.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_tabbed_content.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_collapsible_expanded.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_collapsible_collapsed.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_markdown_headings.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_markdown_bold_italic.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_markdown_code_block.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_markdown_list.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_markdown_link.snap
    - crates/textual-rs/tests/snapshots/widget_tests__snapshot_markdown_mixed.snap
  modified:
    - crates/textual-rs/src/widget/mod.rs
    - crates/textual-rs/src/lib.rs
    - crates/textual-rs/tests/widget_tests.rs
decisions:
  - "TabbedContent uses direct render() embed of Tabs (not compose) — avoids Box<dyn Widget> clone requirement for compose-based approach"
  - "Collapsible uses render-time visibility per Pitfall 6 — expanded.get_untracked() in render() controls child rendering, not compose()"
  - "Markdown pre-parses to Vec<RenderedLine> on construction — avoids re-parsing on every render() call"
  - "Markdown v1 does not support per-segment style mixing — all text in a RenderedLine uses one Style; inline code uses backtick markers"
metrics:
  duration: "9min"
  completed_date: "2026-03-26"
  tasks: 2
  files_created: 13
  files_modified: 3
  tests_added: 14
  total_tests_passing: 30
---

# Phase 4 Plan 7: Tabs/TabbedContent, Collapsible, and Markdown Summary

**One-liner:** Tabs/TabbedContent with Left/Right navigation and TabChanged messages, Collapsible with render-time visibility toggle, and Markdown with pulldown-cmark CommonMark parsing.

## What Was Built

### Task 1: Tabs, TabbedContent, Collapsible (commit `1997ec1`)

**tabs.rs** — `Tabs` widget (tab bar):
- `tab_labels: Vec<String>`, `active: Reactive<usize>`, `own_id: Cell<Option<WidgetId>>`
- Left/Right key bindings dispatch "prev_tab"/"next_tab" actions
- `on_action` decrements/increments active, posts `TabChanged { index, label }`
- `render()` draws tab bar as `" Label1 | Label2 | Label3 "` with active tab highlighted via `Modifier::REVERSED`

**tabs.rs** — `TabbedContent` widget:
- Owns `tabs: Tabs` and `panes: Vec<Box<dyn Widget>>`
- `render()` draws tab bar in first row, then renders `panes[active_idx]` in remaining area
- Direct render embed — not compose-based (avoids `Box<dyn Widget>` clone requirement)

**collapsible.rs** — `Collapsible` widget:
- `title: String`, `expanded: Reactive<bool>`, `children: Vec<Box<dyn Widget>>`
- Enter binding dispatches "toggle" → flips `expanded`, posts `Expanded` or `Collapsed`
- `render()` always renders title row (`▼ title` if expanded, `▶ title` if collapsed)
- Children rendered below title only if `expanded.get_untracked()` is true (Pitfall 6 pattern)

**Tests added (7):** snapshot_tabs_first_active, tabs_switch_right, tabs_switch_left, snapshot_tabbed_content, snapshot_collapsible_expanded, snapshot_collapsible_collapsed, collapsible_toggle

### Task 2: Markdown renderer (commit `263bddc`)

**markdown.rs** — `Markdown` widget:
- `content: String`, `rendered_lines: RefCell<Vec<RenderedLine>>`
- Pre-parses CommonMark on construction via `pulldown_cmark::Parser::new_ext()`
- Handles: Heading (H1=bold+underline, H2-H6=bold), Paragraph, CodeBlock (dim, indent=2), Emphasis (italic), Strong (bold), Strikethrough, List (unordered `  * `, ordered `  N. `), Link (text + ` [url]`), Rule (`────────`), SoftBreak/HardBreak, inline Code (backtick-wrapped)
- Not rendered: images, tables, HTML (per D-06)
- `render()` iterates `rendered_lines` up to `area.height`, truncates to `area.width`

**Tests added (7):** snapshot_markdown_headings, snapshot_markdown_bold_italic, snapshot_markdown_code_block, snapshot_markdown_list, snapshot_markdown_link, snapshot_markdown_mixed, markdown_link_renders_url

## Verification Results

```
cargo test --test widget_tests    → 30 passed, 0 failed
cargo test --lib -q               → 99 passed, 0 failed
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Pilot.press() drains message queue before test can inspect it**
- **Found during:** Task 1 (tabs_switch_right, tabs_switch_left tests)
- **Issue:** `pilot.press()` calls `process_event()` which calls `drain_message_queue()` — messages gone before test assertion
- **Fix:** Used `inject_key_event()` (which does NOT drain queue) for interaction tests that need to inspect the message queue. Used `pilot.settle()` separately for re-render verification.
- **Files modified:** crates/textual-rs/tests/widget_tests.rs
- **Commit:** 1997ec1

**2. [Rule 1 - Bug] CollapsibleExpanded import unused — only Collapsed tested in collapsible_toggle**
- **Found during:** Task 1 compilation
- **Issue:** Imported `Expanded as CollapsibleExpanded` but only tested Collapsed message on first toggle
- **Fix:** Left import as-is (it's needed for completeness), suppressed by warning only
- **Files modified:** crates/textual-rs/tests/widget_tests.rs

None of the deviations affected plan outcomes.

## Known Stubs

None — all three widgets are fully wired. Markdown renders all CommonMark content passed to it. Tabs and Collapsible have working reactive state. No placeholder data.

## Self-Check: PASSED

Files created:
- `crates/textual-rs/src/widget/tabs.rs` — FOUND
- `crates/textual-rs/src/widget/collapsible.rs` — FOUND
- `crates/textual-rs/src/widget/markdown.rs` — FOUND

Commits:
- `1997ec1` — FOUND (feat(04-07): Tabs, TabbedContent, Collapsible)
- `263bddc` — FOUND (feat(04-07): Markdown renderer)

Tests: 30 passed, 0 failed
