# CSS-Like Styling and Layout Engines in Rust

**Domain:** TUI library with CSS-inspired styling (Textual-inspired)
**Researched:** 2026-03-24
**Overall confidence:** HIGH (core layout), MEDIUM (selector/cascade implementation patterns)

---

## 1. CSS Parser Crates in Rust

### Recommendation: Hand-roll a TCSS tokenizer over `cssparser` primitives

The Rust ecosystem has three realistic options for parsing a CSS subset:

| Crate | Version | Role | Verdict |
|-------|---------|------|---------|
| `cssparser` | 0.35.0 (Mar 2025) | CSS Syntax Level 3 tokenizer/component-value parser | Use as tokenizer foundation |
| `lightningcss` | 1.0.0-alpha.37 | Full CSS parser + transformer (browser-grade) | Too heavy; designed for web CSS |
| Hand-rolled (nom/pest) | — | Custom TCSS grammar | Good for a tight subset |

**`cssparser` (servo/rust-cssparser)** is the right foundation. It handles the hard part — tokenization and component value parsing — but deliberately does not parse individual properties or selectors. You implement those steps yourself. This is exactly the right split for TCSS: use `cssparser` to tokenize the `.tcss` file and parse block structure, then dispatch on property names with your own `match` arms.

`lightningcss` (parcel-bundler) is overkill. It builds on `cssparser` and adds a full web CSS property AST — vendor prefixes, transforms, media queries, hundreds of properties. None of that is needed for TCSS. Its API is also oriented toward stylesheets for browsers, not custom TUI property sets. Skip it unless you want to parse standard CSS colors (and even then, `cssparser` includes a color helper).

**What `cssparser` provides for TCSS:**
- CSS Syntax Level 3 compliant tokenizer (raw tokens never materialized as strings until you request them)
- Component value parser (`{...}` block, `(...)` function, token sequence)
- `parse_important!` helper for `!important`
- Color parsing helpers (`cssparser::color`)
- `An+B` expression parser (for `:nth-child` etc.)
- `ParseError` type with source location

**Bottom line:** Use `cssparser` for lexing and block structure. Write a thin property dispatcher (`match property_name { "color" => parse_color(...), ... }`) on top. Do not use `lightningcss`.

---

## 2. Taffy Layout Engine

**Crate:** `taffy` v0.9.2 (November 2025) — [DioxusLabs/taffy](https://github.com/DioxusLabs/taffy)
**Confidence:** HIGH

### What Taffy Provides

Taffy is the de facto standard Rust layout crate. It implements three CSS layout algorithms:

- **Flexbox** (default feature, fully spec-compliant)
- **CSS Grid** (default feature, including `grid-template-areas`, named lines as of v0.9)
- **Block layout** (default feature)

It is used in production by Servo, Bevy, Zed, Lapce, Slint, and iocraft (a TUI framework). The API is tree-based:

```rust
let mut tree = TaffyTree::new();

let child = tree.new_leaf(Style {
    size: Size { width: Dimension::Percent(0.5), height: Dimension::Auto },
    ..Default::default()
})?;

let root = tree.new_with_children(Style {
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    ..Default::default()
}, &[child])?;

tree.compute_layout(root, Size::MAX_CONTENT)?;
let layout = tree.layout(child)?; // x, y, width, height
```

### Style Properties (v0.9 `Style` struct, 39 fields)

**Box model:** `size`, `min_size`, `max_size`, `margin`, `padding`, `border`, `aspect_ratio`, `box_sizing`

**Flexbox:** `flex_direction`, `flex_wrap`, `flex_basis`, `flex_grow`, `flex_shrink`, `align_items`, `align_self`, `align_content`, `justify_content`, `justify_items`, `justify_self`, `gap`

**Grid:** `grid_template_rows`, `grid_template_columns`, `grid_auto_rows`, `grid_auto_columns`, `grid_auto_flow`, `grid_row`, `grid_column`, `grid_template_areas`, `grid_template_row_names`, `grid_template_column_names`

**Positioning:** `display`, `position`, `inset`, `overflow`

**Text:** `text_align`

### TUI-Relevant Layout Patterns

**Vertical/Horizontal dock layouts:** Use `Display::Flex` with `FlexDirection::Column` (vertical stack) or `FlexDirection::Row` (horizontal stack). This maps directly to Textual's `layout: vertical` and `layout: horizontal`.

**Pinned/docked edges (Textual's `dock: top/bottom/left/right`):** Use `position: Position::Absolute` with appropriate `inset` values (e.g., `inset.top = 0, inset.left = 0, inset.right = 0` for a top-docked header). Absolute-positioned children in Taffy are pulled out of normal flow, exactly like CSS.

**Grid layout:** `Display::Grid` with `grid_template_columns`/`grid_template_rows`. For Textual's `grid-columns` property, use `TrackSizingFunction::Repeat` with fixed or `fr` (fraction) units.

**Fractional sizing (Textual's `fr` units):** Use `fr()` in grid tracks or `flex_grow` in flex children. Taffy natively supports fractional allocation.

**Fixed/percentage sizing:** `Dimension::Length(n)` and `Dimension::Percent(p)` map to TCSS integer cells and `%` values.

### Ratatui vs Taffy

Ratatui uses the **Cassowary constraint solver** (via `kasuari` crate). It only supports 1D splits (either horizontal or vertical, nested). It does not support CSS Grid or true 2D layouts natively. There are community proofs-of-concept using Taffy with ratatui's `Rect` type, but they are not merged into ratatui core.

| Capability | Ratatui Constraints | Taffy |
|-----------|---------------------|-------|
| 1D flex split | Yes (`Fill`, `Percentage`, `Length`) | Yes |
| 2D grid | No | Yes |
| Absolute positioning | No | Yes |
| `min`/`max` constraints | Yes | Yes |
| `fr` units | Partial (`Fill`) | Yes (native `fr` type) |
| `dock` to edges | No | Yes (absolute + inset) |
| Nested layouts | Manual nesting | Tree structure |

**Verdict:** For a Textual-inspired TUI library, Taffy is strongly preferred over Ratatui's constraint system. Taffy can express every Textual layout concept. Ratatui's Cassowary solver cannot.

---

## 3. Morphorm vs Taffy

**Crate:** `morphorm` ([vizia/morphorm](https://github.com/vizia/morphorm)) — last updated April 2025, maintained as part of the Vizia GUI framework.

### What Morphorm Does

Morphorm implements a **single-pass, depth-first layout algorithm** with a simpler conceptual model than Flexbox. Its key primitives:

- **Layout type per container:** `Row`, `Column`, or `Grid`
- **Sizing units:** `Pixels`, `Percentage`, `Stretch` (proportional), `Auto` (hug content)
- **Spacing:** padding, gap between children, margins
- **Alignment:** 9-position grid (top-left through bottom-right)
- **Positioning:** `Relative` (in-flow) or `Absolute` (independent)
- **Constraints:** `min`/`max` for both size and gaps

### Morphorm vs Taffy Comparison

| Criterion | Taffy | Morphorm |
|-----------|-------|----------|
| Algorithm | CSS Flexbox + Grid (spec-compliant) | Custom one-pass |
| Spec compliance | Full CSS Flexbox/Grid | No CSS spec — custom model |
| CSS Grid support | Yes | Yes (basic) |
| Complexity to use | Moderate (many style fields) | Low (simpler API) |
| Named grid areas | Yes (v0.9+) | No |
| Absolute positioning | Full CSS model | Yes |
| Adoption | Very broad (Servo, Bevy, Zed, Slint) | Narrow (Vizia only) |
| Integration with CSS text | Natural (same terminology) | Requires mapping |
| Active maintenance | High (DioxusLabs) | Low-medium (Vizia sub-project) |

### Recommendation: Use Taffy

For a TCSS-driven TUI, Taffy wins decisively:

1. TCSS properties use CSS terminology (`display: flex`, `flex-direction`, `grid-template-columns`). Taffy's `Style` struct fields map 1:1 to these property names.
2. Taffy is battle-tested at scale. Morphorm is only used by Vizia.
3. Morphorm's "one-pass" simplicity advantage disappears once you need CSS Grid or full Flexbox semantics — Textual uses both.
4. Taffy's `grid_template_areas` (v0.9) directly supports Textual's grid layout DSL.

---

## 4. CSS Cascade / Specificity / Selector Matching in Rust

### Architecture Pattern

There is no lightweight, embeddable, production-ready CSS selector matching crate for custom non-DOM element trees. The Servo `selectors` crate (now in `servo/stylo`) is the most complete implementation but requires you to implement the `Element` trait against your widget tree — significant boilerplate designed for browser use. The crate was archived as a standalone repo in May 2024 and is now part of `servo/stylo`.

The practical approach for TCSS is to **build a small custom selector engine**, following the well-documented Robinson browser engine pattern:

#### Data Structures

```rust
// Parsed TCSS rule
struct Rule {
    selectors: Vec<Selector>,  // ordered most-specific first
    declarations: Vec<Declaration>,
}

struct Declaration {
    name: String,
    value: Value,
    important: bool,
}

// Selector types TCSS needs
enum Selector {
    Type(String),          // Button, Label
    Class(String),         // .focused
    Id(String),            // #submit
    Universal,             // *
    PseudoClass(PseudoClass),  // :hover, :focus, :disabled
    Descendant(Box<Selector>, Box<Selector>),  // A B
    Child(Box<Selector>, Box<Selector>),       // A > B
    And(Vec<Selector>),    // compound: Button.focused
}

// Specificity as (a, b, c) tuple
struct Specificity(u32, u32, u32);
```

#### Cascade Algorithm

```
1. Collect all rules from all stylesheets in order (author sheets)
2. For each widget node (depth-first):
   a. Find all matching rules
   b. For each rule, record (specificity, source_order, declarations)
   c. Sort matched rules: specificity ASC, then source_order ASC
      (so higher specificity / later rules win by overwriting)
   d. Apply declarations to ComputedStyle in order
   e. Apply inline styles last (always highest priority)
3. Inherit inheritable properties from parent ComputedStyle
```

#### Specificity Calculation

Standard CSS specificity `(a, b, c)`:
- `a` = count of ID selectors (`#id`)
- `b` = count of class selectors, pseudo-classes, attribute selectors
- `c` = count of type selectors, pseudo-elements

Compare as a 3-tuple: `(1,0,0)` beats `(0,99,0)`. For TCSS, this is sufficient.

#### Selector Matching

```rust
fn matches(selector: &Selector, widget: &WidgetNode, dom: &Dom) -> bool {
    match selector {
        Selector::Type(name) => widget.widget_type() == name,
        Selector::Class(cls) => widget.has_class(cls),
        Selector::Id(id) => widget.id() == Some(id),
        Selector::Universal => true,
        Selector::PseudoClass(pc) => widget.has_pseudo_class(pc),
        Selector::Descendant(ancestor, subject) => {
            matches(subject, widget, dom) &&
            dom.ancestors(widget).any(|a| matches(ancestor, a, dom))
        }
        Selector::Child(parent_sel, subject) => {
            matches(subject, widget, dom) &&
            dom.parent(widget).map_or(false, |p| matches(parent_sel, p, dom))
        }
        Selector::And(parts) => parts.iter().all(|p| matches(p, widget, dom)),
    }
}
```

#### What the `selectors` crate from Servo provides

If you want to avoid writing selector matching from scratch: the `selectors` crate (0.25.0, part of `servo/stylo`) provides full CSS4 selector parsing and matching. You implement the `Element` trait on your widget node type. The trait requires: tag name, id, classes, parent/sibling navigation, pseudo-class matching. This is ~300 lines of boilerplate but gives you free `:nth-child`, `:not()`, `:is()`, `:has()`, etc.

**Verdict:** For the initial TCSS implementation, hand-roll selector matching. The TCSS selector surface is small (type, class, id, pseudo-class, descendant, child). Add `selectors` crate later if you need advanced selectors.

---

## 5. How Rust GUI Frameworks Handle Styling

### Iced: Trait-based Appearance System

Iced's styling is entirely **trait-based with no external stylesheet**. Each widget defines an `Appearance` struct (colors, border radii, shadow) and a `StyleSheet` trait with methods like `active()`, `hovered()`, `pressed()`. The `Theme` type implements `StyleSheet` for all widgets.

```rust
// Widget styling: pass a closure that maps Theme -> Appearance
button(label).style(|theme: &Theme, status| {
    match status {
        Status::Hovered => Appearance { background: theme.palette().primary, ... },
        _ => Appearance::default(),
    }
})
```

**Applicability to TCSS:** Iced's approach is compile-time and widget-specific. No runtime cascade, no selectors, no stylesheet files. Not applicable.

### egui: Immediate mode, global `Visuals`

egui has a flat `Visuals` struct (colors, fonts, spacing, rounding) set globally on the `Context`. No widget-level CSS, no selectors. Immediate-mode rendering means style is per-call, not per-widget-identity.

**Applicability to TCSS:** None. egui's philosophy is opposite to a cascade-based system.

### Vizia: CSS stylesheets + morphorm layout

Vizia is the closest existing Rust analog to Textual's styling model. It supports:
- External `.css` files or inline CSS strings loaded at runtime
- Selector matching (type, class, id, compound selectors)
- Hot-reload in debug mode (F5)
- Inline style properties via modifier methods (`.background_color(Color::red())`)
- Standard properties: `background-color`, `border-width`, `border-color`, `border-radius`, `width`, `height`, `alignment`, `gap`
- Layout powered by Morphorm (column/row/grid)

**Applicability to TCSS:** Vizia's CSS-stylesheet-to-widget architecture is the most directly applicable. Key lessons:
1. Store parsed stylesheets separately from the widget tree
2. Re-run selector matching on every structural change (widget added/removed/class changed)
3. Use modifier methods for inline styles, overriding cascade
4. Keep hot-reload for development productivity

### Dioxus: Web-style CSS passthrough + Taffy layout

Dioxus (web target) delegates styling to the browser's CSS engine. For native targets, it uses Taffy for layout and component-level style props. `dioxus_style` scopes CSS to components at compile time.

**Applicability to TCSS:** Taffy integration pattern is directly reusable. The iocraft TUI framework (React-inspired, uses Taffy for flexbox) demonstrates a clean Taffy integration for terminal rendering.

### Key Pattern: Parallel Style Tree

All production engines use a **computed style cache** parallel to the widget tree:
- Widget tree: identity, hierarchy, content
- Computed style tree: resolved CSS values after cascade
- Layout tree: Taffy node IDs + computed geometry

These are kept in sync via change tracking (dirty flags), not recalculated from scratch every frame.

---

## 6. Minimal TCSS Parser Requirements

### Supported Selectors

| Selector | Example | Priority |
|----------|---------|----------|
| Type selector | `Button { }` | Must-have |
| Class selector | `.focused { }` | Must-have |
| ID selector | `#submit { }` | Must-have |
| Universal | `* { }` | Should-have |
| Descendant combinator | `Screen Button { }` | Should-have |
| Child combinator | `Screen > Button { }` | Nice-to-have |
| Pseudo-class | `:hover`, `:focus`, `:disabled` | Must-have |
| `:dark`, `:light` | Theme variants | Must-have |
| `:first-child`, `:last-child` | Structural | Should-have |

### Supported Properties

| Property | Values | Maps to Taffy |
|----------|--------|---------------|
| `display` | `block`, `flex`, `grid`, `none` | `Style::display` |
| `layout` | `vertical`, `horizontal`, `grid` | `flex_direction` / `Display::Grid` |
| `dock` | `top`, `bottom`, `left`, `right` | `position: Absolute` + `inset` |
| `width` | integer, `%`, `fr`, `auto`, `1fr` | `Style::size.width` |
| `height` | integer, `%`, `fr`, `auto` | `Style::size.height` |
| `min-width` / `max-width` | integer, `%` | `Style::min_size` / `max_size` |
| `min-height` / `max-height` | integer, `%` | same |
| `padding` | 1-4 integers | `Style::padding` |
| `margin` | 1-4 integers | `Style::margin` |
| `border` | `none` or `<style> <color>` | Custom border rendering |
| `color` | named, hex, rgb() | Rendered text color |
| `background` | named, hex, rgb() | Cell background color |
| `flex-direction` | `row`, `column` | `Style::flex_direction` |
| `flex-grow` | number | `Style::flex_grow` |
| `align-items` | `start`, `center`, `end`, `stretch` | `Style::align_items` |
| `justify-content` | `start`, `center`, `end`, `space-between` | `Style::justify_content` |
| `grid-columns` | `1fr 1fr`, `auto 1fr` | `grid_template_columns` |
| `grid-rows` | `auto 1fr` | `grid_template_rows` |
| `overflow` | `hidden`, `scroll`, `auto` | `Style::overflow` + scrollbar logic |

### TCSS-Specific (Non-Standard CSS) Properties

| Property | Values | Notes |
|----------|--------|-------|
| `text-style` | `bold`, `italic`, `underline`, `reverse` | Terminal text attributes |
| `text-align` | `left`, `center`, `right` | Rendered inside widget |
| `content-align` | `<horiz> <vert>` | Widget content placement |
| `scrollbar-visibility` | `hidden`, `auto`, `visible` | Terminal scrollbar control |
| `layout` | `vertical`, `horizontal`, `grid` | TCSS shorthand for flex-direction |

### Parser Architecture

```
.tcss file
    ↓
cssparser::Parser (tokenization + block structure)
    ↓
RuleParser (implements cssparser::QualifiedRuleParser + AtRuleParser)
    ↓
SelectorParser → Vec<Selector> with Specificity
PropertyParser → Vec<Declaration> (name: String, value: TcssValue)
    ↓
Stylesheet { rules: Vec<Rule>, variables: HashMap<String, TcssValue> }
```

The `RuleParser` approach lets `cssparser` handle all the syntactic structure (`{...}` blocks, semicolons, `!important`, comments) while your code only needs to match on property names and parse values.

---

## 7. Ratatui Layout Constraints vs CSS Flexbox/Grid

### Ratatui's Constraint System

Ratatui uses the Cassowary constraint solver. Layouts are 1-dimensional splits:

```rust
Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Fill(1), Constraint::Length(1)])
    .split(area)
```

Constraint types:
- `Length(n)` — fixed terminal cells
- `Percentage(p)` — relative to parent
- `Ratio(a, b)` — fractional (e.g., `1/3`)
- `Min(n)` / `Max(n)` — bounded
- `Fill(weight)` — proportional fill (like `flex-grow`)

The `Flex` enum (added in Ratatui v0.26) adds spacing modes: `Start`, `End`, `Center`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly` — loosely mirroring `justify-content`.

### Gap Analysis: Ratatui vs CSS

| CSS Concept | Ratatui Support | Taffy Support |
|-------------|----------------|---------------|
| `flex-direction: column` | Yes (Direction::Vertical) | Yes |
| `flex-direction: row` | Yes (Direction::Horizontal) | Yes |
| `flex-grow` | Partial (`Fill(weight)`) | Yes |
| `flex-shrink` | No | Yes |
| `flex-basis` | No | Yes |
| `justify-content` (spacing modes) | Partial (`Flex` enum) | Yes (full) |
| `align-items` | No | Yes |
| CSS Grid (2D) | No | Yes |
| Absolute positioning | No | Yes |
| `min-width` / `max-width` | Partial (`Min`, `Max`) | Yes |
| `padding` / `margin` | No (widget responsibility) | Yes (in layout pass) |
| `gap` between items | No | Yes |
| `dock` to edges | No | Yes (absolute + inset) |

### Verdict

Ratatui's constraint system is adequate for simple splits but is not a CSS layout engine. It cannot represent:
- 2D grid layouts
- Docked panels
- `align-items` / cross-axis alignment
- `flex-shrink` / proper flex item sizing
- `gap` between items

For a Textual-inspired TUI library, Taffy is necessary. The approach is:
1. Use Taffy for geometry computation (gives you `x, y, width, height` for every node)
2. Convert Taffy's output `Rect` values to ratatui `Rect` values if using ratatui for rendering
3. Or render directly to a crossterm/termion backend using the computed geometry

---

## Architecture Recommendation

### What to Build vs What to Reuse

| Component | Use Existing | Build Custom | Notes |
|-----------|-------------|-------------|-------|
| CSS tokenizer | `cssparser` 0.35 | — | Handles Syntax Level 3, well-tested |
| Layout engine | `taffy` 0.9 | — | Best-in-class, widely adopted |
| Selector parser | Build on `cssparser` | Simple recursive descent | TCSS selectors are simple; ~200 lines |
| Selector matching | — | ~150 lines pattern match | Custom widget tree, simpler than `selectors` crate |
| Specificity | — | 3-tuple `(u32,u32,u32)` | Trivial to implement |
| Cascade resolution | — | Sort + apply | Straightforward |
| Property parser | — | `match name { ... }` dispatch | ~500 lines for full TCSS properties |
| Color parsing | `cssparser::color` | — | Already handles named colors, hex, rgb() |
| Computed style cache | — | HashMap/arena per node | Cache invalidation via dirty flags |
| Inline style props | — | Builder/modifier pattern | Override cascade at highest priority |

### Component Boundaries

```
StyleEngine
├── Parser
│   ├── Tokenizer (cssparser)
│   ├── SelectorParser (custom)
│   └── PropertyParser (custom, dispatches on name)
├── Cascade
│   ├── RuleCollector (gathers all stylesheets + inline)
│   ├── SpecificityCalculator
│   └── CascadeResolver (sorts and applies)
├── ComputedStyleTree (parallel to widget tree)
└── LayoutEngine
    ├── TaffyBridge (maps ComputedStyle → taffy::Style)
    ├── TaffyTree (taffy::TaffyTree instance)
    └── LayoutCache (node → Rect mapping)
```

### Suggested Crate Dependencies

```toml
[dependencies]
taffy = "0.9"            # Layout engine
cssparser = "0.35"       # CSS tokenization

[dev-dependencies]
# No additional deps needed for CSS parsing
```

---

## Sources

- [DioxusLabs/taffy GitHub](https://github.com/DioxusLabs/taffy) — HIGH confidence
- [taffy docs.rs](https://docs.rs/taffy/latest/taffy/) — HIGH confidence
- [servo/rust-cssparser GitHub](https://github.com/servo/rust-cssparser) — HIGH confidence
- [servo/stylo (selectors crate)](https://github.com/servo/stylo/tree/main/selectors) — HIGH confidence
- [lightningcss GitHub](https://github.com/parcel-bundler/lightningcss) — HIGH confidence
- [Ratatui Layout Concepts](https://ratatui.rs/concepts/layout/) — HIGH confidence
- [Textual CSS Guide](https://textual.textualize.io/guide/CSS/) — HIGH confidence
- [Textual Styles Reference](https://textual.textualize.io/guide/styles/) — HIGH confidence
- [iocraft GitHub](https://github.com/ccbrown/iocraft) — MEDIUM confidence
- [Vizia styling docs](https://book.vizia.dev/quickstart/styling.html) — MEDIUM confidence
- [vizia/morphorm GitHub](https://github.com/vizia/morphorm) — MEDIUM confidence
- [Robinson browser engine: Style chapter](https://limpet.net/mbrubeck/2014/08/23/toy-layout-engine-4-style.html) — MEDIUM confidence (2014, but pattern is timeless)
- [rust-selectors archived](https://github.com/servo/rust-selectors) — HIGH confidence (archive notice)
- [Taffy v0.9 grid-template-areas](https://github.com/bevyengine/bevy/pull/21672) — MEDIUM confidence
