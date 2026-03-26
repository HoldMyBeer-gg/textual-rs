# Phase 2: Widget Tree, Layout, and Styling - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Developers can declare a widget tree (`App > Screen > Widget`) with parent/child relationships, lay it out using Taffy Flexbox/Grid/Dock, and style widgets using a `.tcss` stylesheet — and see the correct visual result rendered via ratatui. This phase delivers the widget arena, layout engine integration, and CSS styling engine. Reactive properties, event/message passing, and the test harness are Phase 3.

</domain>

<decisions>
## Implementation Decisions

### Widget API & Borrow Pattern
- **D-01:** AppContext pattern for arena access. Widget methods receive `(&self, ctx: &AppContext)` or `(&self, ctx: &mut AppContext)` instead of holding `&mut self` and `&mut Arena` simultaneously. AppContext owns the SlotMap arena, SecondaryMaps for children/parent/styles/dirty flags. This eliminates the SlotMap borrow conflict entirely.
- **D-02:** Direct ratatui Buffer rendering. `Widget::render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer)` writes to ratatui's Buffer directly. Widgets can use any ratatui widget internally (Block, Paragraph, etc.). No intermediate Strip representation — zero-copy, leverages the full ratatui ecosystem.
- **D-03:** `compose()` returns `Vec<Box<dyn Widget>>`. Simple, idiomatic Rust — no generator/yield needed. AppContext inserts returned widgets into the arena and wires parent/child relationships. Default implementation returns empty vec (leaf widget).
- **D-04:** `dyn Widget` everywhere. All widgets (built-in and user-defined) are `Box<dyn Widget>` in the arena. No enum dispatch for built-ins. Standard extensible type hierarchy pattern.

### Layout-to-Render Bridge
- **D-05:** TaffyBridge sync layer. A dedicated `TaffyBridge` struct owns the `TaffyTree<WidgetId>` and maintains a `HashMap<WidgetId, NodeId>` mapping. It syncs the Taffy tree to match the widget arena, converts `ComputedStyle` to Taffy `Style`, and after `compute_layout()` converts Taffy's f32 positions/sizes to ratatui `Rect` (u16 cells) with rounding.
- **D-06:** Dock layout emulated with nested Flexbox. `dock: top/bottom/left/right` declarations compile down to nested Flexbox containers in the TaffyBridge. No custom layout pass — Taffy's flex engine handles everything.
- **D-07:** Dirty-flag bubbling for incremental relayout. When a widget is marked dirty, its ancestors are marked dirty up to the Screen. Next render pass checks dirty flags, re-syncs only dirty subtrees to Taffy, recomputes layout from the highest dirty ancestor, then clears flags.

### CSS Parser Scope & Property Set
- **D-08:** cssparser crate (Mozilla's tokenizer) + hand-rolled SelectorParser and PropertyParser. The cssparser crate handles tokenization (battle-tested edge case handling). Selector matching and property parsing are built by hand on top of the token stream.
- **D-09:** Selector support matches Python Textual: type (`Button`), class (`.highlight`), ID (`#sidebar`), pseudo-class (`:focus`, `:hover`, `:disabled`), descendant combinator (`Screen Label`), child combinator (`Container > Button`). No sibling combinators, no pseudo-elements in v1.
- **D-10:** Style cascade with CSS specificity rules. Inline styles > ID > class > type. Standard CSS specificity calculation.
- **D-11:** DEFAULT_CSS as static `&'static str` on the Widget trait. Each widget type defines `fn default_css() -> &'static str`. The stylesheet loader collects all DEFAULT_CSS from mounted widget types and prepends them at lowest specificity before user stylesheets.
- **D-12:** Full TCSS property set per requirements CSS-06: color, background, border, border-title, padding, margin, width, height, min-width, min-height, max-width, max-height, display, visibility, opacity, text-align, overflow, scrollbar-gutter. Plus border styles (solid, rounded, heavy, double, ascii, none) and color syntax (named, hex, rgb(), rgba()).

### Screen Stack & Focus Model
- **D-13:** Screens are special WidgetIds in the shared arena. App maintains a `Vec<WidgetId>` as the screen stack. Pushing a screen inserts it + its children into the arena. Popping removes the subtree. Only the top screen renders and receives input.
- **D-14:** DOM-order focus traversal with `can_focus()` flag. Tab follows depth-first tree order. Widgets opt in via `fn can_focus(&self) -> bool` (default false). Focus state (`focused_widget: Option<WidgetId>`) lives on AppContext. Setting focus updates `:focus` pseudo-class on the widget.

### Claude's Discretion
- Exact `Widget` trait method signatures beyond render/compose/can_focus (on_mount, on_unmount lifecycle methods)
- AppContext internal data structure choices (DenseSlotMap vs SlotMap, HashMap vs SecondaryMap for node_map)
- CSS property parsing internals (how property values are stored, enum vs typed structs)
- TaffyBridge sync algorithm details (incremental vs full rebuild)
- Error types and error handling strategy for CSS parse errors
- EventPropagation enum design (for Phase 3, but the type may be defined here)
- Mouse hit map implementation (col x row -> WidgetId lookup)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python Textual Reference (architecture inspiration)
- `textual/src/textual/widget.py` — Widget base class: render(), compose(), lifecycle, reactive integration
- `textual/src/textual/dom.py` — DOMNode: parent/child tree, CSS query, selector matching
- `textual/src/textual/screen.py` — Screen class: focus management, widget discovery, layout updates
- `textual/src/textual/app.py` — App class: screen stack, mode system, driver initialization
- `textual/src/textual/css/parse.py` — CSS selector/declaration parser
- `textual/src/textual/css/stylesheet.py` — Stylesheet class, rule management, specificity
- `textual/src/textual/css/styles.py` — Computed styles per widget
- `textual/src/textual/css/match.py` — Selector matching logic
- `textual/src/textual/layout.py` — Layout interface, WidgetPlacement
- `textual/src/textual/layouts/` — Vertical, Horizontal, Grid layout implementations
- `textual/src/textual/_compositor.py` — Render pipeline, dirty region tracking

### Existing Rust Code (Phase 1 output)
- `crates/textual-rs/src/app.rs` — Current App struct, run(), render_frame(), event loop — Phase 2 replaces the skeleton render with widget tree rendering
- `crates/textual-rs/src/event.rs` — AppEvent enum — Phase 2 extends with widget-relevant events
- `crates/textual-rs/src/terminal.rs` — TerminalGuard, panic hook — unchanged in Phase 2
- `crates/textual-rs/src/lib.rs` — Module exports — Phase 2 adds widget, layout, css modules

### Project Planning Documents
- `.planning/REQUIREMENTS.md` — Phase 2 covers TREE-01 through TREE-05, LAYOUT-01 through LAYOUT-07, CSS-01 through CSS-09
- `.planning/ROADMAP.md` — Phase 2 plan details (02-01, 02-02, 02-03) and success criteria
- `.planning/PROJECT.md` — Key decisions: SlotMap arena, Taffy layout engine, ratatui rendering

### Codebase Analysis
- `.planning/codebase/ARCHITECTURE.md` — Python Textual architecture: event loop, message pump, compositor, layout
- `.planning/codebase/STRUCTURE.md` — Python Textual file organization and module purposes
- `.planning/codebase/CONVENTIONS.md` — Naming patterns, reactive descriptors, message handlers, CSS integration
- `.planning/research/CSS_LAYOUT.md` — CSS layout research (if exists)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App::run()` / `App::run_async()` in `crates/textual-rs/src/app.rs` — Event loop with flume channel. Phase 2 extends this to dispatch render passes through the widget tree instead of the hard-coded demo render.
- `App::render_frame()` — Generic over Backend. Phase 2 replaces the body with widget-tree-based rendering via AppContext + TaffyBridge.
- `TerminalGuard` in `crates/textual-rs/src/terminal.rs` — RAII terminal management. Unchanged.
- `AppEvent` enum in `crates/textual-rs/src/event.rs` — Phase 2 may add layout/style-related events (e.g., `StyleChanged`, `LayoutComplete`).

### Established Patterns
- Tokio LocalSet + flume channel for event bus — all Phase 2 code runs on the same LocalSet, no Send requirements on widget state.
- `anyhow::Result` for error propagation — continue using for Phase 2 public API.
- ratatui `Terminal<B>` with generic Backend — maintains TestBackend compatibility.

### Integration Points
- `App::run_async()` main loop — Phase 2 replaces the match arms with: relayout_if_needed() -> render_widget_tree() -> handle_input_to_focused_widget().
- `App::render()` static method — replaced by AppContext-driven tree render that walks the active screen's widget subtree.
- flume channel — Phase 2 may add AppEvent variants for widget lifecycle events (mount, unmount, focus change).

</code_context>

<specifics>
## Specific Ideas

- The AppContext pattern is the core architectural decision — it resolves the SlotMap borrow problem that was flagged as a spike requirement in the roadmap. No spike needed; the pattern is well-understood.
- Dock layout via nested flex avoids a custom layout algorithm while still giving users the familiar `dock: top` / `dock: bottom` API from Python Textual.
- Direct ratatui Buffer rendering means every ratatui widget (Block, Paragraph, List, Table, etc.) is usable inside textual-rs widgets with zero wrapping. This is a major ecosystem advantage over building an intermediate representation.
- DEFAULT_CSS as `&'static str` means widget default styles are compiled into the binary — no file I/O, no runtime parsing overhead for defaults (they're parsed once at app startup).
- Phase 2 should end with layout demo examples that prove the engine works for real-world apps. Key demo: an IRC client layout (weechat-style) with channel list sidebar, main chat text area, user list sidebar, input bar at bottom, title bar at top, and context menu overlay. This exercises dock layout, multi-pane flex, and nested containers simultaneously.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-widget-tree-layout-and-styling*
*Context gathered: 2026-03-25*
