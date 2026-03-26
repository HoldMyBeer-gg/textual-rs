# Requirements: textual-rs

**Defined:** 2026-03-24
**Core Value:** Developers can build Textual-quality TUI applications in Rust — declare widgets, style with CSS, react to events, get a polished result on any terminal.

## v1 Requirements

### Foundation

- [x] **FOUND-01**: Library compiles on stable Rust (no nightly features)
- [x] **FOUND-02**: Cross-platform: Windows 10+, macOS, Linux (via crossterm backend)
- [x] **FOUND-03**: ratatui-based rendering pipeline with async Tokio event loop
- [x] **FOUND-04**: `App` struct as root entry point with `run()` method
- [x] **FOUND-05**: Alt-screen terminal management (enter/exit cleanly on panic)
- [x] **FOUND-06**: Terminal resize events trigger layout recomputation

### Widget Tree

- [x] **TREE-01**: Index-arena widget tree (`SlotMap<WidgetId, Box<dyn Widget>>`) with no unsafe parent pointers
- [ ] **TREE-02**: `Widget` trait with `render()`, `compose()`, `on_mount()`, `on_unmount()` lifecycle
- [x] **TREE-03**: `App > Screen > Widget` hierarchy with screen stack (push/pop for modals)
- [ ] **TREE-04**: Dynamic widget composition — widgets can add/remove children at runtime
- [x] **TREE-05**: Keyboard focus management with tab order traversal

### Layout Engine

- [x] **LAYOUT-01**: Taffy-backed layout engine supporting CSS Flexbox and CSS Grid
- [x] **LAYOUT-02**: Vertical and horizontal layout containers
- [x] **LAYOUT-03**: Grid layout with configurable rows/columns
- [x] **LAYOUT-04**: Dock layout (dock widgets to top/bottom/left/right edges)
- [x] **LAYOUT-05**: Fractional units (`1fr`, `2fr`) for proportional sizing
- [x] **LAYOUT-06**: Fixed, percentage, and auto sizing modes
- [x] **LAYOUT-07**: Dirty flag system — only recompute layout for changed subtrees

### CSS Styling Engine

- [x] **CSS-01**: TCSS parser (subset of CSS) using `cssparser` tokenizer
- [x] **CSS-02**: Type, class, and ID selector matching
- [x] **CSS-03**: Style cascade with CSS specificity rules
- [x] **CSS-04**: Inline styles on widget instances (highest specificity)
- [x] **CSS-05**: Pseudo-class states: `:focus`, `:hover`, `:disabled`
- [x] **CSS-06**: Supported properties: `color`, `background`, `border`, `border-title`, `padding`, `margin`, `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height`, `display`, `visibility`, `opacity`, `text-align`, `overflow`, `scrollbar-gutter`
- [x] **CSS-07**: Named colors, hex colors (`#rgb`, `#rrggbb`), `rgb()`, `rgba()` syntax
- [x] **CSS-08**: Border styles: `solid`, `rounded`, `heavy`, `double`, `ascii`, `none`
- [x] **CSS-09**: Default CSS defined on widget types, overridable by user stylesheets

### Reactive System

- [x] **REACT-01**: `Reactive<T>` property type that triggers re-render on change
- [x] **REACT-02**: `watch_` method convention: method called automatically when reactive property changes
- [x] **REACT-03**: `validate_` method convention: validate and coerce reactive property on set
- [x] **REACT-04**: `compute_` method convention: derive a property from one or more reactive sources
- [x] **REACT-05**: Render batching — multiple reactive changes in one tick produce one render pass

### Event System

- [x] **EVENT-01**: Typed message system — messages are Rust structs implementing `Message` trait
- [x] **EVENT-02**: `on_` method convention for message handling (e.g., `on_button_pressed`)
- [x] **EVENT-03**: Event bubbling — unhandled messages propagate up the widget tree
- [x] **EVENT-04**: Event stopping — handlers can consume a message to prevent bubbling
- [x] **EVENT-05**: Keyboard event routing to focused widget
- [x] **EVENT-06**: Mouse event routing with hit testing against rendered widget regions
- [x] **EVENT-07**: Key bindings — declare key bindings on widgets with action dispatch
- [x] **EVENT-08**: Timer/interval support for periodic updates

### Built-in Widgets

- [x] **WIDGET-01**: `Label` — static or reactive text display with markup support
- [x] **WIDGET-02**: `Button` — pressable button with label, variants (primary/warning/error/success)
- [x] **WIDGET-03**: `Input` — single-line text input with placeholder, password mode, validation
- [x] **WIDGET-04**: `TextArea` — multi-line text editor with line numbers option
- [x] **WIDGET-05**: `Checkbox` — toggleable boolean input
- [x] **WIDGET-06**: `Switch` — toggle switch (on/off)
- [x] **WIDGET-07**: `RadioButton` / `RadioSet` — mutually exclusive selection
- [x] **WIDGET-08**: `Select` — dropdown selection widget
- [x] **WIDGET-09**: `ListView` — scrollable list with selectable items
- [x] **WIDGET-10**: `DataTable` — tabular data display with sortable columns, scrolling
- [x] **WIDGET-11**: `Tree` — hierarchical tree view with expand/collapse
- [x] **WIDGET-12**: `ProgressBar` — determinate and indeterminate progress display
- [x] **WIDGET-13**: `Sparkline` — inline chart widget
- [x] **WIDGET-14**: `Log` — scrolling log display with auto-scroll
- [x] **WIDGET-15**: `Markdown` — rendered Markdown display
- [x] **WIDGET-16**: `Tabs` / `TabbedContent` — tabbed container navigation
- [x] **WIDGET-17**: `Collapsible` — expand/collapse container
- [x] **WIDGET-18**: `Vertical` / `Horizontal` — layout container widgets
- [x] **WIDGET-19**: `ScrollView` — scrollable container with optional scrollbars
- [x] **WIDGET-20**: `Header` — application header bar with title/subtitle
- [x] **WIDGET-21**: `Footer` — key binding help bar
- [x] **WIDGET-22**: `Placeholder` — development placeholder widget

### Testing Infrastructure

- [x] **TEST-01**: `TestApp` harness using ratatui `TestBackend` — no real terminal required
- [x] **TEST-02**: `Pilot` type for simulating key presses, mouse clicks, and focus changes
- [x] **TEST-03**: `settle().await` primitive that drains pending messages before assertions
- [x] **TEST-04**: Snapshot testing with `insta` for visual regression tests
- [x] **TEST-05**: `assert_buffer_lines()` and cell-level assertions for widget output
- [x] **TEST-06**: Property-based tests for CSS parser and layout engine using `proptest`

### Developer Experience

- [ ] **DX-01**: `#[derive(Widget)]` proc-macro for common widget boilerplate
- [x] **DX-02**: Worker API for running blocking tasks without blocking the event loop
- [x] **DX-03**: `notify()` / `post_message()` API for inter-widget communication
- [ ] **DX-04**: Application-level command palette support
- [ ] **DX-05**: Comprehensive documentation with examples matching Textual's guide structure

## v2 Requirements

### Enhanced Styling

- **STYLE-V2-01**: CSS custom properties (`--variable: value; color: var(--variable)`)
- **STYLE-V2-02**: CSS hot-reload during development
- **STYLE-V2-03**: Animation system for smooth property transitions

### Advanced Widgets

- **WIDGET-V2-01**: `ContentSwitcher` — switch between named content panes
- **WIDGET-V2-02**: Syntax-highlighted code display widget
- **WIDGET-V2-03**: Color picker widget
- **WIDGET-V2-04**: `DirectoryTree` — filesystem tree browser

### Deployment

- **DEPLOY-V2-01**: Web terminal support (xterm.js / WebSocket backend)
- **DEPLOY-V2-02**: Headless mode for testing in CI without a terminal

## Out of Scope

| Feature | Reason |
|---------|--------|
| Python bindings | Pure Rust library — separate project if needed |
| API compatibility with Python Textual | Inspired by, not identical — Rust idioms take precedence |
| WASM/web deployment | v2 — focus on native terminals first |
| Custom terminal backends beyond crossterm | v2 — termwiz has unstable API; termion is Unix-only |
| Full CSS 3 spec compliance | Only the TCSS subset Textual supports; full CSS is overkill for TUI |
| GPU-accelerated rendering | Terminal cells are the rendering primitive |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 through FOUND-06 | Phase 1 | Pending |
| TREE-01 through TREE-05 | Phase 2 | Pending |
| LAYOUT-01 through LAYOUT-07 | Phase 2 | Pending |
| CSS-01 through CSS-09 | Phase 2 | Pending |
| REACT-01 through REACT-05 | Phase 3 | Pending |
| EVENT-01 through EVENT-08 | Phase 3 | Pending |
| TEST-01 through TEST-06 | Phase 3 | Pending |
| WIDGET-01 through WIDGET-22 | Phase 4 | Complete (WIDGET-03 partial) |
| DX-01 through DX-05 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 62 total
- Mapped to phases: 62
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-24*
*Last updated: 2026-03-24 after initial definition*
