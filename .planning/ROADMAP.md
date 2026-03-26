# Roadmap: textual-rs

## Milestones

- **v1.0 MVP** - Phases 1-5 (shipped 2026-03-26)
- **v1.1 Visual Parity with Python Textual** - Phases 1-3 (in progress)

<details>
<summary>v1.0 MVP (Phases 1-5) - SHIPPED 2026-03-26</summary>

## v1.0 Phases

- [x] **Phase 1: Foundation** - Terminal layer, async event loop, and project scaffolding (completed 2026-03-25)
- [x] **Phase 2: Widget Tree, Layout, and Styling** - SlotMap widget arena, Taffy layout engine, and TCSS styling engine
- [x] **Phase 3: Reactive System, Events, and Testing** - Reactive properties, typed message passing, and TestApp harness (completed 2026-03-25)
- [x] **Phase 4: Built-in Widget Library** - All 22 v1 widgets with styling, interaction, and snapshot tests
- [x] **Phase 5: Developer Experience and Polish** - Proc-macro derive, Worker API, command palette, documentation (completed 2026-03-26)

### Phase 1: Foundation
**Goal**: A runnable Cargo workspace where `cargo run` opens a ratatui frame in the alternate screen, handles keyboard input, exits cleanly on `q` or panic, and responds to terminal resize.
**Depends on**: Nothing (first phase)
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06
**Success Criteria** (what must be TRUE):
  1. `cargo build` succeeds on stable Rust with no nightly features
  2. `cargo run` opens an alternate-screen TUI, renders visible content, and exits cleanly with `q`
  3. Panic in any code path restores the terminal to its original state
  4. Resizing the terminal window triggers a layout recomputation and re-render within one event tick
  5. The same binary produces correct output on Windows 10+, macOS, and Linux
**Plans**: 2 plans (complete)

### Phase 2: Widget Tree, Layout, and Styling
**Goal**: Developers can declare a widget tree with parent/child relationships, lay it out using Taffy Flexbox/Grid/Dock, and style widgets using a `.tcss` stylesheet.
**Depends on**: Phase 1
**Requirements**: TREE-01 through TREE-05, LAYOUT-01 through LAYOUT-07, CSS-01 through CSS-09
**Plans**: 4 plans (complete)

### Phase 3: Reactive System, Events, and Testing
**Goal**: Widget state changes trigger re-renders; typed messages bubble up the tree; keyboard/mouse events route correctly; TestApp/Pilot harness works.
**Depends on**: Phase 2
**Requirements**: REACT-01 through REACT-05, EVENT-01 through EVENT-08, TEST-01 through TEST-06
**Plans**: 3 plans (complete)

### Phase 4: Built-in Widget Library
**Goal**: All 22 v1 widgets implemented, styled, keyboard-interactive, and snapshot-tested.
**Depends on**: Phase 3
**Requirements**: WIDGET-01 through WIDGET-22
**Plans**: 9 plans (complete)

### Phase 5: Developer Experience and Polish
**Goal**: Ergonomic API with derive macros, Worker API, command palette, and documentation.
**Depends on**: Phase 4
**Requirements**: DX-01 through DX-05
**Plans**: 4 plans (complete)

</details>

## v1.1: Visual Parity with Python Textual

**Milestone Goal:** Make textual-rs rendering identical to Python Textual -- use the same half-block, sub-cell, and Unicode rendering techniques that give Textual its modern look. Semantic color theming, interactive state feedback, and per-widget visual polish.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [ ] **Phase 1: Semantic Theme Engine** - CSS variable resolution, shade generation, dark theme, custom theme support
- [ ] **Phase 2: Interactive States & Rendering Integration** - Focus/hover/active/selected/invalid visual feedback; sub-cell rendering primitives in real widgets
- [ ] **Phase 3: Widget Visual Polish & Demos** - Per-widget visual quality matching Python Textual; demo apps proving the full stack

## Phase Details

### Phase 1: Semantic Theme Engine
**Goal**: Widgets can reference semantic colors ($primary, $surface, $accent) in CSS and get correct RGB values with shade variants
**Depends on**: Nothing (first v1.1 phase; builds on existing CSS engine from v1.0)
**Requirements**: THEME-01, THEME-02, THEME-03, THEME-04
**Success Criteria** (what must be TRUE):
  1. A widget styled with `color: $primary` renders in the theme's primary color (not silently ignored or black)
  2. `$primary-lighten-2` and `$primary-darken-1` produce visibly distinct shades of the primary color
  3. The default dark theme produces colors matching Python Textual's textual-dark palette on visual comparison
  4. A user-provided CSS file can override theme variables and the change propagates to all widgets using those variables
**Plans**: TBD

Plans:
- [ ] 01-01: TBD
- [ ] 01-02: TBD

### Phase 2: Interactive States & Rendering Integration
**Goal**: Users see clear visual feedback for focus, hover, press, selection, and validation; sub-cell rendering primitives work correctly in real widget contexts
**Depends on**: Phase 1
**Requirements**: STATE-01, STATE-02, STATE-03, STATE-04, STATE-05, RENDER-01, RENDER-02, RENDER-03, RENDER-04, RENDER-05
**Success Criteria** (what must be TRUE):
  1. Pressing Tab to focus a widget shows a visible focus indicator (border color change or highlight) distinct from unfocused state
  2. Moving the mouse over a hoverable widget changes its appearance (color shift, border change, or highlight)
  3. Clicking and holding a Button shows a visually depressed state that reverts on release
  4. Selecting an item in a ListView or similar widget highlights it with accent color and bold text (not terminal REVERSE attribute)
  5. An Input field in invalid state shows red border/text; valid state shows normal or green indication
  6. Scrollable widgets show eighth-block scrollbar thumbs that move at sub-cell resolution
**Plans**: TBD

Plans:
- [ ] 02-01: TBD
- [ ] 02-02: TBD

### Phase 3: Widget Visual Polish & Demos
**Goal**: Individual widgets match Python Textual's visual quality; demo apps prove the full visual stack works together
**Depends on**: Phase 2
**Requirements**: VISUAL-01, VISUAL-02, VISUAL-03, VISUAL-04, VISUAL-05, VISUAL-06, VISUAL-07, DEMO-01, DEMO-02, DEMO-03
**Success Criteria** (what must be TRUE):
  1. Button renders with 3D depth effect (lighter top, darker bottom borders) that inverts visually on press
  2. Switch renders as a pill-shaped toggle with distinct knob/track colors
  3. DataTable shows zebra-striped rows, bold colored headers, and a smooth scrollbar
  4. Tabs show a colored underline/bar on the active tab clearly distinguishing it from inactive tabs
  5. Running the widget gallery demo produces output visually comparable to Python Textual's gallery screenshots
**Plans**: TBD
**UI hint**: yes

Plans:
- [ ] 03-01: TBD
- [ ] 03-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2026-03-25 |
| 2. Widget Tree, Layout, and Styling | v1.0 | 4/4 | Complete | 2026-03-26 |
| 3. Reactive System, Events, and Testing | v1.0 | 3/3 | Complete | 2026-03-25 |
| 4. Built-in Widget Library | v1.0 | 9/9 | Complete | 2026-03-26 |
| 5. Developer Experience and Polish | v1.0 | 4/4 | Complete | 2026-03-26 |
| 1. Semantic Theme Engine | v1.1 | 0/0 | Not started | - |
| 2. Interactive States & Rendering Integration | v1.1 | 0/0 | Not started | - |
| 3. Widget Visual Polish & Demos | v1.1 | 0/0 | Not started | - |
