---
phase: 01-semantic-theme-engine
plan: 01
subsystem: css/theme
tags: [theme, color, hsl, semantic-tokens]
dependency_graph:
  requires: []
  provides: [Theme, default_dark_theme, lighten_color, rgb_to_hsl, hsl_to_rgb]
  affects: [css-variable-resolution, widget-styling]
tech_stack:
  added: []
  patterns: [HSL-color-space, semantic-color-tokens, shade-generation]
key_files:
  created:
    - crates/textual-rs/src/css/theme.rs
  modified:
    - crates/textual-rs/src/css/mod.rs
decisions:
  - Used pure-math HSL conversion (no external crate) for zero-dependency color manipulation
  - Shade parsing uses explicit suffix matching (lighten-1..3, darken-1..3) rather than regex for simplicity and speed
  - Panel color computed via linear RGB blend (surface * 0.9 + primary * 0.1) matching Python Textual behavior
metrics:
  duration: 188s
  completed: 2026-03-26T20:53:34Z
  tasks: 1/1
  files_created: 1
  files_modified: 1
  test_count: 20
  line_count: 472
---

# Phase 01 Plan 01: Theme Struct, Shade Generation, and Default Dark Theme Summary

Theme struct with 10 semantic color slots, HSL-based shade generation (lighten/darken 1-3), and default dark theme matching Python Textual's textual-dark palette with luminosity_spread=0.15.

## What Was Built

### HSL Color Space Conversions
- `rgb_to_hsl(r, g, b) -> (h, s, l)` -- pure-math conversion, H in 0-360, S/L in 0.0-1.0
- `hsl_to_rgb(h, s, l) -> (r, g, b)` -- inverse conversion with proper clamping
- Round-trip accuracy verified for edge cases (pure red, white, black, primary blue)

### lighten_color Function
- `lighten_color(color: TcssColor, delta: f64) -> TcssColor`
- Positive delta lightens, negative darkens, clamped to [0.0, 1.0] luminosity
- Non-Rgb variants (Reset, Named, Rgba) pass through unchanged

### Theme Struct
- 10 semantic color fields: primary, secondary, accent, surface, panel, background, foreground, success, warning, error
- `dark: bool` and `luminosity_spread: f64` control shade generation
- `variables: HashMap<String, TcssColor>` for user-defined overrides

### Theme::resolve()
- Resolves base names ("primary") and shade variants ("primary-lighten-2", "accent-darken-1")
- Checks variables HashMap first for user overrides
- Shade delta = N * (luminosity_spread / 2.0), applied via lighten_color
- Returns None for unknown names or invalid shade numbers (>3)

### default_dark_theme()
- Matches Python Textual's textual-dark palette exactly
- Panel computed via blend_rgb(surface, primary, 0.1) = (27, 39, 48)

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| e47f3ca | test | TDD RED -- 20 failing tests for theme struct, shades, resolve |
| 2fcd092 | feat | TDD GREEN -- full implementation, all 20 tests pass |

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all functions are fully implemented with no placeholders.

## Self-Check: PASSED

- [x] crates/textual-rs/src/css/theme.rs exists (472 lines)
- [x] crates/textual-rs/src/css/mod.rs updated with theme module
- [x] Commit e47f3ca (RED) verified
- [x] Commit 2fcd092 (GREEN) verified
- [x] 20/20 tests pass
