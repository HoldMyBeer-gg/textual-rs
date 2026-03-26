# Roadmap: textual-rs

## Overview

textual-rs delivers a Textual-quality TUI framework for Rust, built on top of ratatui and
crossterm. The five phases follow a strict dependency order: the terminal infrastructure and
async event loop must exist before the widget tree, which must be stable before the CSS and
reactive layers can be layered on top, and the test harness must be solid before the built-in
widget library can be verified. Developer experience polish comes last, when the full stack is
proven. Every phase closes with observable, user-verifiable outcomes — not just "code written."

## Milestone: v1.0

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** - Terminal layer, async event loop, and project scaffolding (completed 2026-03-25)
- [ ] **Phase 2: Widget Tree, Layout, and Styling** - SlotMap widget arena, Taffy layout engine, and TCSS styling engine
- [x] **Phase 3: Reactive System, Events, and Testing** - Reactive<T> properties, typed message passing, and TestApp harness (completed 2026-03-25)
- [ ] **Phase 4: Built-in Widget Library** - All 22 v1 widgets with styling, interaction, and snapshot tests
- [ ] **Phase 5: Developer Experience and Polish** - Proc-macro derive, Worker API, command palette, documentation

## Phase Details

### Phase 1: Foundation
**Goal**: A runnable Cargo workspace where `cargo run` opens a ratatui frame in the alternate screen, handles keyboard input, exits cleanly on `q` or panic, and responds to terminal resize — proving the full ratatui + crossterm + Tokio stack works end-to-end on Windows, macOS, and Linux.
**Depends on**: Nothing (first phase)
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06
**Success Criteria** (what must be TRUE):
  1. `cargo build` succeeds on stable Rust with no nightly features
  2. `cargo run` opens an alternate-screen TUI, renders visible content, and exits cleanly with `q`
  3. Panic in any code path restores the terminal to its original state (no broken shell)
  4. Resizing the terminal window triggers a layout recomputation and re-render within one event tick
  5. The same binary produces correct output on Windows 10+, macOS, and Linux with no platform branches in application code
**Plans**: 2 plans

Plans:
- [x] 01-01: Cargo workspace setup, dependencies, basic App skeleton, Tokio LocalSet event loop with crossterm EventStream and flume channel
- [x] 01-02: Terminal management — alt-screen entry/exit, raw mode, panic hook with terminal restore, resize event handling, TestBackend integration

### Phase 2: Widget Tree, Layout, and Styling
**Goal**: Developers can declare a widget tree (`App > Screen > Widget`) with parent/child relationships, lay it out using Taffy Flexbox/Grid/Dock, and style widgets using a `.tcss` stylesheet — and see the correct visual result rendered via ratatui.
**Depends on**: Phase 1
**Requirements**: TREE-01, TREE-02, TREE-03, TREE-04, TREE-05, LAYOUT-01, LAYOUT-02, LAYOUT-03, LAYOUT-04, LAYOUT-05, LAYOUT-06, LAYOUT-07, CSS-01, CSS-02, CSS-03, CSS-04, CSS-05, CSS-06, CSS-07, CSS-08, CSS-09
**Success Criteria** (what must be TRUE):
  1. A widget added via `compose()` appears on screen; a widget removed via `unmount()` disappears — without unsafe code or runtime borrow panics
  2. Pressing Tab moves keyboard focus through widgets in declared tab order; focused widget receives `:focus` pseudo-class styling
  3. A layout with `dock: top`, `dock: bottom`, and a flex-column center region renders correctly at multiple terminal sizes with fractional (`1fr`, `2fr`) sizing
  4. A `.tcss` stylesheet with type, class, and ID selectors applies correct cascade and specificity — inline styles win over class styles, which win over type styles
  5. Border styles (`solid`, `rounded`, `heavy`, `double`, `ascii`), padding, color, and background properties render correctly when declared in TCSS
**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md — Widget tree: SlotMap arena, Widget trait, AppContext, CSS type definitions, screen stack, focus management
- [x] 02-02-PLAN.md — Layout engine: TaffyBridge, ComputedStyle-to-Taffy conversion, flex/grid/dock layouts, dirty-flag incremental relayout, mouse hit map
- [x] 02-03-PLAN.md — CSS/TCSS styling engine: cssparser tokenizer, selector parser/matcher, property parser, cascade resolver, pseudo-classes, default CSS
- [ ] 02-04-PLAN.md — Render loop integration: wire AppContext + TaffyBridge + Stylesheet into App, IRC demo example

**UI hint**: yes

### Phase 3: Reactive System, Events, and Testing
**Goal**: Widget state changes automatically trigger re-renders; typed messages bubble up the tree and are handled via `on_` methods; keyboard/mouse events route to the correct widget; and a `TestApp`/`Pilot` harness lets tests simulate user interaction with no real terminal.
**Depends on**: Phase 2
**Requirements**: REACT-01, REACT-02, REACT-03, REACT-04, REACT-05, EVENT-01, EVENT-02, EVENT-03, EVENT-04, EVENT-05, EVENT-06, EVENT-07, EVENT-08, TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, TEST-06
**Success Criteria** (what must be TRUE):
  1. Mutating a `Reactive<T>` field on a widget triggers exactly one re-render per tick, even when multiple reactive fields change in the same tick
  2. A `Button::Pressed` message emitted by a child widget is received by an `on_button_pressed` handler on any ancestor widget; stopping propagation prevents further bubbling
  3. A key bound via `key_bindings` on a widget fires the declared action when that key is pressed while the widget has focus
  4. `TestApp::new(app).pilot()` runs the app headlessly; `pilot.press(Key::Tab).await` and `settle().await` produce the expected focused widget without a real terminal
  5. `assert_snapshot!` captures the rendered buffer and fails the test when the widget output changes unexpectedly
**Plans**: 3 plans

Plans:
- [x] 03-01-PLAN.md — Reactive property system: Reactive<T>, ComputedReactive<T>, reactive_graph integration, Executor/Owner init, RenderRequest batching
- [x] 03-02-PLAN.md — Event system: Message trait, on_event dispatch, bubbling, keyboard/mouse routing, key bindings, timer/interval
- [x] 03-03-PLAN.md — Test infrastructure: TestApp/Pilot harness, settle(), insta snapshots, assert_buffer_lines, proptest CSS fuzzing

**Research note — Phase 3 planning requires a spike:** `reactive_graph` + Tokio `LocalSet` integration has MEDIUM confidence. Verify `Executor::init_tokio()` works with `LocalSet` and that effects can be debounced into a single render tick before committing to the API design. Run this spike before `/gsd:plan-phase 3`.

### Phase 4: Built-in Widget Library
**Goal**: All 22 v1 widgets are implemented, styled via TCSS, keyboard-interactive where applicable, and covered by snapshot tests — making textual-rs usable as a complete application framework.
**Depends on**: Phase 3
**Requirements**: WIDGET-01, WIDGET-02, WIDGET-03, WIDGET-04, WIDGET-05, WIDGET-06, WIDGET-07, WIDGET-08, WIDGET-09, WIDGET-10, WIDGET-11, WIDGET-12, WIDGET-13, WIDGET-14, WIDGET-15, WIDGET-16, WIDGET-17, WIDGET-18, WIDGET-19, WIDGET-20, WIDGET-21, WIDGET-22
**Success Criteria** (what must be TRUE):
  1. Each of the 22 widgets renders correctly in a `TestApp` snapshot test with default TCSS styles
  2. Interactive widgets (Input, TextArea, Button, Checkbox, Switch, RadioSet, Select, ListView, DataTable, Tree, Tabs, Collapsible) respond correctly to keyboard events delivered via `Pilot`
  3. ScrollView, ListView, DataTable, and Tree widgets scroll their content when content exceeds the available area
  4. Each widget emits the documented messages (e.g., `Button::Pressed`, `Input::Changed`) when user interaction occurs, verifiable via TestApp message capture
**Plans**: 7 plans

Plans:
- [ ] 04-01-PLAN.md — Deps + infrastructure + Label, Button, Checkbox, Switch widgets
- [ ] 04-02-PLAN.md — Input + RadioButton/RadioSet widgets
- [ ] 04-03-PLAN.md — TextArea + Select widgets
- [ ] 04-04-PLAN.md — Vertical/Horizontal, Header, Footer, Placeholder, ProgressBar, Sparkline widgets
- [ ] 04-05-PLAN.md — ListView, Log, ScrollView widgets
- [ ] 04-06-PLAN.md — DataTable, Tree widgets
- [ ] 04-07-PLAN.md — Tabs/TabbedContent, Collapsible, Markdown widgets

**UI hint**: yes

### Phase 5: Developer Experience and Polish
**Goal**: The framework API is ergonomic enough to build real applications — a `#[derive(Widget)]` macro reduces boilerplate, the Worker API handles background tasks cleanly, `notify()`/`post_message()` enables inter-widget communication, a command palette is available, and the documentation is complete enough for a new user to build a Textual-quality app from the guide.
**Depends on**: Phase 4
**Requirements**: DX-01, DX-02, DX-03, DX-04, DX-05
**Success Criteria** (what must be TRUE):
  1. `#[derive(Widget)]` on a struct generates the Widget trait boilerplate, compiles without errors, and is recognized by rust-analyzer without IDE red-underlines
  2. `self.run_worker(async_fn)` executes a blocking or async task without blocking the event loop; the result arrives as a typed message handled by `on_` dispatch
  3. A widget can call `self.notify(message)` to post to ancestors and `app.post_message(target_id, message)` to post to any widget by ID
  4. An app with command palette enabled shows a searchable command list when the palette key binding is triggered, and executing a command dispatches the correct action
  5. A developer new to textual-rs can follow the documentation guide and produce a working multi-screen TUI with styled widgets and event handlers

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 2/2 | Complete    | 2026-03-25 |
| 2. Widget Tree, Layout, and Styling | 3/4 | In Progress|  |
| 3. Reactive System, Events, and Testing | 3/3 | Complete   | 2026-03-25 |
| 4. Built-in Widget Library | 0/7 | Not started | - |
| 5. Developer Experience and Polish | 0/2 | Not started | - |
