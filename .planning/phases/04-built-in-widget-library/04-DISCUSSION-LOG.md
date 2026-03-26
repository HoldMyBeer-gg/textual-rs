# Phase 4: Built-in Widget Library - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 04-built-in-widget-library
**Areas discussed:** Widget batching, Scroll implementation, TextArea scope, Select dropdown, Markdown rendering
**Mode:** Auto (all areas auto-selected, recommended defaults chosen)

---

## Widget Batching Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Input first, then display | Input widgets exercise reactive + event systems first, validating before complex widgets | auto |
| All widgets in parallel | Maximum parallelism, higher risk of integration issues | |
| By complexity | Simple widgets first (Label, Placeholder), complex last (DataTable, TextArea) | |

**User's choice:** [auto] Input widgets first, then display/layout (recommended default)
**Notes:** Matches roadmap plan split (04-01 input, 04-02 display)

---

## Scroll Implementation

| Option | Description | Selected |
|--------|-------------|----------|
| Simple offset scroll | Track scroll_offset as Reactive<usize>, render from offset into viewport | auto |
| Virtual scroll | Only render visible items, recycle DOM nodes equivalent | |
| Hybrid | Simple for small lists, virtual for large | |

**User's choice:** [auto] Simple offset scroll (recommended default)
**Notes:** Virtual scrolling deferred to v2

---

## TextArea Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Basic multi-line | Cursor, selection, line numbers, no syntax highlighting or undo | auto |
| Full editor | Syntax highlighting, undo/redo, multiple cursors | |
| Minimal | Insert/delete only, no selection or line numbers | |

**User's choice:** [auto] Basic multi-line (recommended default)
**Notes:** Syntax highlighting is v2 (WIDGET-V2-02)

---

## Select Widget Dropdown

| Option | Description | Selected |
|--------|-------------|----------|
| Overlay screen push | Push temporary screen with option list, pop on select | auto |
| Inline expand | Expand options below the select widget inline | |
| Floating panel | Render overlay without screen stack | |

**User's choice:** [auto] Overlay screen push (recommended default)
**Notes:** Consistent with Textual's model, reuses screen stack

---

## Markdown Rendering Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Terminal subset | Headers, bold, italic, code, lists, links, rules -- no images/tables | auto |
| Full CommonMark | Full spec including tables, images as alt text | |
| Minimal | Headers and paragraphs only | |

**User's choice:** [auto] Terminal subset (recommended default)
**Notes:** Tables deferred to v2

## Claude's Discretion

- Exact keyboard shortcuts per widget
- DataTable internal data structures
- Tree expand/collapse behavior
- ProgressBar indeterminate animation
- Sparkline rendering algorithm
- Log auto-scroll details
- Header/Footer layout
- Placeholder content generation

## Deferred Ideas

- Virtual scrolling (v2)
- Syntax highlighting (v2)
- Undo/redo (v2)
- ContentSwitcher (v2)
- DirectoryTree (v2)
- Markdown tables (v2)
- Interactive scrollbar (v2)
