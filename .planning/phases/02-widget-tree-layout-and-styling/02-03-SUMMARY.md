---
phase: 02-widget-tree-layout-and-styling
plan: 03
subsystem: css
tags: [css, styling, parser, cascade, selector]
dependency_graph:
  requires: ["02-01"]
  provides: ["css-engine", "selector-matching", "cascade-resolution"]
  affects: ["02-02"]
tech_stack:
  added: ["cssparser 0.37.0", "cssparser-color 0.5.0"]
  patterns: ["TDD", "cssparser StyleSheetParser", "Specificity ordering", "DFS cascade application"]
key_files:
  created:
    - crates/textual-rs/src/css/selector.rs
    - crates/textual-rs/src/css/property.rs
    - crates/textual-rs/src/css/parser.rs
    - crates/textual-rs/src/css/cascade.rs
  modified:
    - crates/textual-rs/Cargo.toml
    - crates/textual-rs/src/css/mod.rs
    - crates/textual-rs/src/css/types.rs
decisions:
  - "cssparser 0.37.0 StyleSheetParser used with TcssRuleParser implementing QualifiedRuleParser + AtRuleParser"
  - "ParseError::into::<U>() used for error type conversion (not map_custom which doesn't exist in 0.37.0)"
  - "loc.line is 0-indexed in cssparser; +1 applied for human-readable error messages"
  - "rgba() alpha uses CSS 0-1 range; stored as 0-255 byte when < 1.0 (TcssColor::Rgba) or as Rgb when alpha >= 1.0"
  - "TcssValue::Sides and TcssValue::Dimensions added for multi-value padding/margin and grid templates"
  - "default_css() has where Self: Sized — not callable on dyn Widget; stylesheet_from_css_strings() API provided for caller to supply default CSS at startup"
metrics:
  duration: 12min
  completed_date: "2026-03-25"
  tasks_completed: 2
  files_created: 4
  files_modified: 3
  tests_added: 31
---

# Phase 2 Plan 3: TCSS Styling Engine Summary

**One-liner:** cssparser-based TCSS engine with hand-rolled selector parser, property decoder, specificity-ordered cascade resolver, and pseudo-class matching.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Selector parsing, matching, and property parsing | 9cefcdb | selector.rs, property.rs, mod.rs, types.rs |
| 2 | Cascade resolver, stylesheet parser, and default CSS | 7b5b80b | parser.rs, cascade.rs, mod.rs |

## What Was Built

### Task 1: Selector & Property Engine

**`css/selector.rs`** — `Selector` enum with 8 variants (Type, Class, Id, Universal, PseudoClass, Descendant, Child, Compound), `Specificity` struct with `Ord` (id=1,0,0 > class=0,1,0 > type=0,0,1), `selector_matches()` recursively walks the widget tree via `ctx.arena`, `ctx.parent`, and `ctx.pseudo_classes`, `SelectorParser::parse_selector_list()` reads comma-separated selectors with descendant/child combinators.

**`css/property.rs`** — `parse_declaration_block()` handles all CSS-06 properties: colors via cssparser-color (named, hex 3/6, rgb(), rgba()), dimensions (Length/Percent/Fraction/Auto), border styles, display, visibility, opacity, text-align, overflow, scrollbar-gutter, dock, flex-grow, grid-template-columns/rows, layout-direction.

**`css/types.rs`** (extended) — Added `TcssValue::Sides(Sides<f32>)` for padding/margin shorthand with 2+ values; `TcssValue::Dimensions(Vec<TcssDimension>)` for grid templates.

### Task 2: Stylesheet Parser & Cascade

**`css/parser.rs`** — `TcssRuleParser` implementing `QualifiedRuleParser` + `AtRuleParser` for `StyleSheetParser`. `parse_stylesheet()` collects `Ok(Rule)` entries and formats `Err` entries as human-readable error strings with 1-indexed line numbers.

**`css/cascade.rs`** — `Stylesheet` struct, `resolve_cascade()` collects matching rules, sorts by `(Specificity, source_order)` ascending and applies declarations in that order so higher specificity/later source wins. Inline styles applied last. `apply_cascade_to_tree()` walks DFS from screen root and stores computed styles in `ctx.computed_styles`. `stylesheet_from_css_strings()` concatenates multiple CSS strings for default CSS aggregation.

## Test Coverage

62 total tests pass (all library tests):
- 16 selector parsing tests (type, class, id, compound, descendant, child, pseudo-class, universal)
- 15 selector matching tests (type match/mismatch, descendant, child non-parent, pseudo-class)
- 3 specificity ordering tests
- 12 property parsing tests (colors, dimensions, border, display, opacity, dock)
- 5 stylesheet parsing tests (single rule, 3 rules, empty, error collection, multiple declarations)
- 9 cascade resolution tests (type < class < id < inline, same-specificity source order, focus pseudo-class, default CSS override, full roundtrip)
- 2 tree-level tests (apply_cascade_to_tree, stylesheet_from_css_strings)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] cssparser ParseError::map_custom doesn't exist in 0.37.0**
- **Found during:** Task 2 compilation
- **Issue:** Plan specified `.map_err(|e| e.map_custom(...))` but this method doesn't exist in cssparser 0.37.0
- **Fix:** Used `ParseError::into::<U>()` with `T: Into<U>` which is the correct 0.37.0 API
- **Files modified:** `crates/textual-rs/src/css/parser.rs`
- **Commit:** 7b5b80b

**2. [Rule 1 - Bug] cssparser line numbers are 0-indexed**
- **Found during:** Task 2 test failure
- **Issue:** Test expected "line 2" but cssparser reported `loc.line = 1` (0-indexed)
- **Fix:** Added `loc.line + 1` to produce human-readable 1-indexed line numbers in error messages
- **Files modified:** `crates/textual-rs/src/css/parser.rs`
- **Commit:** 7b5b80b

**3. [Rule 2 - Missing] TcssValue::Sides and TcssValue::Dimensions variants needed**
- **Found during:** Task 1 implementation
- **Issue:** `parse_declaration_block` needed to return multi-value padding/margin (not just Float) and grid template lists
- **Fix:** Added `TcssValue::Sides(Sides<f32>)` and `TcssValue::Dimensions(Vec<TcssDimension>)` to `types.rs`
- **Files modified:** `crates/textual-rs/src/css/types.rs`
- **Commit:** 9cefcdb

**4. [Rule 1 - Bug] rgba() test used CSS-invalid alpha value 128**
- **Found during:** Task 1 implementation
- **Issue:** Plan test `rgba(255, 0, 0, 128)` — CSS rgba uses 0.0-1.0 alpha; 128 gets clamped to 1.0 (opaque), producing Rgb not Rgba
- **Fix:** Changed test to `rgba(255, 0, 0, 0.5)` (valid CSS 50% alpha) which correctly produces `TcssColor::Rgba`
- **Files modified:** `crates/textual-rs/src/css/property.rs`
- **Commit:** 9cefcdb

## Known Stubs

None — all functionality is fully implemented.

## Self-Check: PASSED

All created files verified present. Both task commits found in git log.
- selector.rs: FOUND
- property.rs: FOUND
- parser.rs: FOUND
- cascade.rs: FOUND
- Commit 9cefcdb: FOUND
- Commit 7b5b80b: FOUND
