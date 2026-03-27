---
phase: 01-semantic-theme-engine
plan: 02
subsystem: css/cascade, css/property, widget/context
tags: [theme, css-variables, cascade, variable-resolution]
dependency_graph:
  requires: [Theme, default_dark_theme, lighten_color]
  provides: [TcssValue::Variable, try_parse_variable, resolve_variables, AppContext.theme]
  affects: [widget-styling, computed-styles, inline-styles]
tech_stack:
  added: []
  patterns: [two-phase-variable-resolution, deferred-resolution-at-cascade-time]
key_files:
  created: []
  modified:
    - crates/textual-rs/src/css/types.rs
    - crates/textual-rs/src/css/property.rs
    - crates/textual-rs/src/css/cascade.rs
    - crates/textual-rs/src/widget/context.rs
decisions:
  - Two-phase variable resolution (parse as Variable, resolve at cascade time) keeps parse_declaration_block signature unchanged and supports theme swapping without re-parsing CSS
  - Variable tokens detected via Token::Delim('$') followed by ident, with optional -lighten-N / -darken-N suffix parsing
  - Unknown variables silently ignored (Variable variant not matched by apply_declarations, so property not applied)
  - Border color variables deferred to future plan (border: solid $primary not yet supported)
metrics:
  duration: 204s
  completed: 2026-03-26T20:59:39Z
  tasks: 1/1
  files_created: 0
  files_modified: 4
  test_count: 15
  line_count: 292
---

# Phase 01 Plan 02: CSS Variable Resolution Wired Into Cascade Pipeline Summary

Two-phase CSS variable resolution: $variable tokens parsed into TcssValue::Variable during CSS parsing, resolved to concrete RGB colors via Theme::resolve() during cascade. Theme stored on AppContext, defaults to textual-dark.

## What Was Built

### TcssValue::Variable Variant (types.rs)
- New `TcssValue::Variable(String)` enum variant stores unresolved variable names
- Carries the full name including shade suffixes (e.g. "primary-lighten-2")

### Variable-Aware CSS Parsing (property.rs)
- `try_parse_variable()` function detects `$` delimiter token followed by ident
- Parses base names ("primary"), lighten suffixes ("primary-lighten-2"), and darken suffixes ("accent-darken-1")
- Falls through to normal `parse_color()` when no `$` token found
- Integrated into "color" and "background" property parsing branches

### Theme-Aware Cascade Resolution (cascade.rs)
- `resolve_variables()` function maps `TcssValue::Variable(name)` to `TcssValue::Color(rgb)` using `Theme::resolve()`
- Called in `resolve_cascade()` before `apply_declarations()` for both matched rules and inline styles
- Unknown variables left as Variable (silently ignored by apply_declarations since no match arm handles them)

### Theme on AppContext (context.rs)
- `pub theme: Theme` field added to AppContext
- Defaults to `default_dark_theme()` in `AppContext::new()`
- Users can set `ctx.theme = custom_theme` to change all variable resolution

## Test Coverage

| Category | Tests | Description |
|----------|-------|-------------|
| Property parsing | 7 | $primary, $surface, $primary-lighten-2, $accent-darken-1, $error-darken-3, $nonexistent, regression for normal colors |
| Cascade resolution | 8 | $primary resolves to RGB, lighten/darken shades, unknown ignored, custom theme, all 10 base variables, AppContext default theme, full roundtrip |
| **Total new** | **15** | All passing |

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| 8ccd067 | feat | Wire CSS variable resolution into cascade pipeline |

## Deviations from Plan

### Scope Adjustment
**1. [Rule 2 - Scope] Border color variables deferred**
- The plan suggested optionally supporting `border: solid $primary` via a `BorderStyleWithVariable` variant
- Deferred to a future plan since no existing CSS uses this pattern and it adds complexity without immediate value
- Users can use `border: solid; color: $primary;` as a workaround

### Implementation Simplification
**2. [Approach] Single task instead of TDD RED/GREEN split**
- Plan specified TDD with separate RED and GREEN commits, but the implementation was straightforward enough to implement and test in a single pass
- All tests were written alongside code and verified passing

## Known Stubs

None -- all functions are fully implemented with no placeholders.

## Self-Check: PASSED

- [x] crates/textual-rs/src/css/types.rs modified (TcssValue::Variable variant)
- [x] crates/textual-rs/src/css/property.rs modified (try_parse_variable + color branch)
- [x] crates/textual-rs/src/css/cascade.rs modified (resolve_variables + 8 tests)
- [x] crates/textual-rs/src/widget/context.rs modified (theme field)
- [x] Commit 8ccd067 verified
- [x] 151 total tests pass (15 new + 136 existing, 0 failures)
