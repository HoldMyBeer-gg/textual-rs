# Phase 4: Built-in Widget Library - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

All 22 v1 widgets are implemented as `Box<dyn Widget>` structs, styled via TCSS with `default_css()`, keyboard-interactive where applicable, emitting typed messages via `on_event` dispatch, and covered by `TestApp`/`Pilot` snapshot tests. This phase makes textual-rs usable as a complete application framework. The derive macro, Worker API, and command palette are Phase 5.

</domain>

<decisions>
## Implementation Decisions

### Widget Implementation Strategy
- **D-01:** Two-wave batching: input widgets first (Label, Button, Input, TextArea, Checkbox, Switch, RadioButton/RadioSet, Select), then display/layout widgets (ListView, DataTable, Tree, ProgressBar, Sparkline, Log, Markdown, Tabs/TabbedContent, Collapsible, Vertical/Horizontal, ScrollView, Header, Footer, Placeholder). Input widgets exercise the reactive + event systems first, providing validation before building more complex display widgets.
- **D-02:** Each widget follows the same pattern: struct implementing `Widget` trait, `default_css()` for TCSS defaults, `render()` writing to ratatui `Buffer`, `on_event()` for keyboard/mouse handling, message types for parent communication, and at least one `TestApp` snapshot test plus keyboard interaction tests for interactive widgets.

### Scroll Behavior
- **D-03:** Simple offset-based scrolling for v1. ScrollView, ListView, DataTable, and Tree track a `scroll_offset: Reactive<usize>` (line/row offset). Content renders from offset into the viewport area. Scrollbar is a visual indicator showing position, not interactive in v1. Virtual scrolling (rendering only visible items) deferred to v2.

### TextArea Scope
- **D-04:** Basic multi-line text editor. Supports: line insertion/deletion, cursor movement (arrows, Home/End, Ctrl+arrow for word jump), text selection (Shift+arrow), copy/paste via clipboard (if crossterm supports), line numbers option. Does NOT support: syntax highlighting, undo/redo stack, multiple cursors, soft wrapping. These are v2 features.

### Select Widget
- **D-05:** Select renders as a single-line display showing the current selection. When activated (Enter key), it pushes a temporary overlay screen with the option list. Selecting an option pops the overlay and emits `Select::Changed`. This matches Textual's popup approach and reuses the existing screen stack from Phase 2.

### Markdown Widget
- **D-06:** Terminal-appropriate Markdown subset. Renders: headers (h1-h6 with styling), bold, italic, code spans, code blocks (with border), bullet and numbered lists, horizontal rules, links (displayed as `text [url]`). Does NOT render: images, tables, HTML. Parsing via a lightweight Markdown parser crate (pulldown-cmark or similar).

### Message Naming Convention
- **D-07:** Each widget's messages are defined as public structs nested in the widget's module. Convention: `Button::Pressed`, `Input::Changed { value: String }`, `Checkbox::Changed { checked: bool }`, `Select::Changed { value: String, index: usize }`. All implement the `Message` trait from Phase 3. Messages bubble up by default.

### Widget Reactive State
- **D-08:** Interactive widgets use `Reactive<T>` for their primary state. Examples: `Input { value: Reactive<String> }`, `Checkbox { checked: Reactive<bool> }`, `ProgressBar { progress: Reactive<f64> }`. State changes automatically trigger re-renders via the Phase 3 render batching system.

### Default TCSS Styles
- **D-09:** Every widget provides `default_css()` matching Textual's default visual appearance. Button gets border + padding, Input gets border, focused widgets get highlight border color, disabled widgets get dimmed opacity. The Python Textual source in `textual/` subdirectory serves as the primary reference for default style values.

### Claude's Discretion
- Exact keyboard shortcuts for each interactive widget (follow Textual conventions where possible)
- Internal data structures for DataTable (Vec<Vec<Cell>> vs column-major)
- Tree node expansion/collapse animation (if any)
- ProgressBar indeterminate mode animation approach
- Sparkline rendering algorithm (braille vs block chars)
- Log widget auto-scroll behavior details
- Header/Footer layout strategy
- Placeholder widget content generation

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python Textual Reference (widget implementations)
- `textual/src/textual/widgets/_label.py` — Label widget: static/reactive text display
- `textual/src/textual/widgets/_button.py` — Button: variants, pressed message, keyboard
- `textual/src/textual/widgets/_input.py` — Input: single-line, placeholder, validation, Changed/Submitted messages
- `textual/src/textual/widgets/_text_area.py` — TextArea: multi-line editor, line numbers
- `textual/src/textual/widgets/_checkbox.py` — Checkbox: toggle, Changed message
- `textual/src/textual/widgets/_switch.py` — Switch: animated toggle
- `textual/src/textual/widgets/_radio_button.py` — RadioButton/RadioSet: mutual exclusion
- `textual/src/textual/widgets/_select.py` — Select: dropdown with overlay
- `textual/src/textual/widgets/_list_view.py` — ListView: scrollable selectable list
- `textual/src/textual/widgets/_data_table.py` — DataTable: columns, rows, sorting, scrolling
- `textual/src/textual/widgets/_tree.py` — Tree: hierarchical expand/collapse
- `textual/src/textual/widgets/_progress_bar.py` — ProgressBar: determinate/indeterminate
- `textual/src/textual/widgets/_sparkline.py` — Sparkline: inline chart
- `textual/src/textual/widgets/_log.py` — Log: scrolling text log
- `textual/src/textual/widgets/_markdown.py` — Markdown: rendered display
- `textual/src/textual/widgets/_tabs.py` — Tabs/TabbedContent: tab navigation
- `textual/src/textual/widgets/_collapsible.py` — Collapsible: expand/collapse container
- `textual/src/textual/widgets/_header.py` — Header: app title bar
- `textual/src/textual/widgets/_footer.py` — Footer: key binding help
- `textual/src/textual/widgets/_placeholder.py` — Placeholder: dev placeholder

### Existing Rust Code (Phase 2-3 output)
- `crates/textual-rs/src/widget/mod.rs` — Widget trait with render, compose, on_event, key_bindings, default_css
- `crates/textual-rs/src/widget/context.rs` — AppContext: arena, focus, dirty flags, message queue, event_tx
- `crates/textual-rs/src/reactive/mod.rs` — Reactive<T>, ComputedReactive<T>, spawn_render_effect
- `crates/textual-rs/src/event/message.rs` — Message trait for typed widget messages
- `crates/textual-rs/src/event/dispatch.rs` — dispatch_message, event bubbling
- `crates/textual-rs/src/event/keybinding.rs` — KeyBinding struct for widget key bindings
- `crates/textual-rs/src/css/types.rs` — ComputedStyle, CssProperty, BorderStyle
- `crates/textual-rs/src/css/cascade.rs` — Stylesheet, resolve_cascade, apply_cascade_to_tree
- `crates/textual-rs/src/testing/mod.rs` — TestApp harness
- `crates/textual-rs/src/testing/pilot.rs` — Pilot: press, type_text, click, settle
- `crates/textual-rs/src/testing/assertions.rs` — assert_buffer_lines, assert_cell

### Project Planning Documents
- `.planning/REQUIREMENTS.md` — WIDGET-01 through WIDGET-22 definitions
- `.planning/ROADMAP.md` — Phase 4 success criteria, plan structure

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Widget` trait in `widget/mod.rs` — base trait all 22 widgets implement; already has render, compose, on_event, key_bindings, default_css, widget_type_name
- `Reactive<T>` in `reactive/mod.rs` — wrap widget state for automatic re-render on change
- `Message` trait in `event/message.rs` — implement for each widget's message types (Pressed, Changed, etc.)
- `KeyBinding` in `event/keybinding.rs` — declare keyboard shortcuts per widget
- `TestApp`/`Pilot` in `testing/` — headless test harness with press/click/settle for interaction testing
- `insta::assert_snapshot!` — snapshot test pattern already established in tests/snapshot_tests.rs
- `AppContext.post_message()` — interior-mutable message posting for widget event emission
- `ComputedStyle` in `css/types.rs` — full property set for widget rendering (color, border, padding, etc.)

### Established Patterns
- Widget struct + Widget trait impl + default_css() — see irc_demo.rs examples (IrcScreen, ChannelList, etc.)
- `render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer)` — direct ratatui buffer writes
- `on_event(&self, event: &dyn Any, ctx: &AppContext) -> EventPropagation` — downcast to handle specific messages
- DFS tree traversal for rendering + parent chain walk for event bubbling
- `#[tokio::test]` with `TestApp::new(factory).pilot()` for async widget tests

### Integration Points
- `crates/textual-rs/src/widget/` — new widget files go here (label.rs, button.rs, input.rs, etc.)
- `crates/textual-rs/src/lib.rs` — pub mod widget needs to re-export all widget types
- `crates/textual-rs/src/css/cascade.rs` — DEFAULT_CSS collection from mounted widget types
- `crates/textual-rs/tests/` — snapshot and interaction tests for each widget
- `examples/` — update demo and irc_demo to use built-in widgets instead of ad-hoc structs

</code_context>

<specifics>
## Specific Ideas

- The Python Textual source in `textual/` is the primary reference for widget behavior, keyboard shortcuts, default CSS values, and message types. Each widget implementation should read the corresponding Python source before coding.
- Footer widget should discover key bindings from the focused widget's `key_bindings()` method and display them — this is why the binding table was designed as a static return value in Phase 3.
- The irc_demo and demo examples should be updated to use the new built-in widgets, replacing the ad-hoc widget structs. This is the first time the user will see real interactive widgets in the TUI.
- DataTable is the most complex widget — it needs column definitions, row data, sorting, scrolling, and cell selection. Reference Textual's DataTable closely.

</specifics>

<deferred>
## Deferred Ideas

- Virtual scrolling for large lists (v2 — simple offset is sufficient for v1)
- TextArea syntax highlighting (v2 — WIDGET-V2-02 in requirements)
- TextArea undo/redo (v2)
- ContentSwitcher widget (v2 — WIDGET-V2-01)
- DirectoryTree widget (v2 — WIDGET-V2-04)
- Table rendering in Markdown widget (v2)
- Interactive scrollbar (v2 — scrollbar is visual-only in v1)

</deferred>

---

*Phase: 04-built-in-widget-library*
*Context gathered: 2026-03-25*
