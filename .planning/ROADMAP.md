# Roadmap: textual-rs

## Milestones

- **v1.0 MVP** - Phases 1-5 (shipped 2026-03-26)
- **v1.1 Visual Parity** - Phases 1-3 (shipped 2026-03-27)
- **v1.2 Production Readiness** - Phase 4 (shipped 2026-03-27)
- **v1.3 Widget Parity & Ship** - Phases 5-10

<details>
<summary>v1.0 MVP (Phases 1-5) - SHIPPED 2026-03-26</summary>

- [x] Phase 1: Foundation (2/2 plans)
- [x] Phase 2: Widget Tree, Layout, and Styling (4/4 plans)
- [x] Phase 3: Reactive System, Events, and Testing (3/3 plans)
- [x] Phase 4: Built-in Widget Library (9/9 plans)
- [x] Phase 5: Developer Experience and Polish (4/4 plans)

</details>

<details>
<summary>v1.1 Visual Parity (Phases 1-3) - SHIPPED 2026-03-27</summary>

- [x] Phase 1: Semantic Theme Engine (2/2 plans)
- [x] Phase 2: Interactive States & Rendering Integration (2/2 plans)
- [x] Phase 3: Widget Visual Polish & Demos (2/2 plans)

</details>

<details>
<summary>v1.2 Production Readiness (Phase 4) - SHIPPED 2026-03-27</summary>

- [x] Phase 4: Production Readiness (5/5 plans)

</details>

## v1.3 Widget Parity & Ship

### Phases

- [ ] **Phase 5: Screen Stack** - Push/pop/modal screen navigation with focus save/restore
- [ ] **Phase 6: Render-Only Foundation Widgets** - Static, Rule, Link, Pretty, Digits
- [ ] **Phase 7: List and Selection Widgets** - OptionList, SelectionList, ContentSwitcher
- [ ] **Phase 8: Enhanced Display Widgets** - RichLog, LoadingIndicator
- [ ] **Phase 9: Complex Widgets** - MaskedInput, DirectoryTree, Toast
- [x] **Phase 10: Platform Verification and Publish** - CI matrix, crates.io publish (completed 2026-03-28)

### Phase Details

#### Phase 5: Screen Stack
**Goal**: Developers can push, pop, and present modal screens with correct focus scoping
**Depends on**: Nothing (infrastructure phase)
**Requirements**: NAV-01, NAV-02, NAV-03
**Success Criteria** (what must be TRUE):
  1. Calling `ctx.push_screen()` places a new screen on top and redirects all keyboard focus to it
  2. Calling `ctx.pop_screen()` removes the top screen and restores focus to the exact widget that had focus before the push
  3. A `ModalScreen` blocks all keyboard and mouse input to screens below it while it is on top
  4. When a modal screen is dismissed, the screen below repaints cleanly with no render artifacts from the removed overlay
**Plans**: 3 plans
Plans:
- [ ] 05-01-PLAN.md — Multi-screen rendering with modal dimming + integration tests for all 4 success criteria
- [ ] 05-02-PLAN.md — push_screen_wait / pop_screen_with typed async result API
- [ ] 05-03-PLAN.md — tutorial_06_screens example + human verification
**UI hint**: yes

#### Phase 6: Render-Only Foundation Widgets
**Goal**: Developers can use five display-only widgets that form the visual vocabulary of any app
**Depends on**: Phase 5
**Requirements**: WIDGET-01, WIDGET-02, WIDGET-03, WIDGET-04, WIDGET-05
**Success Criteria** (what must be TRUE):
  1. `Static` displays a styled text string; `Link` displays as a Static but emits a message and opens a URL when clicked
  2. `Rule` renders a full-width horizontal or full-height vertical separator line in a chosen style
  3. `Pretty` displays a `serde_json::Value` (or any `Debug`-formattable value) with syntax-colored output
  4. `Digits` renders a numeric string as large block-character digits matching Python Textual's visual output
**Plans**: TBD

#### Phase 7: List and Selection Widgets
**Goal**: Developers can present single-select and multi-select lists, and switch between named content panes
**Depends on**: Phase 6
**Requirements**: WIDGET-06, WIDGET-07, WIDGET-08
**Success Criteria** (what must be TRUE):
  1. `OptionList` shows a scrollable list; pressing Enter or clicking an item emits an `OptionSelected` message
  2. `SelectionList` shows a checkboxable list; Space toggles selection and the widget exposes the current selected set
  3. `ContentSwitcher` shows exactly one of its named child panes at a time; switching panes does not leave ghost content from the hidden pane
  4. All three widgets respond correctly to keyboard navigation (arrow keys, Home, End, Page Up/Down)
**Plans**: TBD
**UI hint**: yes

#### Phase 8: Enhanced Display Widgets
**Goal**: Developers can display scrolling styled log output and overlay a loading spinner on any in-progress widget
**Depends on**: Phase 7
**Requirements**: WIDGET-09, WIDGET-10
**Success Criteria** (what must be TRUE):
  1. `RichLog` accepts styled `Line` objects, scrolls to the bottom on new entries, and evicts old lines when `max_lines` is reached
  2. Setting `widget.loading = true` on any widget overlays a spinner animation on that widget; setting it back to `false` removes the spinner and restores the widget
  3. `LoadingIndicator` as a standalone widget also renders the spinner, gated by `skip_animations` for deterministic snapshot tests
**Plans**: 2 plans
Plans:
- [ ] 08-01-PLAN.md — RichLog widget with styled Line storage, auto-scroll, max_lines eviction
- [ ] 08-02-PLAN.md — LoadingIndicator standalone widget + per-widget loading overlay system
**UI hint**: yes

#### Phase 9: Complex Widgets
**Goal**: Developers can use a masked text input, a filesystem browser, and toast notifications
**Depends on**: Phase 8
**Requirements**: WIDGET-11, WIDGET-12, WIDGET-13
**Success Criteria** (what must be TRUE):
  1. `MaskedInput` accepts a format template (e.g. `##/##/####`); the cursor skips separator characters and Backspace removes only user-typed characters without drifting position
  2. `DirectoryTree` renders a filesystem tree; expanding a directory node lazy-loads its children via a worker without blocking the UI tick
  3. `DirectoryTree` never enters an infinite loop on symlinked directories
  4. Calling `app.notify("message", severity, timeout)` displays a toast in the bottom-right corner; multiple rapid `notify()` calls stack visually and each auto-dismisses on its own timer
**Plans**: 3 plans
Plans:
- [ ] 09-01-PLAN.md — MaskedInput widget with format mask enforcement and raw-space cursor tracking
- [ ] 09-02-PLAN.md — DirectoryTree widget with lazy-loaded filesystem browsing and symlink safety
- [x] 09-03-PLAN.md — Toast notification system with stacking, auto-dismiss, and severity colors
**UI hint**: yes

#### Phase 10: Platform Verification and Publish
**Goal**: The library builds and passes tests on all three platforms and is published to crates.io
**Depends on**: Phase 9
**Requirements**: PLATFORM-01, PUBLISH-01, PUBLISH-02, PUBLISH-03
**Success Criteria** (what must be TRUE):
  1. CI runs the full test suite on macOS and Linux (in addition to Windows) and all tests pass on all three platforms
  2. Every public API item has a rustdoc comment; `cargo doc --no-deps` produces no warnings
  3. `cargo package --list` includes `README.md` and produces no broken-path warnings
  4. The crate is successfully published to crates.io and `cargo add textual-rs` resolves the correct version
**Plans**: 4 plans
Plans:
- [x] 10-01-PLAN.md — Fix CI action references, arboard safety check, three-platform matrix
- [x] 10-02-PLAN.md — Rustdoc for core infrastructure modules (~31 files)
- [x] 10-03-PLAN.md — Cargo manifest fixes, version 0.3.0 bump, and publish to crates.io
- [x] 10-04-PLAN.md — Rustdoc for all widget modules (~32 files)

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1. Foundation | v1.0 | 2/2 | Complete | 2026-03-25 |
| 2. Widget Tree, Layout, Styling | v1.0 | 4/4 | Complete | 2026-03-26 |
| 3. Reactive System, Events, Testing | v1.0 | 3/3 | Complete | 2026-03-25 |
| 4. Built-in Widget Library | v1.0 | 9/9 | Complete | 2026-03-26 |
| 5. Developer Experience, Polish | v1.0 | 4/4 | Complete | 2026-03-26 |
| 1. Semantic Theme Engine | v1.1 | 2/2 | Complete | 2026-03-26 |
| 2. Interactive States & Rendering | v1.1 | 2/2 | Complete | 2026-03-26 |
| 3. Widget Visual Polish & Demos | v1.1 | 2/2 | Complete | 2026-03-27 |
| 4. Production Readiness | v1.2 | 5/5 | Complete | 2026-03-27 |
| 5. Screen Stack | v1.3 | 0/3 | Not started | - |
| 6. Render-Only Foundation Widgets | v1.3 | 0/? | Not started | - |
| 7. List and Selection Widgets | v1.3 | 0/? | Not started | - |
| 8. Enhanced Display Widgets | v1.3 | 0/2 | Not started | - |
| 9. Complex Widgets | v1.3 | 1/3 | In Progress|  |
| 10. Platform Verification and Publish | v1.3 | 4/4 | Complete    | 2026-03-28 |
