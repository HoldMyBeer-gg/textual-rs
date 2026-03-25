# Phase 2: Widget Tree, Layout, and Styling - Research

**Researched:** 2026-03-25
**Domain:** Rust arena-based widget trees, Taffy CSS layout engine, cssparser-based TCSS styling
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Widget API & Borrow Pattern**
- **D-01:** AppContext pattern for arena access. Widget methods receive `(&self, ctx: &AppContext)` or `(&self, ctx: &mut AppContext)` instead of holding `&mut self` and `&mut Arena` simultaneously. AppContext owns the SlotMap arena, SecondaryMaps for children/parent/styles/dirty flags. This eliminates the SlotMap borrow conflict entirely.
- **D-02:** Direct ratatui Buffer rendering. `Widget::render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer)` writes to ratatui's Buffer directly. Widgets can use any ratatui widget internally (Block, Paragraph, etc.). No intermediate Strip representation — zero-copy, leverages the full ratatui ecosystem.
- **D-03:** `compose()` returns `Vec<Box<dyn Widget>>`. Simple, idiomatic Rust — no generator/yield needed. AppContext inserts returned widgets into the arena and wires parent/child relationships. Default implementation returns empty vec (leaf widget).
- **D-04:** `dyn Widget` everywhere. All widgets (built-in and user-defined) are `Box<dyn Widget>` in the arena. No enum dispatch for built-ins. Standard extensible type hierarchy pattern.

**Layout-to-Render Bridge**
- **D-05:** TaffyBridge sync layer. A dedicated `TaffyBridge` struct owns the `TaffyTree<WidgetId>` and maintains a `HashMap<WidgetId, NodeId>` mapping. It syncs the Taffy tree to match the widget arena, converts `ComputedStyle` to Taffy `Style`, and after `compute_layout()` converts Taffy's f32 positions/sizes to ratatui `Rect` (u16 cells) with rounding.
- **D-06:** Dock layout emulated with nested Flexbox. `dock: top/bottom/left/right` declarations compile down to nested Flexbox containers in the TaffyBridge. No custom layout pass — Taffy's flex engine handles everything.
- **D-07:** Dirty-flag bubbling for incremental relayout. When a widget is marked dirty, its ancestors are marked dirty up to the Screen. Next render pass checks dirty flags, re-syncs only dirty subtrees to Taffy, recomputes layout from the highest dirty ancestor, then clears flags.

**CSS Parser Scope & Property Set**
- **D-08:** cssparser crate (Mozilla's tokenizer) + hand-rolled SelectorParser and PropertyParser. The cssparser crate handles tokenization (battle-tested edge case handling). Selector matching and property parsing are built by hand on top of the token stream.
- **D-09:** Selector support matches Python Textual: type (`Button`), class (`.highlight`), ID (`#sidebar`), pseudo-class (`:focus`, `:hover`, `:disabled`), descendant combinator (`Screen Label`), child combinator (`Container > Button`). No sibling combinators, no pseudo-elements in v1.
- **D-10:** Style cascade with CSS specificity rules. Inline styles > ID > class > type. Standard CSS specificity calculation.
- **D-11:** DEFAULT_CSS as static `&'static str` on the Widget trait. Each widget type defines `fn default_css() -> &'static str`. The stylesheet loader collects all DEFAULT_CSS from mounted widget types and prepends them at lowest specificity before user stylesheets.
- **D-12:** Full TCSS property set per requirements CSS-06: color, background, border, border-title, padding, margin, width, height, min-width, min-height, max-width, max-height, display, visibility, opacity, text-align, overflow, scrollbar-gutter. Plus border styles (solid, rounded, heavy, double, ascii, none) and color syntax (named, hex, rgb(), rgba()).

**Screen Stack & Focus Model**
- **D-13:** Screens are special WidgetIds in the shared arena. App maintains a `Vec<WidgetId>` as the screen stack. Pushing a screen inserts it + its children into the arena. Popping removes the subtree. Only the top screen renders and receives input.
- **D-14:** DOM-order focus traversal with `can_focus()` flag. Tab follows depth-first tree order. Widgets opt in via `fn can_focus(&self) -> bool` (default false). Focus state (`focused_widget: Option<WidgetId>`) lives on AppContext. Setting focus updates `:focus` pseudo-class on the widget.

### Claude's Discretion
- Exact `Widget` trait method signatures beyond render/compose/can_focus (on_mount, on_unmount lifecycle methods)
- AppContext internal data structure choices (DenseSlotMap vs SlotMap, HashMap vs SecondaryMap for node_map)
- CSS property parsing internals (how property values are stored, enum vs typed structs)
- TaffyBridge sync algorithm details (incremental vs full rebuild)
- Error types and error handling strategy for CSS parse errors
- EventPropagation enum design (for Phase 3, but the type may be defined here)
- Mouse hit map implementation (col x row -> WidgetId lookup)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TREE-01 | Index-arena widget tree (`SlotMap<WidgetId, Box<dyn Widget>>`) with no unsafe parent pointers | SlotMap 1.1.1 API verified; DenseSlotMap + SecondaryMap is the right combination |
| TREE-02 | `Widget` trait with `render()`, `compose()`, `on_mount()`, `on_unmount()` lifecycle | AppContext pattern resolves borrow conflict; dyn-safe trait design documented |
| TREE-03 | `App > Screen > Widget` hierarchy with screen stack (push/pop for modals) | Screens as WidgetIds in shared arena; `Vec<WidgetId>` screen stack on AppContext |
| TREE-04 | Dynamic widget composition — widgets can add/remove children at runtime | `AppContext::mount()` / `unmount()` wires parent/child in DenseSlotMap + SecondaryMaps |
| TREE-05 | Keyboard focus management with tab order traversal | Depth-first DOM order over `can_focus()` widgets; `:focus` pseudo-class on AppContext |
| LAYOUT-01 | Taffy-backed layout engine supporting CSS Flexbox and CSS Grid | Taffy 0.9.2 — full spec-compliant Flexbox + Grid verified |
| LAYOUT-02 | Vertical and horizontal layout containers | `FlexDirection::Column` / `FlexDirection::Row` in Taffy Style |
| LAYOUT-03 | Grid layout with configurable rows/columns | `Display::Grid` with `grid_template_rows`/`grid_template_columns`, `TrackSizingFunction` |
| LAYOUT-04 | Dock layout (dock widgets to top/bottom/left/right edges) | `Position::Absolute` + `inset` values in TaffyBridge (not a custom algorithm) |
| LAYOUT-05 | Fractional units (`1fr`, `2fr`) for proportional sizing | Taffy native `fr()` type in grid tracks; `flex_grow` for flex children |
| LAYOUT-06 | Fixed, percentage, and auto sizing modes | `Dimension::Length`, `Dimension::Percent`, `Dimension::Auto` |
| LAYOUT-07 | Dirty flag system — only recompute layout for changed subtrees | `SecondaryMap<WidgetId, bool>` dirty flags + ancestor bubbling pattern |
| CSS-01 | TCSS parser (subset of CSS) using `cssparser` tokenizer | cssparser 0.37.0 (current) + `QualifiedRuleParser` impl for block structure |
| CSS-02 | Type, class, and ID selector matching | Custom `Selector` enum + `selector_matches()` fn against AppContext widget metadata |
| CSS-03 | Style cascade with CSS specificity rules | `(a,b,c)` specificity tuple, sort-and-apply algorithm documented |
| CSS-04 | Inline styles on widget instances (highest specificity) | Applied after cascade; overrides all selector-matched rules |
| CSS-05 | Pseudo-class states: `:focus`, `:hover`, `:disabled` | `PseudoClassSet` per widget in AppContext; checked during selector matching |
| CSS-06 | Full supported property set (color, background, border, padding, margin, dimensions, etc.) | `ComputedStyle` struct with typed fields; `PropertyParser` match-dispatch table |
| CSS-07 | Named colors, hex colors (`#rgb`, `#rrggbb`), `rgb()`, `rgba()` syntax | cssparser-color 0.5.0 companion crate handles all color syntax |
| CSS-08 | Border styles: `solid`, `rounded`, `heavy`, `double`, `ascii`, `none` | `BorderStyle` enum → ratatui `BorderType` mapping in render layer |
| CSS-09 | Default CSS defined on widget types, overridable by user stylesheets | `fn default_css() -> &'static str` on `Widget` trait; lowest specificity in cascade |
</phase_requirements>

---

## Summary

Phase 2 builds three distinct but tightly coupled subsystems: the widget arena (plan 02-01), the Taffy layout bridge (plan 02-02), and the TCSS styling engine (plan 02-03). All three architectural decisions are locked in CONTEXT.md, so this research focuses on the concrete implementation details each plan needs to succeed.

The SlotMap arena approach resolves Rust's borrow conflict by passing `AppContext` (the arena owner) as a method argument rather than storing arena references inside widgets. This is the idiomatic Rust arena pattern, well-supported by slotmap 1.1.1's `DenseSlotMap` + `SecondaryMap` API. The AppContext owns the canonical widget data; widgets themselves are stateless capability descriptors inside `Box<dyn Widget>`.

Taffy 0.9.2 is the correct and current layout engine. Its `Style` struct maps 1:1 to TCSS property names, it supports both Flexbox and Grid natively, and dock layouts are expressible as `Position::Absolute` with `inset` values — no custom layout algorithm required. The TaffyBridge maintains a `HashMap<WidgetId, taffy::NodeId>` mapping and syncs layout nodes from `ComputedStyle`. The output f32 geometry is converted to ratatui `Rect` (u16 cells) by rounding.

The TCSS parser stack is: cssparser 0.37.0 tokenizer providing block structure → hand-rolled `SelectorParser` (recursive descent, ~200 lines) → hand-rolled `PropertyParser` (match-on-name dispatch, ~400 lines for the full CSS-06 property set). Color parsing delegates to `cssparser-color 0.5.0` which handles named/hex/rgb()/rgba() correctly. The cascade is a sort-and-apply over `(specificity, source_order, declarations)` tuples.

**Primary recommendation:** Implement in plan order (02-01 widget arena first, 02-02 layout bridge second, 02-03 CSS engine third). Each plan builds the data structures the next plan consumes: the arena provides `WidgetId`s that `TaffyBridge` indexes, and the CSS engine produces `ComputedStyle` that `TaffyBridge` reads to generate `taffy::Style`.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| slotmap | 1.1.1 | Widget arena (WidgetId generational indices, DenseSlotMap, SecondaryMap) | Industry-standard arena crate; prevents use-after-free without unsafe pointer trees |
| taffy | 0.9.2 | CSS Flexbox/Grid/Block layout computation | De facto standard Rust layout engine; used by Servo, Bevy, Zed, Slint, iocraft |
| cssparser | 0.37.0 | CSS Syntax Level 3 tokenizer + block structure parser | Mozilla's battle-tested CSS tokenizer used in Firefox; handles all edge cases |
| cssparser-color | 0.5.0 | Named/hex/rgb()/rgba() color parsing | Companion to cssparser; avoids reimplementing 140+ named colors and rgb syntax |
| ratatui | 0.30.0 | Buffer rendering target (already a Phase 1 dependency) | Widgets render directly to `ratatui::buffer::Buffer`; no intermediate representation |

**Version verification (2026-03-25 via `cargo search`):**
- `slotmap` current: 1.1.1 (confirmed)
- `taffy` current: 0.9.2 (confirmed)
- `cssparser` current: **0.37.0** (prior research documented 0.35.0 — now updated)
- `cssparser-color` current: 0.5.0 (confirmed)

**Installation:**
```toml
# In crates/textual-rs/Cargo.toml — additions for Phase 2
slotmap = "1.1.1"
taffy = "0.9.2"
cssparser = "0.37.0"
cssparser-color = "0.5.0"
```

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `DenseSlotMap` | `SlotMap` (standard) | DenseSlotMap is faster for iteration (rendering hot path); SlotMap is faster for insert/remove. Rendering wins: use DenseSlotMap. |
| `DenseSlotMap` | `HopSlotMap` | HopSlotMap has O(1) remove-by-value; not needed since removal is always by WidgetId. DenseSlotMap wins. |
| Hand-rolled selector engine | servo/stylo `selectors` crate | selectors crate needs ~300-line `Element` trait impl designed for browser DOM; TCSS has only 6 selector types. Hand-roll is ~200 lines and perfectly sufficient. |
| `cssparser-color` 0.5.0 | Hand-rolled color parser | cssparser-color handles `oklch()`, `lab()`, `display-p3` and future additions. Don't hand-roll. |
| Taffy absolute positioning for dock | Custom dock layout pass | Taffy absolute + inset is spec-compliant, tested at scale, and requires no custom algorithm. |
| `lightningcss` | cssparser + hand-rolled | lightningcss is a full web CSS pipeline designed for browsers. 40x heavier than needed. Not applicable. |

---

## Architecture Patterns

### Recommended Module Structure
```
crates/textual-rs/src/
├── app.rs               # Extended: App owns AppContext, drives widget-tree render loop
├── event.rs             # Extended: AppEvent gets WidgetMounted, WidgetUnmounted variants
├── terminal.rs          # Unchanged from Phase 1
├── lib.rs               # Re-exports: add widget, layout, css module paths
├── widget/
│   ├── mod.rs           # Widget trait, EventPropagation enum, public re-exports
│   ├── context.rs       # AppContext struct — arena owner, focus state, screen stack
│   └── tree.rs          # mount(), unmount(), compose traversal, subtree removal helpers
├── layout/
│   ├── mod.rs           # Public layout API: relayout_if_needed()
│   ├── bridge.rs        # TaffyBridge — WidgetId<->NodeId mapping, sync, Rect conversion
│   ├── style_map.rs     # ComputedStyle -> taffy::Style conversion functions
│   └── hit_map.rs       # MouseHitMap — col×row -> WidgetId lookup table
└── css/
    ├── mod.rs           # Public CSS API: parse_stylesheet(), apply_cascade_to_tree()
    ├── parser.rs        # cssparser QualifiedRuleParser impl, RuleListParser wrapper
    ├── selector.rs      # Selector enum, SelectorParser, selector_matches() fn
    ├── property.rs      # PropertyParser, Declaration, TcssValue enum
    ├── cascade.rs       # CascadeResolver, specificity calc, ComputedStyle builder
    └── types.rs         # ComputedStyle struct, all property type enums (Display, etc.)
```

### Pattern 1: AppContext Arena Owner

The core borrow-safety pattern for the entire phase. AppContext is passed by reference to every widget method, replacing the impossible simultaneous `(&mut Widget, &mut Arena)` pattern.

```rust
// Source: CONTEXT.md D-01 (locked decision) + slotmap 1.1.1 API
use slotmap::{DenseSlotMap, SecondaryMap, new_key_type};
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

new_key_type! { pub struct WidgetId; }

pub struct AppContext {
    // Primary arena — all widget objects live here
    pub arena: DenseSlotMap<WidgetId, Box<dyn Widget>>,
    // Structural metadata — SecondaryMap shares WidgetId validity
    pub children: SecondaryMap<WidgetId, Vec<WidgetId>>,
    pub parent:   SecondaryMap<WidgetId, Option<WidgetId>>,
    // Style state
    pub computed_styles:  SecondaryMap<WidgetId, ComputedStyle>,
    pub inline_styles:    SecondaryMap<WidgetId, Vec<Declaration>>,
    pub dirty:            SecondaryMap<WidgetId, bool>,
    pub pseudo_classes:   SecondaryMap<WidgetId, PseudoClassSet>,
    // Application state
    pub focused_widget:   Option<WidgetId>,
    pub screen_stack:     Vec<WidgetId>,
}

pub trait Widget: 'static {
    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer);

    fn compose(&self) -> Vec<Box<dyn Widget>> { vec![] }

    fn on_mount(&self, _ctx: &mut AppContext, _id: WidgetId) {}
    fn on_unmount(&self, _ctx: &mut AppContext, _id: WidgetId) {}

    fn can_focus(&self) -> bool { false }

    // CSS selector matching support — must return short name, NOT full path
    fn widget_type_name(&self) -> &'static str;
    fn classes(&self) -> &[&'static str] { &[] }
    fn id(&self) -> Option<&str> { None }

    // Default CSS at lowest specificity — parsed once at app startup
    fn default_css() -> &'static str where Self: Sized { "" }
}
```

**Critical note on `widget_type_name()`:** Must return the short name (`"Button"`) not the full Rust path (`"textual_rs::widgets::Button"`). Type selectors in TCSS match by short name. Implement this as a literal in every widget impl; do not derive from `std::any::type_name`.

### Pattern 2: TaffyBridge Sync

```rust
// Source: CONTEXT.md D-05 + taffy 0.9.2 docs (https://docs.rs/taffy/0.9.2)
use taffy::{TaffyTree, NodeId};
use std::collections::HashMap;

pub struct TaffyBridge {
    tree:         TaffyTree<WidgetId>,        // user data = WidgetId for each node
    node_map:     HashMap<WidgetId, NodeId>,  // widget -> taffy node
    layout_cache: HashMap<WidgetId, Rect>,    // last computed Rects
}

impl TaffyBridge {
    /// Sync dirty subtree rooted at `dirty_root` from AppContext.
    pub fn sync_dirty_subtree(&mut self, dirty_root: WidgetId, ctx: &AppContext) {
        // Collect subtree in DFS order
        let subtree = collect_subtree_ids(dirty_root, ctx);
        for id in &subtree {
            if !self.node_map.contains_key(id) {
                // New widget — create Taffy node
                let style = taffy_style_from_computed(&ctx.computed_styles[*id]);
                let node = self.tree.new_leaf(style).unwrap();
                self.node_map.insert(*id, node);
            } else {
                // Existing widget — update style
                let node = self.node_map[id];
                let style = taffy_style_from_computed(&ctx.computed_styles[*id]);
                self.tree.set_style(node, style).unwrap();
            }
        }
        // Re-wire children in Taffy tree to match arena
        for id in &subtree {
            let node = self.node_map[id];
            let child_nodes: Vec<NodeId> = ctx.children[*id].iter()
                .map(|c| self.node_map[c])
                .collect();
            self.tree.set_children(node, &child_nodes).unwrap();
        }
    }

    pub fn compute_layout(&mut self, screen_id: WidgetId, cols: u16, rows: u16) {
        let root = self.node_map[&screen_id];
        let space = taffy::geometry::Size {
            width:  taffy::style::AvailableSpace::Definite(cols as f32),
            height: taffy::style::AvailableSpace::Definite(rows as f32),
        };
        self.tree.compute_layout(root, space).unwrap();
        // Cache rects
        for (&id, &node) in &self.node_map {
            if let Ok(layout) = self.tree.layout(node) {
                self.layout_cache.insert(id, Rect {
                    x:      layout.location.x.floor() as u16,
                    y:      layout.location.y.floor() as u16,
                    width:  layout.size.width.round() as u16,
                    height: layout.size.height.round() as u16,
                });
            }
        }
    }

    pub fn rect_for(&self, id: WidgetId) -> Option<Rect> {
        self.layout_cache.get(&id).copied()
    }
}
```

### Pattern 3: Dock Layout via Absolute Positioning in Taffy

```rust
// Source: .planning/research/CSS_LAYOUT.md §2 TUI-Relevant Layout Patterns
// dock: top → absolute, pinned top+left+right, auto height
fn dock_top_style(height_cells: Option<f32>) -> taffy::Style {
    use taffy::prelude::*;
    Style {
        position: Position::Absolute,
        inset: Rect {
            top:    LengthPercentageAuto::Length(0.0),
            left:   LengthPercentageAuto::Length(0.0),
            right:  LengthPercentageAuto::Length(0.0),
            bottom: LengthPercentageAuto::Auto,
        },
        size: Size {
            width:  Dimension::Auto,
            height: height_cells.map(Dimension::Length).unwrap_or(Dimension::Auto),
        },
        ..Default::default()
    }
}
// dock: bottom → pin bottom+left+right, auto height
// dock: left   → pin top+bottom+left, auto width
// dock: right  → pin top+bottom+right, auto width
```

### Pattern 4: TCSS Parser Using cssparser QualifiedRuleParser

```rust
// Source: cssparser 0.37.0 docs (https://docs.rs/cssparser/0.37.0)
use cssparser::{Parser, ParserInput, RuleListParser, QualifiedRuleParser, ParseError};

pub struct TcssRuleParser;

impl<'i> QualifiedRuleParser<'i> for TcssRuleParser {
    type Prelude = Vec<Selector>;
    type QualifiedRule = Rule;
    type Error = TcssParseError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Vec<Selector>, ParseError<'i, TcssParseError>> {
        SelectorParser::parse_selector_list(input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Vec<Selector>,
        _start: &cssparser::ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Rule, ParseError<'i, TcssParseError>> {
        let declarations = PropertyParser::parse_declaration_block(input)?;
        Ok(Rule { selectors: prelude, declarations })
    }
}

pub fn parse_stylesheet(css: &str) -> (Vec<Rule>, Vec<String>) {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    let mut rules = Vec::new();
    let mut errors = Vec::new();
    for result in RuleListParser::new_for_stylesheet(&mut parser, TcssRuleParser) {
        match result {
            Ok(rule) => rules.push(rule),
            Err((err, slice)) => errors.push(format!(
                "CSS parse error at {:?}: {:?}",
                err.location, slice
            )),
        }
    }
    (rules, errors)
}
```

### Pattern 5: Selector Matching Against AppContext

```rust
// Source: .planning/research/CSS_LAYOUT.md §4
pub enum Selector {
    Type(String),
    Class(String),
    Id(String),
    Universal,
    PseudoClass(PseudoClass),
    Descendant(Box<Selector>, Box<Selector>),  // "A B"
    Child(Box<Selector>, Box<Selector>),        // "A > B"
    Compound(Vec<Selector>),                    // "Button.focused"
}

pub fn selector_matches(sel: &Selector, id: WidgetId, ctx: &AppContext) -> bool {
    let widget = &ctx.arena[id];
    match sel {
        Selector::Type(name)      => widget.widget_type_name() == name.as_str(),
        Selector::Class(cls)      => widget.classes().contains(&cls.as_str()),
        Selector::Id(ident)       => widget.id() == Some(ident.as_str()),
        Selector::Universal       => true,
        Selector::PseudoClass(pc) => ctx.pseudo_classes[id].contains(*pc),
        Selector::Descendant(ancestor_sel, subject_sel) => {
            selector_matches(subject_sel, id, ctx)
                && ancestors(id, ctx).any(|a| selector_matches(ancestor_sel, a, ctx))
        }
        Selector::Child(parent_sel, subject_sel) => {
            selector_matches(subject_sel, id, ctx)
                && ctx.parent[id]
                    .flatten()
                    .map_or(false, |p| selector_matches(parent_sel, p, ctx))
        }
        Selector::Compound(parts) => parts.iter().all(|p| selector_matches(p, id, ctx)),
    }
}

fn ancestors(id: WidgetId, ctx: &AppContext) -> impl Iterator<Item = WidgetId> + '_ {
    std::iter::successors(ctx.parent[id].flatten(), move |&cur| {
        ctx.parent[cur].flatten()
    })
}
```

### Pattern 6: Cascade Resolution

```rust
// Source: .planning/research/CSS_LAYOUT.md §4 + CSS Cascade spec
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Specificity(u32, u32, u32);  // (a=id, b=class/pseudo, c=type)

pub fn resolve_cascade(
    widget_id: WidgetId,
    stylesheets: &[Stylesheet],  // default CSS first (lowest), user CSS last (highest)
    ctx: &AppContext,
) -> ComputedStyle {
    let mut matched: Vec<(Specificity, usize, &Vec<Declaration>)> = Vec::new();

    for (sheet_idx, sheet) in stylesheets.iter().enumerate() {
        for (rule_idx, rule) in sheet.rules.iter().enumerate() {
            // Find the highest-specificity matching selector in this rule
            let best = rule.selectors.iter()
                .filter(|s| selector_matches(s, widget_id, ctx))
                .map(|s| s.specificity())
                .max();
            if let Some(spec) = best {
                matched.push((spec, sheet_idx * 100_000 + rule_idx, &rule.declarations));
            }
        }
    }

    // Sort ascending: lower specificity/order applied first, higher overwrites
    matched.sort_by_key(|&(spec, order, _)| (spec, order));

    let mut computed = ComputedStyle::default();
    for (_, _, decls) in &matched {
        computed.apply_declarations(decls);
    }
    // Inline styles always win — applied last
    if let Some(inline) = ctx.inline_styles.get(widget_id) {
        computed.apply_declarations(inline);
    }
    computed
}
```

### Pattern 7: ComputedStyle to Taffy Style Mapping

```rust
// Source: .planning/research/CSS_LAYOUT.md §6 property table + taffy 0.9.2 Style struct
use taffy::prelude::*;

pub fn taffy_style_from_computed(s: &ComputedStyle) -> Style {
    Style {
        display: match s.display {
            TcssDisplay::Flex  => Display::Flex,
            TcssDisplay::Grid  => Display::Grid,
            TcssDisplay::Block => Display::Block,
            TcssDisplay::None  => Display::None,
        },
        flex_direction: match s.layout_direction {
            LayoutDirection::Vertical   => FlexDirection::Column,
            LayoutDirection::Horizontal => FlexDirection::Row,
        },
        size: Size {
            width:  tcss_dim_to_taffy(s.width),
            height: tcss_dim_to_taffy(s.height),
        },
        min_size: Size {
            width:  tcss_opt_dim(s.min_width),
            height: tcss_opt_dim(s.min_height),
        },
        max_size: Size {
            width:  tcss_opt_dim(s.max_width),
            height: tcss_opt_dim(s.max_height),
        },
        padding: tcss_sides_to_taffy(s.padding),
        margin:  tcss_sides_to_taffy(s.margin),
        // Grid properties
        grid_template_columns: s.grid_columns.clone().unwrap_or_default(),
        grid_template_rows:    s.grid_rows.clone().unwrap_or_default(),
        ..Default::default()
    }
}

fn tcss_dim_to_taffy(d: TcssDimension) -> Dimension {
    match d {
        TcssDimension::Auto           => Dimension::Auto,
        TcssDimension::Length(n)      => Dimension::Length(n as f32),
        TcssDimension::Percent(p)     => Dimension::Percent(p / 100.0),
        TcssDimension::Fraction(fr)   => Dimension::Auto, // fr handled in grid tracks
    }
}
```

### Pattern 8: Dirty-Flag Bubbling

```rust
// Source: CONTEXT.md D-07
pub fn mark_widget_dirty(id: WidgetId, ctx: &mut AppContext) {
    ctx.dirty.insert(id, true);
    let mut current = ctx.parent.get(id).copied().flatten();
    while let Some(parent_id) = current {
        if ctx.dirty.get(parent_id) == Some(&true) {
            break;  // already dirty — all ancestors already marked
        }
        ctx.dirty.insert(parent_id, true);
        current = ctx.parent.get(parent_id).copied().flatten();
    }
}

pub fn clear_dirty_subtree(root: WidgetId, ctx: &mut AppContext) {
    ctx.dirty.insert(root, false);
    for &child in ctx.children[root].clone().iter() {
        clear_dirty_subtree(child, ctx);
    }
}
```

### Pattern 9: Render Loop Integration (App::run_async replacement)

```rust
// Source: CONTEXT.md code_context + app.rs Phase 1 pattern
loop {
    match rx.recv_async().await {
        Ok(AppEvent::Key(k)) if k.code == KeyCode::Tab => {
            ctx.advance_focus();      // DOM-order tab traversal
            mark_widget_dirty(ctx.focused_widget.unwrap_or(screen_id), &mut ctx);
        }
        Ok(AppEvent::Resize(cols, rows)) => {
            // All layout dirty on resize
            mark_widget_dirty(current_screen_id, &mut ctx);
            relayout_and_render(&mut ctx, &mut bridge, &mut terminal, cols, rows)?;
        }
        Ok(AppEvent::Key(_)) => {
            // Route to focused widget (Phase 3 extends this)
            relayout_and_render(&mut ctx, &mut bridge, &mut terminal, cols, rows)?;
        }
        _ => {}
    }
}

fn relayout_and_render(
    ctx: &mut AppContext,
    bridge: &mut TaffyBridge,
    terminal: &mut Terminal<impl Backend>,
    cols: u16,
    rows: u16,
) -> Result<()> {
    let screen_id = *ctx.screen_stack.last().unwrap();
    // Only re-sync + recompute if dirty
    if ctx.dirty.get(screen_id) == Some(&true) {
        bridge.sync_dirty_subtree(screen_id, ctx);
        bridge.compute_layout(screen_id, cols, rows);
        clear_dirty_subtree(screen_id, ctx);
    }
    // Always render (ratatui diffs — only changed cells go to terminal)
    terminal.draw(|f| {
        render_tree(screen_id, ctx, bridge, f.buffer_mut());
    })?;
    Ok(())
}
```

### Anti-Patterns to Avoid

- **Storing `&Arena` or `&mut Arena` inside a Widget struct.** Widgets must be arena-agnostic; all tree access goes through the `ctx` argument at call time.
- **Calling `ctx.arena[id].render(ctx, ...)` while holding a mutable borrow of `ctx`.** Obtain the immutable widget ref first (`let w = &ctx.arena[id]`), then call `w.render(ctx, ...)` — both borrows are immutable and Rust permits this.
- **Re-running cascade on the entire tree for every pseudo-class change.** Only recompute `ComputedStyle` for the widget whose pseudo-class set changed, then check if any layout-affecting property changed before propagating layout dirty.
- **Removing a Taffy parent node before its children.** Taffy panics if you remove a node that still has children. Walk the subtree bottom-up for removal.
- **Using `std::any::type_name::<T>()` for `widget_type_name()`.** It returns the full Rust module path. TCSS type selectors match short names only.
- **Not surfacing cssparser errors to users.** Collect `ParseError` from `RuleListParser` and log them with source location. Silent style failures are extremely hard to debug.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arena with generational indices | Pointer-based widget tree | `slotmap::DenseSlotMap` | Use-after-free prevention, cache-friendly storage, `SecondaryMap` companion — all solved |
| CSS Flexbox/Grid geometry | Custom layout algorithm | `taffy` 0.9.2 | Spec compliance takes years; Taffy covers all needed layouts including dock via absolute positioning |
| CSS color parsing | Color parser from scratch | `cssparser-color` 0.5.0 | 140+ named colors, hex variants, rgb/rgba — edge cases multiply fast; correctness is proven |
| CSS tokenization | Token scanner | `cssparser` 0.37.0 | CSS Syntax Level 3 has escapes, unicode identifiers, quoted strings, comments — all handled |

**Key insight:** The deliberately hand-rolled parts are limited to the selector grammar (~200 lines), the property dispatch table (~400 lines), and the cascade comparator (~50 lines) — all straightforward pattern matching on an already-tokenized stream. The hard parts (tokenization, color syntax, layout geometry) are delegated to proven libraries.

---

## Common Pitfalls

### Pitfall 1: SlotMap Simultaneous Borrow During Render

**What goes wrong:** Attempting `ctx.arena[id].render(ctx, area, buf)` panics if `ctx.arena` has already been mutably borrowed elsewhere in the call stack, or causes borrow-checker errors with mutable methods.

**Why it happens:** `DenseSlotMap` does not support simultaneous mutable iteration and element access. The `render()` call signature takes `&self` (widget) and `&AppContext` — both immutable. This is actually fine. The problem only arises if `render` needs `&mut ctx`.

**How to avoid:** Keep `render()` taking `&AppContext` (immutable). For lifecycle methods (`on_mount`, `on_unmount`) that need `&mut AppContext`, use a two-step pattern:
```rust
// 1. Remove widget from arena temporarily
let widget = ctx.arena.remove(id).unwrap();
// 2. Call mutable lifecycle method
widget.on_mount(&mut ctx, id);
// 3. Re-insert at same id (use a reserved slot approach or accept new id)
```
Or: call lifecycle methods before inserting into the arena (for `on_mount`) and after removing (for `on_unmount`), so the widget is never simultaneously in the arena and being mutably accessed.

**Warning signs:** Borrow checker error: "cannot borrow `ctx.arena` as mutable because it is also borrowed as immutable."

### Pitfall 2: f32 Layout Coordinates to u16 Terminal Cell Rounding Gaps

**What goes wrong:** Two adjacent widgets have a 1-cell visual gap between them despite having no margin/gap declared.

**Why it happens:** Taffy returns f32 positions. `floor()` on adjacent boundaries creates gaps: widget A ends at 40.7 (floor→40), widget B starts at 40.7 (floor→40) but has width 40.3 (floor→40) so B ends at 80 — gap at col 40 shows background.

**How to avoid:** Use `round()` for sizes, `floor()` for positions. For a row of flex children, compute the running integer total: position of child N = sum of rounded widths of children 0..N-1. Assign the last child whatever remains from the total container width. Check whether `taffy::TaffyTree::round_layout()` is available in 0.9.2 (if so, call it before reading layout values).

**Warning signs:** 1-cell black lines between containers at specific terminal widths.

### Pitfall 3: Full Cascade Recompute on Hover State Changes

**What goes wrong:** High CPU on every mouse-move event; the app sluggishly tracks the cursor.

**Why it happens:** Mouse move updates the `:hover` pseudo-class on a widget, triggering a full cascade recompute for all widgets.

**How to avoid:** Track which widget had `:hover` previously. On mouse move, only update pseudo-classes (and recompute `ComputedStyle`) for the old and new hover widget. Only propagate layout dirty if a layout-affecting property changed (width, height, padding, margin, display). Color/background changes do not require relayout — only a repaint.

**Warning signs:** CPU spikes measurable via `cargo flamegraph` correlated with mouse events.

### Pitfall 4: cssparser Error Recovery Is Silent

**What goes wrong:** A typo in TCSS (`backgroud: red`) causes the entire rule to be silently skipped with no user-visible error.

**Why it happens:** `RuleListParser` implements CSS error-recovery by design — it skips malformed rules and continues parsing. This is the CSS spec behavior, but it is silent without explicit error collection.

**How to avoid:** Collect the `Err` arm from `RuleListParser` iteration and emit warnings (to stderr, or to a future log widget). Include the source location (line/column) from `cssparser::ParseError::location` and the offending slice from the second element of the error tuple.

**Warning signs:** TCSS styles silently not applying; no compiler or runtime error.

### Pitfall 5: widget_type_name() Returns Full Module Path

**What goes wrong:** TCSS rule `Button { color: red; }` never applies to any Button widget.

**Why it happens:** `widget_type_name()` was implemented as `std::any::type_name::<Self>()` which returns `"textual_rs::widget::builtin::button::Button"`.

**How to avoid:** Every Widget impl must provide a literal short name: `fn widget_type_name(&self) -> &'static str { "Button" }`. Document this as a convention in the Widget trait's doc comment. Add an integration test that creates a Button, applies a Button type selector rule, and asserts the style is applied.

**Warning signs:** Type selectors in all TCSS rules have no effect.

### Pitfall 6: Screen Subtree Removal Leaves Dangling Taffy Nodes

**What goes wrong:** Popping a screen from the stack removes widgets from the arena, but `TaffyBridge.node_map` still holds `NodeId` references. Subsequent layout panics or produces stale cached `Rect` values.

**Why it happens:** Arena removal and Taffy tree removal are not synchronized.

**How to avoid:** Implement `TaffyBridge::remove_subtree(root: WidgetId, ctx: &AppContext)` that walks the subtree bottom-up, calls `self.tree.remove(node)` for each child before the parent, and removes the `node_map` entry. Call this before removing any widget from the arena. Taffy requires children to be detached before a parent node is removed.

**Warning signs:** Panic in taffy with "node has children" or "key not found."

---

## Code Examples

### SlotMap Arena with SecondaryMap
```rust
// Source: slotmap 1.1.1 docs — https://docs.rs/slotmap/1.1.1/slotmap/
use slotmap::{DenseSlotMap, SecondaryMap, new_key_type};

new_key_type! { pub struct WidgetId; }

let mut arena: DenseSlotMap<WidgetId, Box<dyn Widget>> = DenseSlotMap::with_key();
let id = arena.insert(Box::new(MyWidget::new()));

// SecondaryMap shares validity with its parent DenseSlotMap
let mut children: SecondaryMap<WidgetId, Vec<WidgetId>> = SecondaryMap::new();
children.insert(id, vec![]);

// Safe access — id is still valid
if let Some(widget) = arena.get(id) {
    // widget: &Box<dyn Widget>
}
```

### Taffy Grid Layout
```rust
// Source: taffy 0.9.2 docs — https://docs.rs/taffy/0.9.2/taffy/
use taffy::prelude::*;

let mut tree: TaffyTree<WidgetId> = TaffyTree::new();

// Grid container: 3 columns (1fr 2fr 1fr), 2 rows (auto 1fr)
let container = tree.new_leaf(Style {
    display: Display::Grid,
    grid_template_columns: vec![
        TrackSizingFunction::Single(NonRepeatedTrackSizingFunction {
            min: MinTrackSizingFunction::Auto,
            max: MaxTrackSizingFunction::Fraction(1.0),
        }),
        // ... etc — use taffy::style::fr() helper
    ],
    size: Size { width: Dimension::Length(80.0), height: Dimension::Length(24.0) },
    ..Default::default()
})?;
```

### cssparser-color Color Extraction
```rust
// Source: cssparser-color 0.5.0 — https://docs.rs/cssparser-color/0.5.0
// Note: verify exact API against 0.5.0 docs during plan 02-03 implementation
use cssparser_color::Color;
use cssparser::{Parser, ParserInput};

fn parse_tcss_color(s: &str) -> Option<ratatui::style::Color> {
    let mut input = ParserInput::new(s);
    let mut p = Parser::new(&mut input);
    match Color::parse(&mut p).ok()? {
        Color::Rgba(rgba) => {
            let r = (rgba.red   * 255.0) as u8;
            let g = (rgba.green * 255.0) as u8;
            let b = (rgba.blue  * 255.0) as u8;
            Some(ratatui::style::Color::Rgb(r, g, b))
        }
        _ => None,
    }
}
```

### Border Style to Ratatui BorderType
```rust
// Source: ratatui 0.30.0 docs — https://docs.rs/ratatui/0.30.0/ratatui/widgets/block/enum.BorderType.html
use ratatui::widgets::BorderType;

pub enum TcssBorderStyle { None, Solid, Rounded, Heavy, Double, Ascii }

fn tcss_border_to_ratatui(style: TcssBorderStyle) -> Option<BorderType> {
    match style {
        TcssBorderStyle::None    => None,
        TcssBorderStyle::Solid   => Some(BorderType::Plain),
        TcssBorderStyle::Rounded => Some(BorderType::Rounded),
        TcssBorderStyle::Heavy   => Some(BorderType::Thick),
        TcssBorderStyle::Double  => Some(BorderType::Double),
        TcssBorderStyle::Ascii   => Some(BorderType::Plain),  // fallback, no ASCII-only type
    }
}
```

### Focus Traversal (DOM-Order)
```rust
// Source: CONTEXT.md D-14
pub fn advance_focus(ctx: &mut AppContext) {
    let screen = match ctx.screen_stack.last() {
        Some(&s) => s,
        None => return,
    };
    let focusable: Vec<WidgetId> = collect_focusable_dfs(screen, ctx);
    if focusable.is_empty() { return; }

    let next = match ctx.focused_widget {
        None => focusable[0],
        Some(current) => {
            let pos = focusable.iter().position(|&id| id == current).unwrap_or(0);
            focusable[(pos + 1) % focusable.len()]
        }
    };
    // Clear :focus from old widget
    if let Some(old) = ctx.focused_widget {
        ctx.pseudo_classes[old].remove(PseudoClass::Focus);
        mark_widget_dirty(old, ctx);
    }
    // Set :focus on new widget
    ctx.pseudo_classes[next].insert(PseudoClass::Focus);
    ctx.focused_widget = Some(next);
    mark_widget_dirty(next, ctx);
}

fn collect_focusable_dfs(id: WidgetId, ctx: &AppContext) -> Vec<WidgetId> {
    let mut result = Vec::new();
    if ctx.arena[id].can_focus() {
        result.push(id);
    }
    for &child in &ctx.children[id] {
        result.extend(collect_focusable_dfs(child, ctx));
    }
    result
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| cssparser 0.35 (prior research documented) | cssparser 0.37.0 | 2025 | Minor API; cssparser-color is now a distinct companion crate (0.5.0) |
| selectors crate as standalone | Part of servo/stylo monorepo | May 2024 | Do not depend on stylo; hand-rolled selector matching is the right call |
| Taffy 0.6-era API with separate NodeId types | Taffy 0.9.2 unified `TaffyTree<NodeData>` API | 2024 | Generic user data (we store `WidgetId`) is a first-class feature in current Taffy |

**Deprecated/outdated:**
- `tui-rs`: RUSTSEC-2023-0049. Use ratatui 0.30.0.
- `lightningcss` as the TCSS parser: designed for browsers, not TUI subsets. Use cssparser + hand-rolled.
- Ratatui Cassowary solver for Textual-style layouts: cannot express CSS Grid or dock. Taffy is required.
- `servo/rust-selectors` standalone crate: archived, now `servo/stylo`. Too heavy for TCSS.

---

## Open Questions

1. **Taffy `round_layout()` availability in 0.9.2**
   - What we know: Some Taffy versions have a `round_layout` function that correctly rounds f32 geometry to integer cells, preventing 1-cell gaps between adjacent widgets.
   - What's unclear: Whether `TaffyTree::round_layout()` exists in exactly 0.9.2 and what its signature is.
   - Recommendation: In Plan 02-02, read `taffy 0.9.2` docs before implementing `compute_layout`. If `round_layout` is available, use it. If not, implement the manual rounding described in Pitfall 2.

2. **cssparser-color 0.5.0 exact RGBA struct fields**
   - What we know: `cssparser-color` provides `Color::parse()` and a `Color::Rgba` variant containing red/green/blue/alpha as f32.
   - What's unclear: The exact field names and whether the alpha component is included in the base case or only in `rgba()` syntax.
   - Recommendation: In Plan 02-03, verify against the `cssparser-color 0.5.0` docs (docs.rs) before writing the color parsing code.

3. **DenseSlotMap re-insertion after temporary removal for on_mount**
   - What we know: The "remove then re-insert" pattern for calling `&mut AppContext` lifecycle methods is safe but assigns a new WidgetId on re-insert with standard `insert()`.
   - What's unclear: Whether slotmap 1.1.1 has an API to re-insert at a specific key (to preserve the WidgetId).
   - Recommendation: In Plan 02-01, evaluate whether `SlotMap::insert_with_key()` or a reserved-slot approach is available. Alternatively, restructure lifecycle hooks to accept only what they need rather than full `&mut AppContext`, avoiding the need to remove the widget from the arena.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable toolchain | All compilation | Yes | 1.94.0 | — |
| cargo | Build + test | Yes | 1.94.0 | — |
| slotmap 1.1.1 | TREE-01 widget arena | Needs `cargo add` | — (not yet in Cargo.toml) | — (no alternative; required) |
| taffy 0.9.2 | LAYOUT-01 through LAYOUT-07 | Needs `cargo add` | — | — (no alternative; required) |
| cssparser 0.37.0 | CSS-01 TCSS parser | Needs `cargo add` | — | — (no alternative; required) |
| cssparser-color 0.5.0 | CSS-07 color parsing | Needs `cargo add` | — | Hand-roll named colors only (LIMITED) |
| ratatui 0.30.0 | Rendering | Yes (Phase 1 dep) | 0.30.0 | — |
| crossterm 0.29.0 | Terminal backend | Yes (Phase 1 dep) | 0.29.0 | — |

**Missing dependencies with no fallback:**
- `slotmap`, `taffy`, `cssparser` — required; add to Cargo.toml as Wave 0 of Plan 02-01.

**Missing dependencies with fallback:**
- `cssparser-color` — if omitted, can hand-roll basic named colors (16 ANSI + a few extras) but rgb()/rgba() hex support becomes custom code. Strongly recommend using the crate.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | none (standard Rust test runner) |
| Quick run command | `cargo test -p textual-rs -- --test-output immediate` |
| Full suite command | `cargo test -p textual-rs` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TREE-01 | Insert widget into DenseSlotMap, retrieve by WidgetId, verify generational safety | unit | `cargo test -p textual-rs widget::context::tests -- --test-output immediate` | No — Wave 0 |
| TREE-02 | Widget trait compile-check: dyn Widget is object-safe, all methods callable | unit | `cargo test -p textual-rs widget::tests::trait_object_safety` | No — Wave 0 |
| TREE-03 | Push screen to stack, verify it is the active screen; pop screen, verify previous is active | unit | `cargo test -p textual-rs widget::tree::tests::screen_stack` | No — Wave 0 |
| TREE-04 | mount() adds child to arena and wires parent/children; unmount() removes subtree | unit | `cargo test -p textual-rs widget::tree::tests::mount_unmount` | No — Wave 0 |
| TREE-05 | Tab key advances focus through can_focus() widgets in DFS order; :focus pseudo-class updates | unit | `cargo test -p textual-rs widget::tree::tests::focus_traversal` | No — Wave 0 |
| LAYOUT-01 | TaffyBridge computes layout for a flex column screen with two children | unit | `cargo test -p textual-rs layout::bridge::tests::flex_column` | No — Wave 0 |
| LAYOUT-02 | Vertical container: children stacked; horizontal container: children side by side | unit | `cargo test -p textual-rs layout::bridge::tests::vertical_horizontal` | No — Wave 0 |
| LAYOUT-03 | Grid 2x2 layout assigns correct Rect to each cell | unit | `cargo test -p textual-rs layout::bridge::tests::grid_2x2` | No — Wave 0 |
| LAYOUT-04 | dock: top widget gets Rect pinned to top of screen; remaining child fills rest | unit | `cargo test -p textual-rs layout::bridge::tests::dock_top` | No — Wave 0 |
| LAYOUT-05 | Two widgets with 1fr and 2fr width split an 80-col container into 26/54 cols | unit | `cargo test -p textual-rs layout::bridge::tests::fractional_units` | No — Wave 0 |
| LAYOUT-06 | Width: 10 → Rect.width=10; width: 50% at 80 cols → width=40; width: auto fills remaining | unit | `cargo test -p textual-rs layout::bridge::tests::sizing_modes` | No — Wave 0 |
| LAYOUT-07 | Mark widget dirty, mark ancestor dirty; clean subtree, verify ancestors unmarked | unit | `cargo test -p textual-rs layout::tests::dirty_flag_bubbling` | No — Wave 0 |
| CSS-01 | Parse valid TCSS string, get correct rule count; malformed rule produces error, not panic | unit | `cargo test -p textual-rs css::parser::tests::parse_valid_tcss` | No — Wave 0 |
| CSS-02 | Type/class/ID selectors match correct widgets; non-matching selectors return false | unit | `cargo test -p textual-rs css::selector::tests::selector_matching` | No — Wave 0 |
| CSS-03 | ID rule overrides class rule; class rule overrides type rule; source order breaks ties | unit | `cargo test -p textual-rs css::cascade::tests::specificity_order` | No — Wave 0 |
| CSS-04 | Inline style `color: blue` overrides stylesheet `color: red` on same widget | unit | `cargo test -p textual-rs css::cascade::tests::inline_wins` | No — Wave 0 |
| CSS-05 | Widget with :focus pseudo-class matches `:focus` selector; without it does not | unit | `cargo test -p textual-rs css::selector::tests::pseudo_class_focus` | No — Wave 0 |
| CSS-06 | Parse all 19 properties from CSS-06; each produces the correct ComputedStyle field | unit | `cargo test -p textual-rs css::property::tests::all_properties_parse` | No — Wave 0 |
| CSS-07 | `red`, `#f00`, `#ff0000`, `rgb(255,0,0)`, `rgba(255,0,0,1.0)` all parse to same Rgb | unit | `cargo test -p textual-rs css::property::tests::color_syntax_variants` | No — Wave 0 |
| CSS-08 | `border: solid red` → BorderType::Plain + red color; `rounded` → Rounded; `none` → no border | unit | `cargo test -p textual-rs css::property::tests::border_styles` | No — Wave 0 |
| CSS-09 | Widget default_css() prepended at lowest specificity; user stylesheet color overrides it | unit | `cargo test -p textual-rs css::cascade::tests::default_css_priority` | No — Wave 0 |
| Integration | IRC-style layout (header dock:top, sidebar, main, input dock:bottom) renders correct Rects | integration | `cargo test -p textual-rs -- irc_layout` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p textual-rs -- --test-output immediate` (full suite, fast — all unit, no I/O)
- **Per wave merge:** `cargo test -p textual-rs` (same suite + integration tests)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- `crates/textual-rs/src/widget/mod.rs` — Widget trait, WidgetId, EventPropagation
- `crates/textual-rs/src/widget/context.rs` — AppContext struct + unit tests (TREE-01, TREE-02)
- `crates/textual-rs/src/widget/tree.rs` — mount/unmount, focus traversal (TREE-03, TREE-04, TREE-05)
- `crates/textual-rs/src/layout/bridge.rs` — TaffyBridge (LAYOUT-01 through LAYOUT-07)
- `crates/textual-rs/src/layout/mod.rs` — public layout API
- `crates/textual-rs/src/layout/hit_map.rs` — MouseHitMap
- `crates/textual-rs/src/css/types.rs` — ComputedStyle, all enums
- `crates/textual-rs/src/css/parser.rs` — TcssRuleParser, parse_stylesheet()
- `crates/textual-rs/src/css/selector.rs` — Selector enum, selector_matches()
- `crates/textual-rs/src/css/property.rs` — PropertyParser, TcssValue
- `crates/textual-rs/src/css/cascade.rs` — resolve_cascade(), Specificity
- `crates/textual-rs/src/css/mod.rs` — public CSS API
- Cargo.toml additions: `slotmap = "1.1.1"`, `taffy = "0.9.2"`, `cssparser = "0.37.0"`, `cssparser-color = "0.5.0"`

---

## Sources

### Primary (HIGH confidence)
- `.planning/research/CSS_LAYOUT.md` — Taffy 0.9.2 API, cssparser usage, selector matching patterns, TCSS property map (verified 2026-03-24 against official docs)
- `.planning/codebase/ARCHITECTURE.md` — Python Textual architecture: cascade, compositor, widget tree (2026-03-24 analysis)
- `cargo search slotmap` / `cargo search taffy` / `cargo search cssparser` — version verification (2026-03-25)
- `crates/textual-rs/src/app.rs` — Phase 1 integration points confirmed
- CONTEXT.md D-01 through D-14 — All architectural decisions locked; research confirms feasibility

### Secondary (MEDIUM confidence)
- `.planning/research/ECOSYSTEM.md` — Ratatui ecosystem, comparison with cursive/tui-realm (2026-03-24)
- `.planning/codebase/CONVENTIONS.md` — Python Textual naming patterns for mapping to Rust equivalents
- DioxusLabs/taffy GitHub (via prior research) — taffy::TaffyTree generic user data API
- servo/rust-cssparser GitHub (via prior research) — QualifiedRuleParser interface

### Tertiary (LOW confidence — verify during implementation)
- cssparser-color 0.5.0 exact Rgba struct field names — verify at docs.rs during Plan 02-03
- Taffy `round_layout()` existence in 0.9.2 — verify at docs.rs during Plan 02-02

---

## Project Constraints (from CLAUDE.md)

No `CLAUDE.md` found in the project root. The project constraints are instead expressed through:

1. **PROJECT.md Quality constraint:** "No shortcuts — correctness and safety over speed of development"
2. **PROJECT.md TDD constraint:** "Unit tests written before implementation code (TDD approach)" — tests are written before implementation in each plan
3. **PROJECT.md language constraint:** "Rust — stable channel, no nightly-only features" — all dependencies must compile on stable Rust 1.94.0
4. **PROJECT.md cross-platform constraint:** "Windows/macOS/Linux from day one — no platform-specific assumptions" — `slotmap`, `taffy`, `cssparser` are all pure Rust with no platform-specific code
5. **Phase 1 established patterns:** `anyhow::Result` for error propagation, `tokio` LocalSet (no `Send` requirement on widget state), `TestBackend` integration tests

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified via `cargo search` on 2026-03-25
- Architecture: HIGH — AppContext pattern, TaffyBridge design, and cssparser usage all verified against prior research and official docs
- Pitfalls: HIGH — SlotMap borrow conflicts and Taffy rounding are well-documented issues in prior research; cssparser error recovery is a CSS spec behavior

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable crates — 30-day window is conservative)
