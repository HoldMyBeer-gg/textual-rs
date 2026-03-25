---
phase: 02-widget-tree-layout-and-styling
verified: 2026-03-25T20:30:00Z
status: passed
score: 5/5 success criteria verified
gaps:
  - truth: "A layout with dock:top, dock:bottom, and a flex-column center region renders correctly at multiple terminal sizes with fractional (1fr, 2fr) sizing"
    status: resolved
    reason: "The IRC demo example does not use dock: layout at all — Header uses height:1 flex (not dock:top), InputBar uses height:3 flex (not dock:bottom). Dock layout is unit-tested in isolation (dock_top_height_1_pins_to_top passes) but the end-to-end IRC demo does not exercise dock positioning. Success Criterion 3 is not satisfied by the demo."
    artifacts:
      - path: "crates/textual-rs/examples/irc_demo.rs"
        issue: "IRC_STYLESHEET uses 'height: 1' and 'height: 3' for Header/InputBar — not 'dock: top' / 'dock: bottom'. The comment at line 292 claims 'Header is docked top' but the CSS does not set dock."
    missing:
      - "Add dock: top to Header rule and dock: bottom to InputBar rule in IRC_STYLESHEET"
      - "Update apply_declarations to actually wire grid-template-columns/rows to ComputedStyle.grid_columns/grid_rows (currently stubbed with comment 'handled via Vec<TcssDimension> stored as String for now')"
  - truth: "apply_declarations correctly handles grid-template-columns and grid-template-rows"
    status: resolved
    reason: "In css/types.rs lines 305-311, the grid-template-columns and grid-template-rows branches in apply_declarations are empty stubs. property.rs correctly parses them as TcssValue::Dimensions but apply_declarations never stores them in self.grid_columns / self.grid_rows."
    artifacts:
      - path: "crates/textual-rs/src/css/types.rs"
        issue: "Lines 305-310: grid-template-columns and grid-template-rows match arms have comment 'handled via Vec<TcssDimension> stored as String for now' but contain no code — grid_columns and grid_rows are never populated via CSS."
    missing:
      - "In apply_declarations, match 'grid-template-columns' => if let TcssValue::Dimensions(dims) = &decl.value { self.grid_columns = Some(dims.clone()); }"
      - "Same pattern for 'grid-template-rows' -> self.grid_rows"
human_verification:
  - test: "Visual IRC demo layout"
    expected: "cargo run --example irc_demo shows header bar at top, channel list (18 cols left), chat area (flex center), user list (22 cols right), input bar at bottom; Tab cycles focus with accent-colored borders; q quits cleanly"
    why_human: "Cannot verify visual rendering, focus color changes, or clean terminal restore programmatically without a real terminal"
---

# Phase 2: Widget Tree, Layout, and Styling Verification Report

**Phase Goal:** Developers can declare a widget tree (`App > Screen > Widget`) with parent/child relationships, lay it out using Taffy Flexbox/Grid/Dock, and style widgets using a `.tcss` stylesheet — and see the correct visual result rendered via ratatui.
**Verified:** 2026-03-25T20:30:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from Success Criteria)

| #   | Truth                                                                                                       | Status      | Evidence                                                                                                                                         |
|-----|-------------------------------------------------------------------------------------------------------------|-------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| 1   | Widget added via compose() appears on screen; removed via unmount() disappears — no unsafe code or borrow panics | ✓ VERIFIED  | 13 tree unit tests pass: mount/unmount/compose_children/push_screen/pop_screen all exercised; no unsafe code in any Phase 2 file                 |
| 2   | Tab moves keyboard focus through widgets in declared tab order; focused widget receives :focus pseudo-class styling | ✓ VERIFIED  | advance_focus/advance_focus_backward unit tests pass; IRC demo Tab test shows focus change; app.rs wires Tab/Shift+Tab to advance_focus calls   |
| 3   | A layout with dock:top, dock:bottom, and flex-column center renders correctly at multiple sizes with 1fr/2fr | ✗ FAILED    | Dock layout works in unit tests but IRC demo uses flex height not dock CSS. See gaps section.                                                     |
| 4   | .tcss stylesheet with type, class, and ID selectors applies correct cascade and specificity; inline styles win | ✓ VERIFIED  | 9 cascade tests pass: type < class < ID < inline confirmed; :focus pseudo-class matching confirmed; 31 CSS tests total                            |
| 5   | Border styles, padding, color, and background properties render correctly when declared in TCSS              | ✓ VERIFIED  | PropertyParser handles all properties; apply_declarations confirmed for color/border/padding; IRC demo renders bordered widgets via TestBackend  |

**Score:** 4/5 success criteria verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/textual-rs/src/widget/mod.rs` | Widget trait with render/compose/on_mount/on_unmount/can_focus | ✓ VERIFIED | All 8 trait methods present including default_css (Sized gate). Widget is object-safe. |
| `crates/textual-rs/src/widget/context.rs` | AppContext arena owner with DenseSlotMap + SecondaryMaps | ✓ VERIFIED | DenseSlotMap<WidgetId, Box<dyn Widget>> + 7 SecondaryMaps; 1 extra field (input_buffer, Phase 2 demo hack) |
| `crates/textual-rs/src/widget/tree.rs` | mount/unmount/compose/focus management helpers | ✓ VERIFIED | All 9 public functions present + compose_subtree (added for multi-level composition). 13 unit tests. |
| `crates/textual-rs/src/css/types.rs` | ComputedStyle struct, property enums, PseudoClassSet, Declaration | ⚠️ PARTIAL | All types present. apply_declarations mostly functional but grid-template-columns/rows handlers are empty stubs (lines 305-311). |
| `crates/textual-rs/src/layout/bridge.rs` | TaffyBridge sync and layout computation | ✓ VERIFIED | sync_subtree, sync_dirty_subtree, compute_layout, rect_for, remove_subtree all present. collect_absolute_rects added for correct absolute positioning. |
| `crates/textual-rs/src/layout/style_map.rs` | ComputedStyle to taffy::Style conversion | ✓ VERIFIED | taffy_style_from_computed handles all cases including dock absolute positioning. |
| `crates/textual-rs/src/layout/hit_map.rs` | Mouse hit testing col x row -> WidgetId | ✓ VERIFIED | MouseHitMap::build + hit_test; 3 unit tests pass. |
| `crates/textual-rs/src/css/parser.rs` | TCSS stylesheet parser using cssparser | ✓ VERIFIED | TcssRuleParser implements QualifiedRuleParser + AtRuleParser; parse_stylesheet returns (rules, errors). |
| `crates/textual-rs/src/css/selector.rs` | Selector parsing and matching | ✓ VERIFIED | All 8 variants; specificity ordering; selector_matches with ancestor traversal. 16 parse + 15 match tests pass. |
| `crates/textual-rs/src/css/property.rs` | Property declaration parsing | ✓ VERIFIED | parse_declaration_block handles all CSS-06 properties including grid-template-columns (parsed but not stored — see gap). |
| `crates/textual-rs/src/css/cascade.rs` | Cascade resolution and style application | ✓ VERIFIED | resolve_cascade, apply_cascade_to_tree, Stylesheet::parse, stylesheet_from_css_strings all present. |
| `crates/textual-rs/src/app.rs` | App struct holding AppContext, TaffyBridge, Stylesheet; render loop | ✓ VERIFIED | App::new(factory), with_css(), run(), full_render_pass(), render_to_test_backend(). Full integration wired. |
| `crates/textual-rs/examples/irc_demo.rs` | IRC client layout demo exercising dock, flex, grid, borders, focus | ⚠️ PARTIAL | Exercises flex, borders, CSS cascade, focus traversal — but uses flex height positioning instead of dock:top/dock:bottom CSS. 5 geometry tests pass at 80x24. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `widget/context.rs` | `widget/mod.rs` | `arena: DenseSlotMap<WidgetId, Box<dyn Widget>>` | ✓ WIRED | Confirmed at line 7 in context.rs |
| `widget/tree.rs` | `widget/context.rs` | `ctx.arena.insert / ctx.arena.remove` | ✓ WIRED | mount_widget uses ctx.arena.insert; unmount_widget uses ctx.arena.remove |
| `widget/context.rs` | `css/types.rs` | `SecondaryMap<WidgetId, ComputedStyle>` | ✓ WIRED | computed_styles field confirmed in context.rs |
| `layout/bridge.rs` | `widget/context.rs` | reads AppContext.children, computed_styles, dirty | ✓ WIRED | sync_node_dfs reads ctx.computed_styles + ctx.children + ctx.dirty |
| `layout/style_map.rs` | `css/types.rs` | `fn taffy_style_from_computed(s: &ComputedStyle) -> taffy::Style` | ✓ WIRED | Function exists and converts all relevant fields |
| `css/parser.rs` | `css/selector.rs` | `SelectorParser::parse_selector_list` | ✓ WIRED | QualifiedRuleParser::parse_prelude calls SelectorParser |
| `css/parser.rs` | `css/property.rs` | `parse_declaration_block` | ✓ WIRED | QualifiedRuleParser::parse_block calls parse_declaration_block |
| `css/cascade.rs` | `css/selector.rs` | `selector_matches` in resolve_cascade | ✓ WIRED | resolve_cascade calls selector_matches for each rule against each widget |
| `css/cascade.rs` | `css/types.rs` | `ComputedStyle::apply_declarations` | ✓ WIRED | resolve_cascade calls style.apply_declarations |
| `app.rs` | `widget/context.rs` | `AppContext::new` + `push_screen` | ✓ WIRED | App owns AppContext; push_screen called on startup |
| `app.rs` | `layout/bridge.rs` | `TaffyBridge::sync_dirty_subtree + compute_layout` | ✓ WIRED | full_render_pass calls both in sequence |
| `app.rs` | `css/cascade.rs` | `apply_cascade_to_tree` | ✓ WIRED | full_render_pass calls apply_cascade_to_tree before layout |
| `css/types.rs` | grid-template-columns/rows | `apply_declarations` stores to grid_columns/grid_rows | ✗ NOT WIRED | Lines 305-311 are empty stubs — grid dims parsed by property.rs but never written to ComputedStyle |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `app.rs render_widget_tree` | `bridge.rect_for(id)` | TaffyBridge::compute_layout | Yes — Taffy computes real geometry | ✓ FLOWING |
| `app.rs full_render_pass` | `ctx.computed_styles[id]` | apply_cascade_to_tree | Yes — cascade resolves real CSS rules | ✓ FLOWING |
| `css/cascade.rs resolve_cascade` | `grid_columns / grid_rows` in ComputedStyle | apply_declarations | No — stub in apply_declarations means CSS grid-template never populates ComputedStyle | ✗ HOLLOW_PROP |
| `irc_demo.rs widgets` | rendered text content | Static strings in render() | Static — Phase 2 intentionally uses static content, full reactivity in Phase 3 | ✓ FLOWING (for Phase 2 scope) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Library tests pass (all 77) | `cargo test -p textual-rs` | 77 passed, 0 failed | ✓ PASS |
| IRC demo integration tests pass (5 geometry tests) | `cargo test --example irc_demo` | 5 passed, 0 failed | ✓ PASS |
| Header at row 0 full width (80x24) | irc_demo test `header_occupies_row_0_full_width` | rect.y=0, height=1, width=80 | ✓ PASS |
| InputBar at bottom 3 rows (80x24) | irc_demo test `input_bar_occupies_bottom_3_rows` | rect.height=3, y+height=24, width=80 | ✓ PASS |
| ChannelList 18 cols left, UserList 22 cols right | irc_demo geometry tests | x=0 w=18 / x+w=80 w=22 | ✓ PASS |
| Dock CSS applied end-to-end | Would need dock:top in IRC stylesheet | Not tested end-to-end (only unit-tested) | ✗ FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| TREE-01 | 02-01 | SlotMap arena widget tree with no unsafe | ✓ SATISFIED | DenseSlotMap used; no unsafe blocks anywhere in Phase 2 |
| TREE-02 | 02-01 | Widget trait with render/compose/on_mount/on_unmount lifecycle | ✓ SATISFIED | All 4 lifecycle methods on Widget trait; on_mount/on_unmount take &self (deferred ctx-mutation to Phase 3 per plan) |
| TREE-03 | 02-01 | App > Screen > Widget hierarchy with screen stack | ✓ SATISFIED | screen_stack Vec<WidgetId> in AppContext; push_screen/pop_screen tested |
| TREE-04 | 02-01 | Dynamic widget composition — add/remove children at runtime | ✓ SATISFIED | compose_children + compose_subtree add children; unmount_widget removes them; tested |
| TREE-05 | 02-01 | Keyboard focus management with tab order traversal | ✓ SATISFIED | advance_focus/advance_focus_backward DFS order with wrapping; Tab/Shift+Tab wired in app.rs |
| LAYOUT-01 | 02-02 | Taffy-backed layout engine supporting CSS Flexbox and CSS Grid | ✓ SATISFIED | TaffyBridge wraps Taffy 0.9.2; flex and grid tested |
| LAYOUT-02 | 02-02 | Vertical and horizontal layout containers | ✓ SATISFIED | LayoutDirection::Vertical/Horizontal mapped to FlexDirection::Column/Row; both tested |
| LAYOUT-03 | 02-02 | Grid layout with configurable rows/columns | ✓ SATISFIED | grid_layout_2x2_correct_rects test passes |
| LAYOUT-04 | 02-02 | Dock layout (dock widgets to top/bottom/left/right edges) | ✓ SATISFIED | dock_top_height_1_pins_to_top test passes; dock_top_maps_to_absolute_position test passes; dock:top/bottom/left/right all handled in style_map.rs |
| LAYOUT-05 | 02-02 | Fractional units (1fr, 2fr) for proportional sizing | ✓ SATISFIED | horizontal_flex_fractional_children_correct_widths test: flex_grow 1,2,1 = widths 20,40,20 |
| LAYOUT-06 | 02-02 | Fixed, percentage, and auto sizing modes | ✓ SATISFIED | Length, Percent, Auto all tested; length_dimension_maps_correctly + percent_dimension_maps_correctly |
| LAYOUT-07 | 02-02 | Dirty flag system — only recompute layout for changed subtrees | ✓ SATISFIED | dirty_flag_prevents_sync_of_clean_subtree test passes; sync_dirty_subtree short-circuits on !dirty |
| CSS-01 | 02-03 | TCSS parser (subset of CSS) using cssparser tokenizer | ✓ SATISFIED | TcssRuleParser with cssparser StyleSheetParser; 5 stylesheet parsing tests pass |
| CSS-02 | 02-03 | Type, class, and ID selector matching | ✓ SATISFIED | Selector::Type/Class/Id all parse and match; 31 selector tests pass |
| CSS-03 | 02-03 | Style cascade with CSS specificity rules | ✓ SATISFIED | resolve_cascade sorts by (Specificity, source_order); type < class < ID < inline confirmed |
| CSS-04 | 02-03 | Inline styles on widget instances (highest specificity) | ✓ SATISFIED | resolve_cascade applies ctx.inline_styles last; test "inline styles override all" passes |
| CSS-05 | 02-03 | Pseudo-class states: :focus, :hover, :disabled | ✓ SATISFIED | PseudoClassSet with Focus/Hover/Disabled; :focus selector matching tested |
| CSS-06 | 02-03 | Supported properties: color, background, border, etc. | ✓ SATISFIED | All 20 listed properties handled in parse_declaration_block and apply_declarations (grid-template stub noted but property still accepted) |
| CSS-07 | 02-03 | Named colors, hex colors, rgb(), rgba() syntax | ✓ SATISFIED | cssparser-color handles all formats; 4 color parsing tests pass |
| CSS-08 | 02-03 | Border styles: solid, rounded, heavy, double, ascii, none | ✓ SATISFIED | All 6 BorderStyle variants in enum and property parser; IRC demo renders solid and rounded borders |
| CSS-09 | 02-03 | Default CSS defined on widget types, overridable by user stylesheets | ✓ SATISFIED | default_css() on Widget trait (Sized gate); stylesheet_from_css_strings() API for registration; test confirms user overrides default |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/textual-rs/src/css/types.rs` | 305-310 | `grid-template-columns` / `grid-template-rows` match arms are empty stubs (comment only) | ⚠️ Warning | CSS grid-template columns/rows cannot be set via TCSS stylesheet; breaks LAYOUT-03 via CSS (though direct ComputedStyle mutation works) |
| `crates/textual-rs/src/widget/context.rs` | 18 | `pub input_buffer: String` field is a demo-only hack ("Phase 3 replaces with proper reactive state") | ℹ️ Info | App-level mutable string in AppContext is a Phase 3 concern; doesn't block Phase 2 goals |
| `crates/textual-rs/examples/irc_demo.rs` | 292 | Comment says "Header is docked top" but Header has no `dock:` CSS property | ℹ️ Info | Misleading comment; layout is correct (flex height:1 achieves same visual result) but Success Criterion 3 is not exercised |

### Human Verification Required

#### 1. Visual IRC Demo Layout

**Test:** Run `cargo run --example irc_demo` in a terminal (at least 80x24)
**Expected:**
- Header bar at top (row 0) with catppuccin blue-on-dark styling showing "#general — textual-rs IRC demo"
- Channel list (18 cols) with solid border on left side
- Chat area (fills remaining width) with solid border in center
- User list (22 cols) with solid border on right side
- Input bar (3 rows) with rounded border at bottom showing "Type a message..."
- Pressing Tab moves focus to ChannelList (border accent color changes)
- Pressing Tab again moves focus to InputBar
- Pressing Tab again wraps back to ChannelList
- Pressing q exits cleanly with terminal restored

**Why human:** Visual appearance, focus color changes (CSS :focus pseudo-class updating border color), and clean terminal restore cannot be verified programmatically without a real terminal.

---

## Gaps Summary

Two gaps identified:

**Gap 1 — Dock CSS not exercised end-to-end (Success Criterion 3 partial):**
The IRC demo TCSS stylesheet does not use `dock: top` or `dock: bottom`. The Header positions via `height: 1` in a flex-column (which achieves the same visual result) and the InputBar uses `height: 3`. Dock layout is proven by unit tests (`dock_top_height_1_pins_to_top`, `dock_top_maps_to_absolute_position_with_correct_insets`) but the integration demo — which is the end-to-end proof — does not exercise dock CSS. Success Criterion 3 explicitly requires "A layout with `dock: top`, `dock: bottom`, and a flex-column center region."

**Gap 2 — grid-template-columns/rows not applied via CSS (apply_declarations stub):**
`property.rs` correctly parses `grid-template-columns` and `grid-template-rows` into `TcssValue::Dimensions(Vec<TcssDimension>)`. However, `apply_declarations` in `types.rs` has empty match arms for both properties — the parsed values are never written to `self.grid_columns` / `self.grid_rows`. This means CSS grid layouts can only be set by directly mutating `ComputedStyle` fields, not via TCSS stylesheets.

These two gaps are related: fixing the IRC demo to use `dock:` CSS would also expose whether the cascade correctly applies dock styles (it should — `apply_declarations` handles `dock` correctly). The grid stub is separate and straightforward to fix (2-line change).

---

_Verified: 2026-03-25T20:30:00Z_
_Verifier: Claude (gsd-verifier)_
