# Requirements: textual-rs

**Defined:** 2026-03-24 (v1.0), updated 2026-03-26 (v1.1)
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
- [ ] **THEME-02**: Shade generation (-lighten-1/2/3, -darken-1/2/3) works on any color variable
- [ ] **THEME-03**: A default dark theme ships matching Python Textual's textual-dark palette
- [ ] **THEME-04**: User can provide a custom theme via CSS

### Widget Visual Quality

- [ ] **VISUAL-01**: Button renders with 3D depth -- lighter top border, darker bottom; inverted on press
- [ ] **VISUAL-02**: Switch renders as pill-shaped toggle with distinct knob/track colors
- [ ] **VISUAL-03**: DataTable has zebra-striped rows, bold colored headers, smooth scrollbar
- [ ] **VISUAL-04**: Tabs show active indicator (underline/color bar) distinguishing selected from inactive
- [ ] **VISUAL-05**: Markdown renders headings in accent colors, code blocks with dark bg, links colored+underlined
- [ ] **VISUAL-06**: Placeholder renders with cross-hatch/textured background pattern
- [ ] **VISUAL-07**: Footer key badges use high-contrast accent colors

### Interactive States

- [ ] **STATE-01**: Every focusable widget shows visible focus indicator when focused
- [ ] **STATE-02**: Hover state changes widget appearance on mouse-over
- [ ] **STATE-03**: Active/pressed state on Button shows visual depression during click
- [ ] **STATE-04**: Selected items in list widgets use accent color + bold (not REVERSED)
- [ ] **STATE-05**: Invalid Input fields show red border/text; valid show normal/green

### Demo Quality

- [ ] **DEMO-01**: Demo UI visually comparable to Python Textual widget gallery
- [ ] **DEMO-02**: IRC demo renders professional-looking client with clear visual hierarchy
- [ ] **DEMO-03**: Both demos use McGugan Box borders with proper background depth

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| THEME-01 | Phase 1 | Pending |
| THEME-02 | Phase 1 | Pending |
| THEME-03 | Phase 1 | Pending |
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
| VISUAL-02 | Phase 3 | Pending |
| VISUAL-03 | Phase 3 | Pending |
| VISUAL-04 | Phase 3 | Pending |
| VISUAL-05 | Phase 3 | Pending |
| VISUAL-06 | Phase 3 | Pending |
| VISUAL-07 | Phase 3 | Pending |
| DEMO-01 | Phase 3 | Pending |
| DEMO-02 | Phase 3 | Pending |
| DEMO-03 | Phase 3 | Pending |
