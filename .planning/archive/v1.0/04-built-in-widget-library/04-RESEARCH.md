# Phase 4: Built-in Widget Library - Research

**Researched:** 2026-03-25
**Domain:** Rust TUI widget implementation (ratatui + crossterm) with reactive state, TCSS styling, and snapshot testing
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Two-wave batching: input widgets first (Label, Button, Input, TextArea, Checkbox, Switch, RadioButton/RadioSet, Select), then display/layout widgets (ListView, DataTable, Tree, ProgressBar, Sparkline, Log, Markdown, Tabs/TabbedContent, Collapsible, Vertical/Horizontal, ScrollView, Header, Footer, Placeholder).
- **D-02:** Each widget follows the same pattern: struct implementing `Widget` trait, `default_css()` for TCSS defaults, `render()` writing to ratatui `Buffer`, `on_event()` for keyboard/mouse handling, message types for parent communication, and at least one `TestApp` snapshot test plus keyboard interaction tests for interactive widgets.
- **D-03:** Simple offset-based scrolling for v1. ScrollView, ListView, DataTable, and Tree track a `scroll_offset: Reactive<usize>` (line/row offset). Content renders from offset into the viewport area. Scrollbar is a visual indicator showing position, not interactive in v1. Virtual scrolling deferred to v2.
- **D-04:** TextArea: basic multi-line editor with cursor movement (arrows, Home/End, Ctrl+arrow word jump), text selection (Shift+arrow), optional line numbers, copy/paste via clipboard. No syntax highlighting, undo/redo, soft wrapping (v2).
- **D-05:** Select renders as single-line display; Enter pushes a temporary overlay screen with option list; selecting pops the overlay and emits `Select::Changed`. Reuses screen stack from Phase 2.
- **D-06:** Markdown renders headers, bold, italic, code spans, code blocks, bullet/numbered lists, horizontal rules, links as `text [url]`. No images, tables, HTML. Parse via pulldown-cmark.
- **D-07:** Message naming: `Button::Pressed`, `Input::Changed { value: String }`, `Checkbox::Changed { checked: bool }`, `Select::Changed { value: String, index: usize }`. All implement `Message` trait.
- **D-08:** Interactive widgets use `Reactive<T>` for primary state: `Input { value: Reactive<String> }`, `Checkbox { checked: Reactive<bool> }`, `ProgressBar { progress: Reactive<f64> }`.
- **D-09:** Every widget provides `default_css()` matching Textual's default visual appearance. The Python Textual source in `textual/` is the primary reference for default style values.

### Claude's Discretion

- Exact keyboard shortcuts for each interactive widget (follow Textual conventions where possible)
- Internal data structures for DataTable (Vec<Vec<Cell>> vs column-major)
- Tree node expansion/collapse animation (if any)
- ProgressBar indeterminate mode animation approach
- Sparkline rendering algorithm (braille vs block chars)
- Log widget auto-scroll behavior details
- Header/Footer layout strategy
- Placeholder widget content generation

### Deferred Ideas (OUT OF SCOPE)

- Virtual scrolling for large lists (v2)
- TextArea syntax highlighting (v2 — WIDGET-V2-02)
- TextArea undo/redo (v2)
- ContentSwitcher widget (v2 — WIDGET-V2-01)
- DirectoryTree widget (v2 — WIDGET-V2-04)
- Table rendering in Markdown widget (v2)
- Interactive scrollbar (v2)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIDGET-01 | `Label` — static or reactive text display with markup support | Static text render to ratatui Buffer; Reactive<String> triggers re-render |
| WIDGET-02 | `Button` — pressable button with label, variants (primary/warning/error/success) | Enter/Space keypress → Button::Pressed; border+padding default CSS; variant field for color |
| WIDGET-03 | `Input` — single-line text input with placeholder, password mode, validation | Cursor position as Reactive<usize>; char insertion/deletion; Input::Changed on change |
| WIDGET-04 | `TextArea` — multi-line text editor with line numbers option | Vec<String> line storage; cursor (row, col) pair; Shift+arrow selection tracking |
| WIDGET-05 | `Checkbox` — toggleable boolean input | Reactive<bool> checked; Space/Enter toggle; Checkbox::Changed message |
| WIDGET-06 | `Switch` — toggle switch (on/off) | Visual slider using Unicode chars; Enter/Space toggle; Switch::Changed message |
| WIDGET-07 | `RadioButton` / `RadioSet` — mutually exclusive selection | RadioSet owns group of RadioButtons; on child Changed, clear others |
| WIDGET-08 | `Select` — dropdown selection widget | Screen stack push for overlay; Select::Changed on selection |
| WIDGET-09 | `ListView` — scrollable list with selectable items | scroll_offset: Reactive<usize>; Up/Down navigation; ListView::Selected message |
| WIDGET-10 | `DataTable` — tabular data display with sortable columns, scrolling | Column defs + row Vec<Vec<String>>; scroll_offset_row + scroll_offset_col; DataTable::RowSelected |
| WIDGET-11 | `Tree` — hierarchical tree view with expand/collapse | TreeNode struct with children Vec and expanded bool; Space/Enter toggles; Tree::NodeSelected |
| WIDGET-12 | `ProgressBar` — determinate and indeterminate progress display | progress: Reactive<f64> (0.0–1.0); None = indeterminate with tick counter |
| WIDGET-13 | `Sparkline` — inline chart widget | Vec<f64> data; braille block chars for rendering height encoding |
| WIDGET-14 | `Log` — scrolling log display with auto-scroll | Vec<String> lines; auto-scroll to bottom on new entry unless manually scrolled up |
| WIDGET-15 | `Markdown` — rendered Markdown display | pulldown-cmark parser; map tags to ratatui Style; code blocks with border |
| WIDGET-16 | `Tabs` / `TabbedContent` — tabbed container navigation | active_tab: Reactive<usize>; Left/Right keys switch tabs; Tabs::TabChanged message |
| WIDGET-17 | `Collapsible` — expand/collapse container | expanded: Reactive<bool>; title row focusable; Enter toggles; compose() returns children only when expanded |
| WIDGET-18 | `Vertical` / `Horizontal` — layout container widgets | Thin wrappers that set layout_direction in default_css() |
| WIDGET-19 | `ScrollView` — scrollable container with optional scrollbars | scroll_offset_x + scroll_offset_y: Reactive<usize>; overflow:hidden clip |
| WIDGET-20 | `Header` — application header bar with title/subtitle | Docked top, height:1, title: Reactive<String>; no clock in v1 (timer deferred) |
| WIDGET-21 | `Footer` — key binding help bar | Reads focused widget's key_bindings(); renders key+description pairs; docked bottom |
| WIDGET-22 | `Placeholder` — development placeholder widget | Renders widget name + area dimensions; border:rounded; fixed color scheme |
</phase_requirements>

---

## Summary

Phase 4 implements all 22 v1 widgets as `Box<dyn Widget>` structs on top of the infrastructure built in Phases 1–3. The foundation is solid: the `Widget` trait, `AppContext` arena, `Reactive<T>` signals, `TestApp`/`Pilot` harness, and `dispatch_message`/`post_message` event system are all verified passing at 99 + 17 tests. Every widget will be a new `.rs` file under `crates/textual-rs/src/widget/`, re-exported from `lib.rs`.

The two main external additions needed are `pulldown-cmark` (0.13.3, MIT) for WIDGET-15 Markdown parsing, and `arboard` (3.6.1) for TextArea clipboard support (D-04). Neither has breaking API changes from training-data assumptions — both are verified against crates.io. The `Reactive<T>` wrapper over `ArcRwSignal` covers all interactive widget state; `spawn_render_effect` handles render coalescing. The Python Textual source in `textual/src/textual/widgets/` is checked into the repo and is the primary reference for default CSS values, keyboard shortcuts, and message naming.

The most complex widgets are DataTable (column defs, row storage, two-axis scrolling, sorting) and TextArea (cursor model, selection, clipboard). Both should be planned as their own sub-tasks with extra buffer. The simplest widgets (Label, Placeholder, Vertical/Horizontal, Header, Footer) can be batched efficiently.

**Primary recommendation:** Follow D-01's wave order strictly. Validate Wave 1 widgets (8 input widgets) against the test harness before writing Wave 2. Each widget file is self-contained; no shared state beyond AppContext. Add `pulldown-cmark` as a regular dependency and `arboard` as a conditional dependency (TextArea only) in the first planning wave.

---

## Standard Stack

### Core (already in Cargo.toml — no new deps needed for most widgets)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 | Buffer rendering — all widgets write cells here | Already dependency; provides Rect, Buffer, Style |
| crossterm | 0.29.0 | KeyCode, KeyModifiers for on_event/key_bindings | Already dependency; crossterm KeyEvent is the input type |
| reactive_graph | 0.2.13 | ArcRwSignal backing Reactive<T> | Already dependency; signals trigger render coalescing |
| slotmap | 1.0 | WidgetId arena storage | Already dependency; AppContext.arena uses DenseSlotMap |

### New Dependencies Required

| Library | Version | Purpose | When to Add |
|---------|---------|---------|-------------|
| pulldown-cmark | 0.13.3 | CommonMark parsing for Markdown widget | Wave 2 (WIDGET-15); add to [dependencies] |
| arboard | 3.6.1 | OS clipboard for TextArea copy/paste | Wave 1 (WIDGET-04); add to [dependencies]; feature = [] (no image-data needed) |

**Installation (add to crates/textual-rs/Cargo.toml):**
```toml
pulldown-cmark = { version = "0.13.3", default-features = false }
arboard = { version = "3.6.1", default-features = false }
```

**Version verification (2026-03-25):**
- `pulldown-cmark` 0.13.3 — confirmed via `cargo search pulldown-cmark` and `cargo info`
- `arboard` 3.6.1 — confirmed via `cargo search arboard` and `cargo info`
- Both require Rust 1.71.0+; project MSRV is 1.88 — compatible

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| arboard (clipboard) | copypasta 0.10.2 | arboard is maintained by 1Password, wider platform support; copypasta has fewer recent releases |
| pulldown-cmark | comrak | comrak is heavier (GFM+extensions); pulldown-cmark is minimal and fits WIDGET-15 scope exactly |

---

## Architecture Patterns

### Widget File Layout

```
crates/textual-rs/src/widget/
├── mod.rs              # Widget trait (existing)
├── context.rs          # AppContext (existing)
├── tree.rs             # mount/unmount/focus (existing)
├── label.rs            # WIDGET-01
├── button.rs           # WIDGET-02
├── input.rs            # WIDGET-03
├── text_area.rs        # WIDGET-04
├── checkbox.rs         # WIDGET-05
├── switch.rs           # WIDGET-06
├── radio.rs            # WIDGET-07 (RadioButton + RadioSet)
├── select.rs           # WIDGET-08
├── list_view.rs        # WIDGET-09
├── data_table.rs       # WIDGET-10
├── tree_view.rs        # WIDGET-11 (named tree_view to avoid conflict with widget::tree)
├── progress_bar.rs     # WIDGET-12
├── sparkline.rs        # WIDGET-13
├── log.rs              # WIDGET-14
├── markdown.rs         # WIDGET-15
├── tabs.rs             # WIDGET-16 (Tabs + TabbedContent)
├── collapsible.rs      # WIDGET-17
├── layout.rs           # WIDGET-18 (Vertical + Horizontal)
├── scroll_view.rs      # WIDGET-19
├── header.rs           # WIDGET-20
├── footer.rs           # WIDGET-21
└── placeholder.rs      # WIDGET-22
```

### Pattern 1: Standard Widget Struct

Every widget follows this exact pattern (from D-02):

```rust
// Source: crates/textual-rs/src/widget/mod.rs (Widget trait)
// crates/textual-rs/src/widget/button.rs

use std::any::Any;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use crate::event::message::Message;
use crate::event::keybinding::KeyBinding;
use crate::widget::{EventPropagation, Widget, WidgetId};
use crate::widget::context::AppContext;
use crate::reactive::Reactive;
use crossterm::event::{KeyCode, KeyModifiers};

// --- Message types ---
pub mod messages {
    use super::*;
    pub struct Pressed;
    impl Message for Pressed {}
}

// --- Widget struct ---
pub struct Button {
    pub label: String,
    pub variant: ButtonVariant,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant { Default, Primary, Warning, Error, Success }

impl Widget for Button {
    fn widget_type_name(&self) -> &'static str { "Button" }
    fn can_focus(&self) -> bool { true }

    fn default_css() -> &'static str where Self: Sized {
        "Button { border: tall $primary; min-width: 16; height: 3; text-align: center; }"
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        &[
            KeyBinding { key: KeyCode::Enter, modifiers: KeyModifiers::NONE,
                         action: "press", description: "Press", show: false },
            KeyBinding { key: KeyCode::Char(' '), modifiers: KeyModifiers::NONE,
                         action: "press", description: "Press", show: false },
        ]
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        // widget_id for message source requires on_mount to store id
        // Use ctx.focused_widget as the source (safe for button)
        if action == "press" {
            if let Some(id) = ctx.focused_widget {
                ctx.post_message(id, messages::Pressed);
            }
        }
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // Write label centered in area
        // ...
    }
}
```

### Pattern 2: Reactive State Widget

```rust
// Source: crates/textual-rs/src/reactive/mod.rs (Reactive<T>)
// crates/textual-rs/src/widget/checkbox.rs

pub struct Checkbox {
    pub checked: Reactive<bool>,
    pub label: String,
    // own_id stored on mount for message sourcing
    own_id: std::cell::Cell<Option<WidgetId>>,
}

impl Widget for Checkbox {
    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }
    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "toggle" {
            let new_val = !self.checked.get_untracked();
            self.checked.set(new_val);
            if let Some(id) = self.own_id.get() {
                ctx.post_message(id, messages::Changed { checked: new_val });
            }
        }
    }
    // ...
}
```

**Key insight:** `on_mount` receives the widget's own `WidgetId` — store it in a `Cell<Option<WidgetId>>` to use as the message source in `on_action`/`on_event`. This is the correct pattern since `on_event`/`on_action` take `&self` not `&mut self`.

### Pattern 3: Scroll Offset Widget (D-03)

```rust
// For ListView, DataTable, Tree, ScrollView, Log
pub struct ListView {
    pub items: Vec<String>,
    pub selected: Reactive<usize>,
    pub scroll_offset: Reactive<usize>,
    own_id: std::cell::Cell<Option<WidgetId>>,
}

impl Widget for ListView {
    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        let offset = self.scroll_offset.get_untracked();
        let visible_height = area.height as usize;
        let items_slice = &self.items[offset..self.items.len().min(offset + visible_height)];
        // render scrollbar indicator on right column
        // ...
    }
    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "cursor_down" => {
                let next = (self.selected.get_untracked() + 1).min(self.items.len().saturating_sub(1));
                self.selected.set(next);
                // scroll if needed
                let offset = self.scroll_offset.get_untracked();
                // ...
            }
            _ => {}
        }
    }
}
```

### Pattern 4: Snapshot Test

```rust
// Source: crates/textual-rs/tests/snapshot_tests.rs
// crates/textual-rs/tests/widget_snapshots.rs

#[test]
fn snapshot_button_default() {
    use textual_rs::widget::button::Button;
    use textual_rs::testing::TestApp;
    let app = TestApp::new(40, 3, || Box::new(Button {
        label: "OK".to_string(),
        variant: ButtonVariant::Default,
    }));
    insta::assert_snapshot!(format!("{}", app.backend()));
}
```

### Pattern 5: Keyboard Interaction Test

```rust
// Source: crates/textual-rs/tests/test_harness.rs (existing Pilot usage)

#[tokio::test]
async fn checkbox_toggle_on_space() {
    use textual_rs::widget::checkbox::{Checkbox, messages};
    use std::sync::{Arc, Mutex};
    // ... build test screen with checkbox + message capture ...
    let mut app = TestApp::new(40, 3, || Box::new(TestScreen::new()));
    let mut pilot = app.pilot();
    pilot.press(KeyCode::Tab).await;      // focus checkbox
    pilot.press(KeyCode::Char(' ')).await; // toggle
    pilot.settle().await;
    // assert state changed
}
```

### Pattern 6: Select Overlay (D-05)

The Select widget pushes an overlay screen via the screen stack:
```rust
fn on_action(&self, action: &str, ctx: &AppContext) {
    if action == "open" {
        // Push SelectOverlay screen onto ctx.screen_stack
        // SelectOverlay renders the option list, on Enter pops itself
        // and posts Select::Changed to the originating widget
    }
}
```

This requires `ctx` to expose a `push_screen` operation callable from `on_action`. The existing `push_screen` function in `widget/tree.rs` takes `&mut AppContext` — this may require wrapping in `RefCell` or adding a `pending_screens: RefCell<Vec<Box<dyn Widget>>>` queue similar to `message_queue`. **This is a design decision for the planner to address.**

### Anti-Patterns to Avoid

- **Storing `WidgetId` in widget constructor args:** IDs are only known after `on_mount`. Use `Cell<Option<WidgetId>>` pattern.
- **Using `Reactive::get()` inside `render()`:** This creates tracking dependencies that fire render effects during render, causing infinite loops. Use `get_untracked()` in render paths.
- **Calling `ctx.post_message()` from `render()`:** render is called during the render pass. Post messages from `on_action`/`on_event` only.
- **Implementing `default_css()` as instance method:** It is a `where Self: Sized` static method — the signature must match the trait exactly.
- **Naming a widget file `tree.rs`:** Conflicts with `widget/tree.rs` (the arena tree utilities). Use `tree_view.rs` for WIDGET-11.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown parsing | Custom tokenizer | pulldown-cmark 0.13.3 | CommonMark compliance; handles escaped chars, nested structures, edge cases |
| Clipboard read/write | Direct WinAPI/X11 calls | arboard 3.6.1 | Handles Windows/macOS/Linux; wayland vs x11 detection; image-data not needed |
| Unicode character width | `len()` on strings | `unicode_width` crate (already used by ratatui) | CJK chars are 2 cells wide; `UnicodeWidthStr::width()` |
| Braille sparkline encoding | Custom bit manipulation | ratatui `symbols::braille` module | ratatui already provides `BRAILLE` symbol constants; use them directly |
| Block char selection for sparkline | Manual lookup table | `"▁▂▃▄▅▆▇█"` literal slice | 8 block chars cover 8 height levels — a simple `chars()[index]` lookup is fine |

**Key insight:** ratatui already provides `symbols::block::FULL`, `symbols::block::SEVEN_EIGHTHS`, etc., and braille symbols. Use these for Sparkline and ProgressBar rather than embedding Unicode codepoints as magic literals.

---

## Widget-by-Widget Implementation Notes

### Wave 1: Input Widgets

**WIDGET-01 Label**
- Renders text string into buffer with word-wrap optional (v1: single-line truncation)
- Python ref uses `Static` base class; in Rust this is just `render()` writing to buf
- `default_css()`: `"Label { width: auto; height: auto; min-height: 1; }"`
- No `can_focus()` — Label is non-interactive

**WIDGET-02 Button**
- Variants: Default, Primary, Warning, Error, Success — stored as enum field
- `can_focus() = true`; key_bindings: Enter + Space → "press" action
- `default_css()`: border:tall, min-width:16, height:3, text-align:center
- Message: `Button::Pressed` — no payload; bubble = true
- Python ref: `_button.py` DEFAULT_CSS uses `border: block $surface` for default variant

**WIDGET-03 Input**
- `value: Reactive<String>`, `cursor_pos: Cell<usize>`, `placeholder: String`, `password: bool`
- Key bindings: Left/Right (cursor), Home/End, Ctrl+Left/Right (word jump), Backspace/Delete, Char(*) for insertion
- Messages: `Input::Changed { value: String }` on each keystroke, `Input::Submitted { value: String }` on Enter
- Python ref: `_input.py` BINDINGS list — 20+ bindings covering all cursor movements
- `can_focus() = true`

**WIDGET-04 TextArea**
- `lines: Vec<String>`, `cursor: Cell<(usize, usize)>` (row, col), `selection: Cell<Option<(usize,usize),(usize,usize)>>`
- Clipboard via `arboard::Clipboard::new()` — create fresh per paste/copy operation (crossterm doesn't manage clipboard lifetime)
- Key bindings: all Input bindings plus Up/Down, Ctrl+A select-all, Ctrl+C copy, Ctrl+V paste, Ctrl+X cut
- Line numbers: `show_line_numbers: bool` constructor field; rendered in left margin
- Messages: `TextArea::Changed { value: String }` where value is full text joined with "\n"
- This is the most stateful Wave 1 widget; plan as its own sub-task

**WIDGET-05 Checkbox**
- `checked: Reactive<bool>`, `label: String`
- Renders: `"[x] label"` or `"[ ] label"` — Unicode checkmark optional
- Key bindings: Space + Enter → "toggle"
- Message: `Checkbox::Changed { checked: bool }`

**WIDGET-06 Switch**
- `value: Reactive<bool>`, `label: String`
- Renders as `"━━━━◉"` (on) or `"◉━━━━"` (off) — use block/line chars
- Python ref: `_switch.py` uses ScrollBarRender internally for the slider animation — for v1 use static characters
- Key bindings: Enter + Space → "toggle"
- Message: `Switch::Changed { value: bool }`

**WIDGET-07 RadioButton / RadioSet**
- `RadioButton { checked: Reactive<bool>, label: String }` — thin wrapper
- `RadioSet { buttons: Vec<RadioButton>, selected: Reactive<usize> }`
- RadioSet implements `compose()` returning its RadioButton children
- When RadioSet receives `RadioButton::Changed` via bubbling, it clears all other buttons
- Python ref: RadioButton char is `●` (`\u25cf`) for checked, `○` for unchecked
- Messages: `RadioSet::Changed { index: usize, value: String }`

**WIDGET-08 Select**
- `options: Vec<String>`, `selected: Reactive<usize>`
- Renders: `"▼ Option Name"` in a bordered box
- Screen push for overlay requires a mechanism to push a screen from within `on_action(&self, ...)`. The planner must decide: add `pending_screen_pushes: RefCell<Vec<Box<dyn Widget>>>` to AppContext, drained similarly to message_queue.
- Messages: `Select::Changed { value: String, index: usize }`

### Wave 2: Display/Layout Widgets

**WIDGET-09 ListView**
- `items: Vec<String>`, `selected: Reactive<usize>`, `scroll_offset: Reactive<usize>`
- Up/Down navigation; Enter selects
- Scrollbar: right column visual bar showing `selected / items.len()` position
- Messages: `ListView::Selected { index: usize, value: String }`, `ListView::Highlighted { index: usize }`

**WIDGET-10 DataTable**
- Internal structure (Claude's discretion): `columns: Vec<ColumnDef>`, `rows: Vec<Vec<String>>`
- Row-major storage is simpler for render; column-major only needed for sort (sort by column = sort rows by column index)
- `scroll_offset_row: Reactive<usize>`, `scroll_offset_col: Reactive<usize>`
- Cursor: `cursor_row: Reactive<usize>`, `cursor_col: Reactive<usize>`
- Sorting: `sort_column: Option<usize>`, `sort_ascending: bool` — toggle on column header press
- Messages: `DataTable::RowSelected { row: usize }`, `DataTable::SortChanged { column: usize, ascending: bool }`
- Python ref: `_data_table.py` uses TwoWayDict for row/col key lookups — v1 can use simple Vec indices

**WIDGET-11 Tree (tree_view.rs)**
- `TreeNode { label: String, data: Option<String>, children: Vec<TreeNode>, expanded: bool }`
- Flat render list: walk tree in pre-order, skip collapsed subtrees
- `scroll_offset: Reactive<usize>`, `cursor: Reactive<usize>` (index into flat render list)
- Guide chars: `"├── "`, `"└── "`, `"│   "` (standard tree drawing)
- Messages: `Tree::NodeSelected { path: Vec<usize> }`, `Tree::NodeExpanded`, `Tree::NodeCollapsed`

**WIDGET-12 ProgressBar**
- `progress: Reactive<Option<f64>>` — None = indeterminate
- Determinate: fill bar left-to-right using `"█"` and `"░"` chars
- Indeterminate: tick counter drives a bouncing `"███"` block using `EVENT-08` timer — if timer not convenient, a `tick: Cell<u8>` updated on render is acceptable for v1
- Python ref: `_progress_bar.py` uses `Bar` sub-widget and `ETA` helper — v1 renders inline without sub-widgets

**WIDGET-13 Sparkline**
- `data: Reactive<Vec<f64>>`
- Block chars: `"▁▂▃▄▅▆▇█"` — 8 levels; normalize data to [0,7] and index
- One cell per data point; clip to widget width
- Color: min value uses dimmer color, max uses brighter (Claude's discretion on exact colors)

**WIDGET-14 Log**
- `lines: Reactive<Vec<String>>`, `scroll_offset: Reactive<usize>`, `auto_scroll: bool`
- `push_line(&self, line: String)` method that appends to lines and if auto_scroll is true, sets scroll_offset to show last line
- Auto-scroll behavior: if scroll_offset == last_visible_line before push, auto-advance; if user manually scrolled up, disable auto-scroll until scroll to bottom

**WIDGET-15 Markdown**
- `content: String` (static for v1 — no Reactive needed unless dynamic markdown)
- Parse with `pulldown-cmark`: iterate `Event` enum
- Tag → ratatui Style mapping: Heading → bold + bright, CodeBlock → bordered block, Strong → bold, Emphasis → italic
- Links: render as `"text [url]"` with dim style on url part
- Store rendered `Vec<(String, ratatui::style::Style)>` lines in a pre-computed field, recomputed on content change

**WIDGET-16 Tabs / TabbedContent**
- `Tabs { tab_labels: Vec<String>, active: Reactive<usize> }`
- Renders tab bar: `" Label1  | Label2  | Label3 "` with active tab highlighted
- `TabbedContent { tabs: Tabs, panes: Vec<Box<dyn Widget>> }` — compose returns Tabs + active pane
- Left/Right arrows switch active tab
- Messages: `Tabs::TabChanged { index: usize, label: String }`

**WIDGET-17 Collapsible**
- `title: String`, `expanded: Reactive<bool>`, `children: Vec<Box<dyn Widget>>`
- Title row: renders `"▶ Title"` (collapsed) or `"▼ Title"` (expanded)
- `compose()` returns `[title_widget]` when collapsed, `[title_widget] + children` when expanded
- Enter on title row toggles
- Messages: `Collapsible::Expanded`, `Collapsible::Collapsed`
- Python ref: `_collapsible.py` uses `CollapsibleTitle` sub-widget that posts `Toggle` message; v1 integrates this logic directly in the Collapsible widget

**WIDGET-18 Vertical / Horizontal**
- Thin layout containers with no render logic
- `default_css()` sets `layout: vertical` or `layout: horizontal` + `width: 1fr; height: 1fr`
- Used to compose children into flex layouts

**WIDGET-19 ScrollView**
- `scroll_offset_x: Reactive<usize>`, `scroll_offset_y: Reactive<usize>`
- Renders children into a virtual buffer larger than the visible area, then clips
- Arrow keys scroll; mouse scroll events increment offset
- Visual scrollbar: right column (vertical) and bottom row (horizontal) — 1-char indicator

**WIDGET-20 Header**
- `title: Reactive<String>`, `subtitle: Reactive<String>`
- Docked top, height:1
- Renders: `" title — subtitle "` centered
- Python ref: `_header.py` has HeaderIcon (left), HeaderTitle (center), HeaderClock (right); v1 renders in one widget without sub-widgets (no clock timer complexity)

**WIDGET-21 Footer**
- Reads `ctx.focused_widget` → looks up that widget's `key_bindings()` → renders `key description` pairs
- Docked bottom, height:1
- Each binding shown as `"[KEY] Description"` separated by spaces
- Respects `KeyBinding::show` field — only display bindings where `show = true`
- This widget needs read access to the focused widget's `key_bindings()`, meaning it must look up the arena entry from `AppContext`

**WIDGET-22 Placeholder**
- Renders: widget type name + area dimensions (`"Placeholder\n80×24"`)
- `border: rounded`, default color scheme (dimmed)
- Used during layout development; no interactivity

---

## Common Pitfalls

### Pitfall 1: WidgetId Not Available at Construction Time
**What goes wrong:** Widget tries to call `ctx.post_message(self_id, ...)` but `self_id` was never stored.
**Why it happens:** `WidgetId` is only assigned by the arena in `mount_widget()`, called after construction.
**How to avoid:** Add `own_id: std::cell::Cell<Option<WidgetId>>` to each interactive widget; set it in `on_mount`; unwrap with `.expect()` or early return in on_action.
**Warning signs:** `post_message` called with a hardcoded dummy ID; messages arrive from wrong source.

### Pitfall 2: Reactive::get() Tracked in Render
**What goes wrong:** Calling `self.value.get()` inside `render()` registers a reactive dependency; the dependency fires a `RenderRequest`; which calls `render()` again — infinite loop.
**Why it happens:** `ArcRwSignal::get()` registers the current tracking context; `render()` runs inside the render effect's scope.
**How to avoid:** Always use `self.value.get_untracked()` inside `render()`. Use `self.value.get()` only inside `Effect` closures.
**Warning signs:** App freezes or consumes 100% CPU; settle() never returns in tests.

### Pitfall 3: Select Overlay Requires Screen Push from &self Context
**What goes wrong:** `push_screen` in `widget/tree.rs` takes `&mut AppContext`; `on_action` only receives `&AppContext`.
**Why it happens:** The architecture uses interior mutability for the message queue but not for the screen stack.
**How to avoid:** Add `pending_screen_pushes: RefCell<Vec<Box<dyn Widget>>>` to AppContext, following the same pattern as `message_queue`. Drain in the event loop after `drain_message_queue`. The planner must include this as a prerequisite task.
**Warning signs:** Select widget's open action panics or does nothing.

### Pitfall 4: Footer Requires Arena Access During Render
**What goes wrong:** Footer calls `ctx.arena[focused_id].key_bindings()` — but arena borrow may conflict if render is called while arena is borrowed elsewhere.
**Why it happens:** DenseSlotMap is not RefCell-wrapped; borrowing is checked at compile time.
**How to avoid:** Footer's render only reads (`&ctx.arena`), which is a shared borrow. As long as no `&mut` borrow is active during render, this is safe. Confirm in implementation that the render pass holds only `&AppContext`.
**Warning signs:** Borrow checker error in footer.rs referencing arena.

### Pitfall 5: Tree Widget Module Name Conflict
**What goes wrong:** `mod tree;` in `widget/mod.rs` already refers to `widget/tree.rs` (the arena utilities).
**Why it happens:** Two files would both want the name `tree`.
**How to avoid:** Name the Tree widget file `tree_view.rs` and the public struct `Tree` (the struct name doesn't conflict with the module name).
**Warning signs:** Compile error `ambiguous name tree`.

### Pitfall 6: Collapsible compose() with Dynamic Children
**What goes wrong:** `compose()` is called once at mount time in the current implementation; returning different children based on `expanded` state won't update the tree.
**Why it happens:** The current `compose_subtree` in `widget/tree.rs` is called once during `push_screen`/mount, not reactively.
**How to avoid:** Collapsible must use a different approach for v1: always compose all children but hide them via `visibility: hidden` or `display: none` CSS when collapsed, rather than adding/removing from the tree. Alternatively, treat `expanded` toggle as requiring a re-compose triggered from `on_action` — which would require `ctx.recompose_widget(id)` API. The planner must decide; the simpler option is the visibility CSS approach.
**Warning signs:** Collapsible only shows children on first expand, never hides them again.

### Pitfall 7: DataTable Column Width Calculation
**What goes wrong:** Columns render with wrong widths; content truncated or overflows.
**Why it happens:** Column width must be max(header_len, max(cell_len for each row)) — easy to get wrong with Unicode.
**How to avoid:** Use `unicode_width::UnicodeWidthStr::width()` for all cell width calculations. Pre-compute column widths on `add_column`/`add_row` rather than recalculating on every render.
**Warning signs:** CJK characters cause columns to misalign.

---

## Code Examples

### Message Type Pattern

```rust
// Source: crates/textual-rs/src/event/message.rs (Message trait)
// Convention from D-07

pub mod messages {
    use crate::event::message::Message;

    pub struct Pressed;
    impl Message for Pressed {}

    pub struct Changed {
        pub value: String,
    }
    impl Message for Changed {}

    pub struct Submitted {
        pub value: String,
    }
    impl Message for Submitted {}
}
```

### Rendering Text with Ratatui Style

```rust
// Source: ratatui 0.30 Buffer API
use ratatui::{buffer::Buffer, layout::Rect, style::{Color, Style, Modifier}};

fn render_button(label: &str, focused: bool, area: Rect, buf: &mut Buffer) {
    let style = if focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    // Center text in area
    let x = area.x + (area.width.saturating_sub(label.len() as u16)) / 2;
    let y = area.y + area.height / 2;
    buf.set_string(x, y, label, style);
}
```

### Scrollbar Indicator

```rust
// Simple visual scrollbar — 1 column wide, right edge of widget area
fn render_scrollbar(total: usize, offset: usize, viewport_height: usize, area: Rect, buf: &mut Buffer) {
    if total == 0 { return; }
    let bar_height = area.height as usize;
    let thumb_pos = (offset * bar_height) / total.max(1);
    let thumb_size = ((viewport_height * bar_height) / total.max(1)).max(1);
    for row in 0..bar_height {
        let sym = if row >= thumb_pos && row < thumb_pos + thumb_size { "█" } else { "░" };
        buf.set_string(area.x + area.width - 1, area.y + row as u16, sym,
                       ratatui::style::Style::default());
    }
}
```

### pulldown-cmark Usage

```rust
// Source: pulldown-cmark 0.13.3 docs
use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};

fn parse_markdown(input: &str) -> Vec<(String, ratatui::style::Style)> {
    let parser = Parser::new(input);
    let mut lines: Vec<(String, ratatui::style::Style)> = Vec::new();
    let base_style = ratatui::style::Style::default();
    let mut current_style = base_style;
    let mut current_line = String::new();

    for event in parser {
        match event {
            Event::Text(text) => current_line.push_str(&text),
            Event::Start(Tag::Heading { level, .. }) => {
                current_style = ratatui::style::Style::default()
                    .add_modifier(ratatui::style::Modifier::BOLD);
            }
            Event::End(TagEnd::Heading(_)) => {
                lines.push((current_line.drain(..).collect(), current_style));
                current_style = base_style;
            }
            // ...
            _ => {}
        }
    }
    lines
}
```

### Sparkline Block Chars

```rust
const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn sparkline_char(value: f64, max_value: f64) -> char {
    if max_value == 0.0 { return BLOCKS[0]; }
    let idx = ((value / max_value) * 7.0).round().clamp(0.0, 7.0) as usize;
    BLOCKS[idx]
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual buffer writing only | ratatui 0.30 provides Widget/StatefulWidget traits | ratatui 0.28+ | We write directly to Buffer not via ratatui Widget — consistent with existing codebase |
| Reactive state via channels | reactive_graph 0.2.13 ArcRwSignal | Phases 3 (2026) | Signals automatically trigger render; no manual notify() calls needed |
| push_screen from mut ctx | Pending: push via RefCell queue | Phase 4 (to add) | Select overlay needs this — must be planned |

---

## Open Questions

1. **Select Overlay Screen Push Mechanism**
   - What we know: `push_screen` in `widget/tree.rs` requires `&mut AppContext`; `on_action` receives only `&AppContext`.
   - What's unclear: Whether to add `pending_screen_pushes: RefCell<Vec<...>>` to AppContext, or give `on_action` a `&mut AppContext` signature change, or use a different mechanism.
   - Recommendation: Add `pending_screen_pushes: RefCell<Vec<Box<dyn Widget>>>` to AppContext following the exact same pattern as `message_queue`; drain it in `App::drain_message_queue` or a new `drain_pending_screens` method. This is the lowest-risk change. The planner should make this a prerequisite task in Wave 1 before implementing Select.

2. **Collapsible Dynamic Composition**
   - What we know: `compose()` is only called at mount; the widget tree is not re-composed on reactive changes.
   - What's unclear: Whether to implement visibility toggling or re-composition support.
   - Recommendation: v1 uses visibility CSS (`display: none` / `display: flex`) — set `ctx.computed_styles[child_id].display` on toggle. Simpler than re-composition. Requires that the CSS cascade system allows runtime style override (currently `inline_styles` field on AppContext supports this via `Declaration` injection).

3. **Footer Key Binding Discovery**
   - What we know: Footer needs focused widget's `key_bindings()`. `AppContext.arena` holds all widgets as `Box<dyn Widget>`.
   - What's unclear: Whether the Footer can borrow from the arena at render time without conflict.
   - Recommendation: Footer renders last in DFS order (it's a leaf), and `AppContext` is `&` during all renders — no conflict. Simply call `ctx.arena[focused_id].key_bindings()` in Footer's render().

4. **TextArea Clipboard on Windows**
   - What we know: `arboard` 3.6.1 supports Windows via clipboard API.
   - What's unclear: Whether clipboard operations work correctly in all Windows terminal contexts (Windows Terminal vs cmd vs conhost).
   - Recommendation: Wrap clipboard calls in `Result`; log error to stderr on failure; TextArea degrades gracefully without clipboard support rather than panicking.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | All widgets | Yes | 1.88 (MSRV confirmed) | — |
| cargo | Build | Yes | (workspace) | — |
| insta | Snapshot tests | Yes (dev-dep) | 1.46.3 | — |
| pulldown-cmark | WIDGET-15 | No (not in Cargo.toml yet) | 0.13.3 available | None — add to Cargo.toml |
| arboard | WIDGET-04 TextArea clipboard | No (not in Cargo.toml yet) | 3.6.1 available | Disable clipboard; TextArea still works without copy/paste |

**Missing dependencies with no fallback:**
- pulldown-cmark — required for Markdown widget; no in-tree alternative

**Missing dependencies with fallback:**
- arboard — TextArea clipboard is optional per D-04 ("if crossterm supports"); arboard is the correct solution but clipboard failure can be non-fatal

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + insta 1.46.3 snapshots + proptest 1.11.0 |
| Config file | None (standard Cargo test runner) |
| Quick run command | `cargo test --lib -q` |
| Full suite command | `cargo test` (lib + integration tests) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIDGET-01 | Label renders text | snapshot | `cargo test --test widget_snapshots label` | No — Wave 0 |
| WIDGET-02 | Button renders + Pressed on Enter | snapshot + interaction | `cargo test --test widget_tests button` | No — Wave 0 |
| WIDGET-03 | Input captures keystrokes, emits Changed | interaction | `cargo test --test widget_tests input` | No — Wave 0 |
| WIDGET-04 | TextArea multi-line edit, cursor movement | interaction | `cargo test --test widget_tests text_area` | No — Wave 0 |
| WIDGET-05 | Checkbox toggles on Space | interaction | `cargo test --test widget_tests checkbox` | No — Wave 0 |
| WIDGET-06 | Switch toggles, Changed message | interaction | `cargo test --test widget_tests switch` | No — Wave 0 |
| WIDGET-07 | RadioSet exclusive selection | interaction | `cargo test --test widget_tests radio` | No — Wave 0 |
| WIDGET-08 | Select opens overlay, Changed message | interaction | `cargo test --test widget_tests select` | No — Wave 0 |
| WIDGET-09 | ListView scroll, Selected message | interaction + snapshot | `cargo test --test widget_tests list_view` | No — Wave 0 |
| WIDGET-10 | DataTable scroll, sort, RowSelected | interaction + snapshot | `cargo test --test widget_tests data_table` | No — Wave 0 |
| WIDGET-11 | Tree expand/collapse, NodeSelected | interaction + snapshot | `cargo test --test widget_tests tree_view` | No — Wave 0 |
| WIDGET-12 | ProgressBar renders at 0%, 50%, 100% | snapshot | `cargo test --test widget_snapshots progress_bar` | No — Wave 0 |
| WIDGET-13 | Sparkline renders data sequence | snapshot | `cargo test --test widget_snapshots sparkline` | No — Wave 0 |
| WIDGET-14 | Log appends lines, auto-scrolls | interaction | `cargo test --test widget_tests log` | No — Wave 0 |
| WIDGET-15 | Markdown renders headers + code blocks | snapshot | `cargo test --test widget_snapshots markdown` | No — Wave 0 |
| WIDGET-16 | Tabs switch on Left/Right, TabChanged | interaction + snapshot | `cargo test --test widget_tests tabs` | No — Wave 0 |
| WIDGET-17 | Collapsible expand/collapse | interaction | `cargo test --test widget_tests collapsible` | No — Wave 0 |
| WIDGET-18 | Vertical/Horizontal layout direction | snapshot | `cargo test --test widget_snapshots layout` | No — Wave 0 |
| WIDGET-19 | ScrollView scrolls content | interaction | `cargo test --test widget_tests scroll_view` | No — Wave 0 |
| WIDGET-20 | Header renders title/subtitle | snapshot | `cargo test --test widget_snapshots header` | No — Wave 0 |
| WIDGET-21 | Footer shows key_bindings of focused widget | snapshot | `cargo test --test widget_snapshots footer` | No — Wave 0 |
| WIDGET-22 | Placeholder renders name + dimensions | snapshot | `cargo test --test widget_snapshots placeholder` | No — Wave 0 |

### Sampling Rate

- **Per widget commit:** `cargo test --lib -q` (unit tests, < 5 seconds)
- **Per wave merge:** `cargo test` (full suite including snapshots, < 30 seconds)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `crates/textual-rs/tests/widget_snapshots.rs` — snapshot tests for all 22 widgets
- [ ] `crates/textual-rs/tests/widget_tests.rs` — interaction tests for 12 interactive widgets
- [ ] Cargo.toml additions: `pulldown-cmark`, `arboard` in `[dependencies]`

*(No framework install needed — cargo test runner already configured)*

---

## Project Constraints (from CLAUDE.md)

No `CLAUDE.md` exists in the project root. Project conventions extracted from existing code and CONTEXT.md:

- Rust stable only (MSRV 1.88, confirmed in `Cargo.toml`)
- No `unsafe` code — slotmap arena avoids raw pointers (confirmed by Phase 2 decisions)
- Widget trait methods take `&self` not `&mut self` — all mutable state via `Reactive<T>` or `Cell<T>`
- Tests use `#[tokio::test]` for async tests (Pilot), plain `#[test]` for sync/snapshot tests
- Snapshot files committed to `crates/textual-rs/tests/snapshots/` (insta convention)
- `dispatch_message` / `post_message` for all widget-to-parent communication — no direct field mutation of other widgets
- Existing module structure: new widget files go in `crates/textual-rs/src/widget/`; re-exported from `lib.rs`

---

## Sources

### Primary (HIGH confidence)

- Python Textual source `textual/src/textual/widgets/` (checked into repo) — widget behavior, keyboard shortcuts, default CSS values for all 22 widgets
- `crates/textual-rs/src/widget/mod.rs` — Widget trait signature (verified 2026-03-25)
- `crates/textual-rs/src/widget/context.rs` — AppContext fields, post_message API (verified 2026-03-25)
- `crates/textual-rs/src/reactive/mod.rs` — Reactive<T>, get_untracked(), ArcRwSignal pattern (verified 2026-03-25)
- `crates/textual-rs/src/testing/mod.rs` + `pilot.rs` — TestApp/Pilot API (verified 2026-03-25)
- `crates/textual-rs/tests/` — existing test patterns (snapshot, interaction) (verified 2026-03-25)
- `cargo info pulldown-cmark` output — version 0.13.3, MIT, Rust 1.71.1+ (verified 2026-03-25)
- `cargo info arboard` output — version 3.6.1, MIT/Apache-2.0, Rust 1.71.0+ (verified 2026-03-25)
- All 99 lib tests + 17 integration tests passing (verified via `cargo test` 2026-03-25)

### Secondary (MEDIUM confidence)

- Python `_data_table.py`, `_tree.py`, `_tabs.py` scanned for data structure patterns — translated to Rust idioms; Python-specific patterns (TwoWayDict, LRUCache) replaced with simpler Vec-based v1 alternatives

### Tertiary (LOW confidence)

- Sparkline braille block char rendering approach — based on common TUI practice; not verified against specific ratatui symbol constants in this version. Implementer should run `cargo doc --open` for ratatui `symbols` module to confirm available constants.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all core deps verified in Cargo.toml; new deps verified via cargo registry
- Architecture patterns: HIGH — derived directly from existing working code; Widget trait, AppContext, Reactive<T> all inspected
- Pitfalls: HIGH (WidgetId timing, get() vs get_untracked()) — derived from code analysis; MEDIUM (Select overlay mechanism) — identified gap, solution proposed but not implemented
- Widget-by-widget notes: MEDIUM — derived from Python reference source + Rust trait analysis; individual keyboard shortcut lists not exhaustively verified

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable tech stack; no fast-moving dependencies)
