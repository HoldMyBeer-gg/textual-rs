# Requirements: textual-rs

**Defined:** 2026-03-24 (v1.0), updated 2026-03-26 (v1.1, v1.2)
**Core Value:** Developers can build Textual-quality TUI applications in Rust -- declare widgets, style with CSS, react to events, get a polished result on any terminal.

## v1.1: Visual Parity with Python Textual

### Rendering Primitives

- [ ] **RENDER-01**: McGugan Box borders render correctly with independent inside/outside/border colors on all supported terminals
- [ ] **RENDER-02**: Braille characters (2x4 dots/cell) used in Sparkline multi-row mode produce smooth, visually correct curves
- [ ] **RENDER-03**: Quadrant characters (2x2 pixels/cell) available and used for smoother UI elements
- [ ] **RENDER-04**: Eighth-block scrollbars render with sub-cell thumb positioning in all scrollable widgets
- [ ] **RENDER-05**: Half-block gradient fills used for backgrounds where depth/layering is needed

### Semantic Color Theming

- [ ] **THEME-01**: CSS variables ($primary, $secondary, $accent, $surface, $panel, $background, $success, $warning, $error, $foreground) resolve to concrete RGB values during cascade
- [x] **THEME-02**: Shade generation (-lighten-1/2/3, -darken-1/2/3) works on any color variable
- [x] **THEME-03**: A default dark theme ships matching Python Textual's textual-dark palette
- [ ] **THEME-04**: User can provide a custom theme via CSS

### Widget Visual Quality

- [ ] **VISUAL-01**: Button renders with 3D depth -- lighter top border, darker bottom; inverted on press
- [x] **VISUAL-02**: Switch renders as pill-shaped toggle with distinct knob/track colors
- [ ] **VISUAL-03**: DataTable has zebra-striped rows, bold colored headers, smooth scrollbar
- [x] **VISUAL-04**: Tabs show active indicator (underline/color bar) distinguishing selected from inactive
- [x] **VISUAL-05**: Markdown renders headings in accent colors, code blocks with dark bg, links colored+underlined
- [x] **VISUAL-06**: Placeholder renders with cross-hatch/textured background pattern
- [x] **VISUAL-07**: Footer key badges use high-contrast accent colors

### Interactive States

- [ ] **STATE-01**: Every focusable widget shows visible focus indicator when focused
- [ ] **STATE-02**: Hover state changes widget appearance on mouse-over
- [ ] **STATE-03**: Active/pressed state on Button shows visual depression during click
- [ ] **STATE-04**: Selected items in list widgets use accent color + bold (not REVERSED)
- [ ] **STATE-05**: Invalid Input fields show red border/text; valid show normal/green

### Demo Quality

- [x] **DEMO-01**: Demo UI visually comparable to Python Textual widget gallery
- [x] **DEMO-02**: IRC demo renders professional-looking client with clear visual hierarchy
- [x] **DEMO-03**: Both demos use McGugan Box borders with proper background depth

## v1.2: Production Readiness

### Critical Bugs & Gaps (Must Have)

- [ ] **PROD-01**: Clipboard integration via arboard — Ctrl+C/V/X wired to Input and TextArea
- [ ] **PROD-02**: Text selection in Input and TextArea — Shift+arrow keys, Shift+Home/End
- [ ] **PROD-03**: Select overlay and CommandPalette use active_overlay pattern (no screen blank)
- [ ] **PROD-04**: CSS $variable in border shorthand — `border: tall $primary` resolves correctly
- [ ] **PROD-05**: McGugan Box fallback for terminals without U+1FB87 — use U+2595 instead
- [x] **PROD-06**: Terminal capability detection — color depth, Unicode support, mouse
- [x] **PROD-07**: Resize reflow — full layout recomputation on terminal resize

### Important Features (Should Have)

- [ ] **PROD-08**: Animation system — Tween with easing, used by Switch toggle and Tab underline
- [ ] **PROD-09**: text-align CSS property actually centers content in widget render
- [ ] **PROD-10**: Horizontal mouse wheel scrolling dispatches scroll_left/scroll_right
- [ ] **PROD-11**: Ctrl+C as copy (not terminal interrupt) when text widget has selection
- [ ] **PROD-12**: Worker progress reporting — workers can send incremental updates
- [ ] **PROD-13**: Hot-reload .tcss files — poll for changes, re-cascade on modification

### Polish (Nice to Have)

- [ ] **PROD-14**: Light theme + runtime theme switching via Ctrl+T
- [ ] **PROD-15**: 5+ built-in themes (tokyo-night, nord, gruvbox, dracula, catppuccin)
- [ ] **PROD-16**: hatch CSS property — background pattern fills
- [ ] **PROD-17**: keyline CSS property — grid lines between child widgets
- [ ] **PROD-18**: Image rendering via half-block canvas
- [ ] **PROD-19**: Syntax highlighting in Markdown code blocks

## v1.3: Widget Parity & Ship

### Widgets

- [ ] **WIDGET-01**: User can display static text content with Static widget (base for Label/Link)
- [ ] **WIDGET-02**: User can render a horizontal or vertical Rule separator
- [ ] **WIDGET-03**: User can render a clickable Link that emits a message on press
- [ ] **WIDGET-04**: User can display formatted data structures with Pretty widget (JSON/Debug via serde_json)
- [ ] **WIDGET-05**: User can display large numbers as block-character Digits
- [ ] **WIDGET-06**: User can select from a list of options with OptionList (keyboard + mouse navigation)
- [ ] **WIDGET-07**: User can select multiple items from a list with SelectionList (checkboxes)
- [ ] **WIDGET-08**: User can switch between named content panes with ContentSwitcher
- [x] **WIDGET-09**: User can view scrolling rich-text log output with RichLog (styled Lines, not plain strings)
- [x] **WIDGET-10**: User can display a loading spinner on any widget via `widget.loading = true`
- [ ] **WIDGET-11**: User can enter text with a format mask using MaskedInput (e.g. date, phone)
- [ ] **WIDGET-12**: User can browse a filesystem tree with DirectoryTree (lazy-loaded, async)
- [x] **WIDGET-13**: User receives toast notifications via `app.notify(message, severity, timeout)`

### Navigation

- [ ] **NAV-01**: Developer can push a new screen onto the stack with `ctx.push_screen()`
- [ ] **NAV-02**: Developer can pop the top screen and restore focus to the underlying screen
- [ ] **NAV-03**: Developer can create modal screens that block input to screens below

### Platform

- [x] **PLATFORM-01**: Library builds and all tests pass on macOS and Linux (CI verified)

### Publish

- [ ] **PUBLISH-01**: Library is published to crates.io with correct README, docs, and semver metadata
- [ ] **PUBLISH-02**: All public API items have rustdoc documentation
- [ ] **PUBLISH-03**: `cargo package --list` produces a clean, complete package with no broken paths

## Future Requirements

### Widgets

- **WIDGET-F01**: Modes system (independent screen stacks per named section)
- **WIDGET-F02**: push_screen_wait (async callback on pop)
- **WIDGET-F03**: widget.loading viral base-class property

## Out of Scope

| Feature | Reason |
|---------|--------|
| Web/WASM deployment | Native terminals first |
| Python bindings | Pure Rust library |
| Direct API compatibility with Python Textual | Inspired by, not identical |
| Accessibility / screen reader support | Future consideration |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| THEME-01 | Phase 1 | Pending |
| THEME-02 | Phase 1 | Complete |
| THEME-03 | Phase 1 | Complete |
| THEME-04 | Phase 1 | Pending |
| STATE-01 | Phase 2 | Pending |
| STATE-02 | Phase 2 | Pending |
| STATE-03 | Phase 2 | Pending |
| STATE-04 | Phase 2 | Pending |
| STATE-05 | Phase 2 | Pending |
| RENDER-01 | Phase 2 | Pending |
| RENDER-02 | Phase 2 | Pending |
| RENDER-03 | Phase 2 | Pending |
| RENDER-04 | Phase 2 | Pending |
| RENDER-05 | Phase 2 | Pending |
| VISUAL-01 | Phase 3 | Pending |
| VISUAL-02 | Phase 3 | Complete |
| VISUAL-03 | Phase 3 | Pending |
| VISUAL-04 | Phase 3 | Complete |
| VISUAL-05 | Phase 3 | Complete |
| VISUAL-06 | Phase 3 | Complete |
| VISUAL-07 | Phase 3 | Complete |
| DEMO-01 | Phase 3 | Complete |
| DEMO-02 | Phase 3 | Complete |
| DEMO-03 | Phase 3 | Complete |
| PROD-01 | Phase 4 | Pending |
| PROD-02 | Phase 4 | Pending |
| PROD-03 | Phase 4 | Pending |
| PROD-04 | Phase 4 | Pending |
| PROD-05 | Phase 4 | Pending |
| PROD-06 | Phase 4 | Complete |
| PROD-07 | Phase 4 | Complete |
| PROD-08 | Phase 4 | Pending |
| PROD-09 | Phase 4 | Pending |
| PROD-10 | Phase 4 | Pending |
| PROD-11 | Phase 4 | Pending |
| PROD-12 | Phase 4 | Pending |
| PROD-13 | Phase 4 | Pending |
| PROD-14 | Phase 4 | Pending |
| PROD-15 | Phase 4 | Pending |
| PROD-16 | Phase 4 | Pending |
| PROD-17 | Phase 4 | Pending |
| PROD-18 | Phase 4 | Pending |
| PROD-19 | Phase 4 | Pending |
| NAV-01 | TBD | Pending |
| NAV-02 | TBD | Pending |
| NAV-03 | TBD | Pending |
| WIDGET-01 | TBD | Pending |
| WIDGET-02 | TBD | Pending |
| WIDGET-03 | TBD | Pending |
| WIDGET-04 | TBD | Pending |
| WIDGET-05 | TBD | Pending |
| WIDGET-06 | TBD | Pending |
| WIDGET-07 | TBD | Pending |
| WIDGET-08 | TBD | Pending |
| WIDGET-09 | TBD | Complete |
| WIDGET-10 | TBD | Complete |
| WIDGET-11 | TBD | Pending |
| WIDGET-12 | TBD | Pending |
| WIDGET-13 | TBD | Complete |
| PLATFORM-01 | TBD | Complete |
| PUBLISH-01 | TBD | Pending |
| PUBLISH-02 | TBD | Pending |
| PUBLISH-03 | TBD | Pending |
