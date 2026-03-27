# textual-rs CSS Property Reference

textual-rs uses TCSS (Textual CSS) for styling. Stylesheets are parsed and applied via a specificity-based cascade. This document covers every supported property, value format, and theme variable.

## Selectors

### Type Selector

Matches widgets by their `widget_type_name()`. Convention is PascalCase.

```css
Button { color: white; }
MyCustomWidget { background: #1e1e1e; }
```

### Class Selector

Matches widgets that return the class name from `classes()`.

```css
.primary { background: #0178d4; }
.error { color: #ba3c5b; }
```

### ID Selector

Matches a single widget by the value returned from `id()`.

```css
#sidebar { width: 20; }
#main-content { flex-grow: 1; }
```

### Universal Selector

Matches all widgets.

```css
* { color: white; }
```

### Pseudo-class Selector

Matches widgets in a specific state. Append to any other selector.

```css
Button:focus { border: tall #00ffa3; }
Input:hover { background: #2a2a3a; }
*:disabled { opacity: 0.5; }
```

Supported pseudo-classes:

| Pseudo-class | When active |
|-------------|-------------|
| `:focus` | Widget has keyboard focus |
| `:hover` | Mouse cursor is over the widget |
| `:disabled` | Widget is in disabled state |

---

## Layout Properties

### display

Controls the layout algorithm.

| Value | Description |
|-------|-------------|
| `flex` | Flexbox layout (default) |
| `grid` | CSS Grid layout |
| `block` | Block layout |
| `none` | Widget is not rendered |

```css
MyWidget { display: flex; }
GridContainer { display: grid; }
```

### layout-direction

Controls the main axis for flex children.

| Value | Description |
|-------|-------------|
| `vertical` | Stack children top-to-bottom (default) |
| `horizontal` | Arrange children left-to-right |

```css
Sidebar { layout-direction: vertical; }
Toolbar { layout-direction: horizontal; }
```

### flex-grow

Proportion of remaining space this widget should consume. Default: `0`.

```css
ChatLog { flex-grow: 1; }
Sidebar { flex-grow: 0; }   /* Does not grow */
```

Multiple siblings with `flex-grow` share space proportionally (e.g., `1` and `2` gives 1/3 and 2/3).

### width / height

Set explicit dimensions.

| Value | Description |
|-------|-------------|
| `auto` | Determined by content and layout (default) |
| `30` | Fixed columns/rows |
| `50%` | Percentage of parent |
| `1fr` | Fractional unit (grid/flex) |

```css
Sidebar { width: 20; }
Header { height: 1; }
Panel { width: 50%; }
```

### min-width / min-height / max-width / max-height

Constrain dimensions.

```css
Button { min-width: 16; }
DataTable { min-height: 5; }
Panel { max-height: 50%; }
```

### padding

Space between the widget's border and its content. Specified in character cells.

```css
/* All sides */
MyWidget { padding: 1; }

/* Vertical Horizontal */
MyWidget { padding: 1 2; }

/* Top Right Bottom Left */
MyWidget { padding: 1 2 3 4; }
```

### margin

Space outside the widget's border.

```css
MyWidget { margin: 1; }
MyWidget { margin: 0 1; }
MyWidget { margin: 1 2 1 2; }
```

### dock

Pin a widget to an edge of its parent container. Docked widgets are removed from normal flow.

| Value | Description |
|-------|-------------|
| `top` | Pin to top edge |
| `bottom` | Pin to bottom edge |
| `left` | Pin to left edge |
| `right` | Pin to right edge |

```css
Header { dock: top; height: 1; }
Footer { dock: bottom; height: 1; }
```

### overflow

Controls content overflow behavior.

| Value | Description |
|-------|-------------|
| `visible` | Content is not clipped (default) |
| `hidden` | Content is clipped |
| `scroll` | Scrollbars are always shown |
| `auto` | Scrollbars shown when content overflows |

```css
ScrollView { overflow: auto; }
```

### scrollbar-gutter

Reserve space for scrollbar even when not scrolling.

```css
ScrollView { scrollbar-gutter: true; }
```

---

## Grid Properties

### grid-template-columns / grid-template-rows

Define grid track sizes. Accepts space-separated dimension values.

```css
GridContainer {
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    grid-template-rows: auto 1fr;
}
```

Fractional units (`fr`) distribute remaining space proportionally.

### keyline

Draw separator lines between grid children.

```css
GridContainer {
    display: grid;
    keyline: $primary;
}
```

Accepts any color value (hex, rgb, theme variable).

---

## Visual Properties

### color

Foreground text color.

```css
Label { color: #e0e0e0; }
Button { color: $foreground; }
```

### background

Background fill color.

```css
Panel { background: #1e1e1e; }
Screen { background: $background; }
```

### border

Border style around the widget. Optionally includes a border color.

```css
/* Style only */
Input { border: rounded; }

/* Style + color */
Input { border: tall #4a4a5a; }
Input { border: inner $primary; }
```

### Border Styles

| Style | Characters | Description |
|-------|-----------|-------------|
| `none` | (none) | No border |
| `solid` | `+-\|` | Single-line box drawing |
| `rounded` | `╭╮╰╯│─` | Rounded corners |
| `heavy` | `┏┓┗┛┃━` | Thick/heavy lines |
| `double` | `╔╗╚╝║═` | Double-line box drawing |
| `ascii` | `+-\|` | Plain ASCII characters |
| `tall` | `▀▄▐▌` | Half-block characters for thin frames |
| `inner` / `mcgugan-box` | `▁▔▎` | One-eighth-block ultra-thin borders (the signature Textual style) |

The `tall` and `inner`/`mcgugan-box` styles produce the thinnest possible borders using Unicode block elements, giving a polished modern look.

### border-title

Text displayed in the border's top edge.

```css
Input { border: rounded; border-title: "Username"; }
```

### text-align

Horizontal text alignment within the widget.

| Value | Description |
|-------|-------------|
| `left` | Left-aligned (default) |
| `center` | Centered |
| `right` | Right-aligned |

```css
Button { text-align: center; }
```

### visibility

Toggle widget visibility.

| Value | Description |
|-------|-------------|
| `visible` | Rendered normally (default) |
| `hidden` | Space reserved but not painted |

```css
.hidden { visibility: hidden; }
```

### opacity

Widget opacity from `0.0` (fully transparent) to `1.0` (fully opaque). Default: `1.0`.

```css
*:disabled { opacity: 0.5; }
```

### hatch

Background fill pattern using Unicode characters.

| Value | Description |
|-------|-------------|
| `cross` | Cross-hatch pattern (braille dots) |
| `horizontal` | Horizontal lines |
| `vertical` | Vertical lines |
| `left` | Diagonal lines (top-right to bottom-left) |
| `right` | Diagonal lines (top-left to bottom-right) |

```css
Placeholder { hatch: cross; }
```

---

## Color Formats

### Hex Colors

```css
MyWidget {
    color: #ffffff;       /* 6-digit hex */
    background: #1e1e1e;
}
```

### RGB Function

```css
MyWidget {
    color: rgb(255, 255, 255);
    background: rgb(30, 30, 30);
}
```

### Named Colors

Standard terminal color names:

```css
MyWidget { color: red; }
MyWidget { color: white; }
```

### Theme Variables

Reference the active theme's color palette with `$`:

```css
MyWidget {
    color: $foreground;
    background: $primary;
}
```

### Lighten / Darken Modifiers

Adjust theme colors by appending `-lighten-N` or `-darken-N` (N = 1, 2, or 3):

```css
MyWidget {
    background: $primary-lighten-1;   /* Slightly lighter */
    color: $primary-lighten-2;        /* Lighter still */
    border: tall $accent-darken-1;    /* Slightly darker accent */
}
```

The step size is controlled by the theme's `luminosity_spread` (default 0.15). Each level adjusts luminosity by `spread / 2`.

---

## Theme Variables Reference

All built-in themes provide these variables:

| Variable | Semantic Role | textual-dark Default |
|----------|---------------|---------------------|
| `$primary` | Primary brand color, active elements | `rgb(1, 120, 212)` |
| `$secondary` | Secondary/muted variant of primary | `rgb(0, 69, 120)` |
| `$accent` | Accent/highlight color for emphasis | `rgb(255, 166, 43)` |
| `$surface` | Elevated surface (cards, panels base) | `rgb(30, 30, 30)` |
| `$panel` | Panel background (blended surface+primary) | `rgb(27, 39, 48)` |
| `$background` | Page/screen background | `rgb(18, 18, 18)` |
| `$foreground` | Default text color | `rgb(224, 224, 224)` |
| `$success` | Success state color | `rgb(78, 191, 113)` |
| `$warning` | Warning state color | `rgb(255, 166, 43)` |
| `$error` | Error/danger state color | `rgb(186, 60, 91)` |

Each variable also supports shade suffixes:

- `$primary-lighten-1`, `$primary-lighten-2`, `$primary-lighten-3`
- `$primary-darken-1`, `$primary-darken-2`, `$primary-darken-3`

This applies to all 10 base variables (e.g., `$accent-lighten-2`, `$error-darken-1`).

---

## Built-in Widget Defaults

The framework applies these default styles at the lowest priority. Your stylesheets always override them.

```css
Button        { border: heavy; min-width: 16; height: 3; text-align: center; }
Checkbox      { height: 1; }
Collapsible   { min-height: 1; }
DataTable     { border: rounded; min-height: 5; }
Footer        { height: 1; }
Header        { height: 1; }
Horizontal    { layout-direction: horizontal; }
Input         { border: rounded; height: 3; }
Label         { min-height: 1; }
ListView      { min-height: 3; flex-grow: 1; }
Log           { min-height: 3; flex-grow: 1; }
Markdown      { min-height: 3; }
Placeholder   { border: rounded; min-height: 3; min-width: 10; }
ProgressBar   { height: 1; }
RadioButton   { height: 1; }
RadioSet      { layout-direction: vertical; }
ScrollView    { overflow: auto; }
Select        { border: rounded; height: 3; }
Sparkline     { height: 1; }
Switch        { height: 1; }
TabbedContent { min-height: 3; layout-direction: vertical; }
TabBar        { height: 1; }
TextArea      { border: rounded; min-height: 5; }
Tree          { border: rounded; min-height: 5; }
Vertical      { layout-direction: vertical; }
```
